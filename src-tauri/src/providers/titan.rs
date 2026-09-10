use std::{fmt, time::Duration};

use reqwest::{
    blocking::{Client, Response},
    header::{HeaderValue, ACCEPT, CONTENT_TYPE, COOKIE},
    redirect::Policy,
    StatusCode, Url,
};
use scraper::{Html, Selector};
use serde_json::Value;

use overlay_core::map_yaw_to_bearing_deg;

use crate::islepilot::parser::QuestStatus;

use super::model::{
    ConnectionStatus, ProviderId, ProviderSnapshot, SharedFriend, SharedPlayer, SharedStatBar,
};

const PRIME_EN: [&str; 10] = [
    "Visit a Sanctuary while juvenile",
    "Be born from another player's nest",
    "Have all three nutrient groups at once",
    "Visit a Mass Migration zone",
    "Visit two different Migration zones",
    "Visit four Patrol zones",
    "Never acquire the Infertile mutation",
    "Never get Muscle Spasms (avoid cannibalism)",
    "Raise offspring to Subadult",
    "Play a small species (Hypsi · Troodon · Beipi · Dryo · Deino)",
];
const PRIME_VI: [&str; 10] = [
    "Ghé Sanctuary khi còn nhỏ",
    "Sinh ra từ tổ người chơi khác",
    "Ăn đủ 3 nhóm dinh dưỡng cùng lúc",
    "Ghé vùng Đại di cư (Mass Migration)",
    "Ghé 2 vùng Di cư khác nhau",
    "Ghé 4 vùng Tuần tra (Patrol)",
    "Không dính đột biến Vô sinh",
    "Không dính Co giật cơ (đừng ăn đồng loại)",
    "Nuôi con non tới bán trưởng thành",
    "Chơi loài nhỏ (Hypsi · Troodon · Beipi · Dryo · Deino)",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TitanError {
    LoginRequired,
    Temporary,
    InvalidResponse,
    MissingServer,
    InvalidOrigin,
}

impl fmt::Display for TitanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::LoginRequired => "Cần đăng nhập The Real Server VN.",
            Self::Temporary => "Tạm thời không thể kết nối The Real Server VN.",
            Self::InvalidResponse => "Dữ liệu The Real Server VN không hợp lệ.",
            Self::MissingServer => "Tài khoản chưa có server The Isle để theo dõi.",
            Self::InvalidOrigin => "Địa chỉ The Real Server VN không hợp lệ.",
        };
        f.write_str(message)
    }
}

pub fn discover_server_id(html: &str) -> Result<String, TitanError> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("#dino[data-sv]").map_err(|_| TitanError::InvalidResponse)?;
    let value = document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("data-sv"))
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 512)
        .ok_or(TitanError::MissingServer)?;
    Ok(value.to_string())
}

fn number(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn fraction_percent(value: Option<&Value>) -> Option<f64> {
    number(value)
        .filter(|value| (0.0..=1.0).contains(value))
        .map(|value| value * 100.0)
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn percent_bar(dino: &Value, key: &str) -> Option<SharedStatBar> {
    fraction_percent(dino.get(key)).map(SharedStatBar::from_percent)
}

fn health_bar(dino: &Value) -> Option<SharedStatBar> {
    let percent = fraction_percent(dino.get("health"))?;
    let hp = dino.get("hp").and_then(Value::as_array);
    match hp {
        Some(values) if values.len() == 2 => {
            let current = number(values.first());
            let max = number(values.get(1));
            match (current, max) {
                (Some(current), Some(max))
                    if current >= 0.0
                        && max > 0.0
                        && ((current / max * 100.0) - percent).abs() <= 5.0 =>
                {
                    Some(SharedStatBar::from_values(current, max))
                }
                _ => Some(SharedStatBar::from_percent(percent)),
            }
        }
        _ => Some(SharedStatBar::from_percent(percent)),
    }
}

fn prime_quests(dino: &Value) -> Vec<QuestStatus> {
    let Some(values) = dino.get("prime_dk").and_then(Value::as_array) else {
        return Vec::new();
    };
    if values.len() != 10 || values.iter().any(|value| !value.is_boolean()) {
        return Vec::new();
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| QuestStatus {
            text: PRIME_EN[index].to_string(),
            text_vi: Some(PRIME_VI[index].to_string()),
            completed: value.as_bool().unwrap_or(false),
        })
        .collect()
}

fn shared_friends(value: &Value, online: bool) -> Vec<SharedFriend> {
    if !online {
        return Vec::new();
    }
    value
        .get("friends")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(5)
        .filter_map(|friend| {
            let name = optional_string(friend.get("name"))?;
            let position_cm = friend
                .get("loc")
                .and_then(Value::as_array)
                .and_then(|values| titan_loc_to_world(values));
            Some(SharedFriend {
                slot: friend
                    .get("n")
                    .and_then(Value::as_u64)
                    .filter(|slot| (1..=5).contains(slot))
                    .map(|slot| slot as u8),
                name,
                dino_name: optional_string(friend.get("species")),
                online: position_cm.is_some(),
                position_cm,
                position_px: None,
            })
        })
        .collect()
}

pub fn normalize(value: &Value, received_at_ms: i64) -> Result<ProviderSnapshot, TitanError> {
    if value.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(TitanError::InvalidResponse);
    }
    let online = value
        .get("online")
        .and_then(Value::as_bool)
        .ok_or(TitanError::InvalidResponse)?;
    let dino = value.get("dino").filter(|value| value.is_object());
    if online && dino.is_none() {
        return Err(TitanError::InvalidResponse);
    }

    let player = dino.map(|dino| SharedPlayer {
        name: optional_string(dino.get("name")),
        dino_name: optional_string(dino.get("species")),
        female: optional_string(dino.get("gender")).and_then(|gender| {
            if gender.eq_ignore_ascii_case("female") {
                Some(true)
            } else if gender.eq_ignore_ascii_case("male") {
                Some(false)
            } else {
                None
            }
        }),
        growth_pct: fraction_percent(dino.get("growth")),
        health: health_bar(dino),
        stamina: percent_bar(dino, "stamina"),
        hunger: percent_bar(dino, "hunger"),
        thirst: percent_bar(dino, "thirst"),
        mutations: dino
            .get("mutations")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| optional_string(Some(item)))
                    .collect()
            })
            .unwrap_or_default(),
        nutrition: None,
        prime_quests: prime_quests(dino),
    });

    // Titan's public `IsleMap.toPct(loc)` defines loc[0] as the HORIZONTAL
    // map axis and loc[1] as the VERTICAL axis. overlay-core stores world
    // coordinates as (vertical game X, horizontal game Y), so these two API
    // values must be swapped at the provider boundary, just like Era's
    // location object. Keeping the conversion here makes every map renderer,
    // waypoint bearing and trail consume one canonical coordinate order.
    let position_cm = if online {
        dino.and_then(|dino| dino.get("loc"))
            .and_then(Value::as_array)
            .filter(|values| values.len() >= 2)
            .and_then(|values| titan_loc_to_world(values))
    } else {
        None
    };
    // Titan's live HUD uses camera yaw when present and falls back to body
    // yaw. Both use screen-right/east as zero, hence the +90 conversion.
    let heading_deg = if online { dino } else { None }
        .and_then(|dino| number(dino.get("cam")).or_else(|| number(dino.get("yaw"))))
        .and_then(map_yaw_to_bearing_deg);
    let friends = shared_friends(value, online);

    Ok(ProviderSnapshot {
        provider: ProviderId::Titan,
        status: if online {
            ConnectionStatus::AuthenticatedOnline
        } else {
            ConnectionStatus::AuthenticatedOffline
        },
        server_id: None,
        server_name: optional_string(value.get("server")),
        received_at_ms,
        source_timestamp_ms: number(value.get("chot_ts")).map(|value| (value * 1000.0) as i64),
        player,
        friends,
        position_cm,
        heading_deg,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitanLiveSample {
    pub position_cm: (f64, f64, f64),
    pub camera_heading_deg: Option<f64>,
    pub body_heading_deg: Option<f64>,
}

fn titan_loc_to_world(values: &[Value]) -> Option<(f64, f64, f64)> {
    let horizontal_y = number(values.first())?;
    let vertical_x = number(values.get(1))?;
    let z = values
        .get(2)
        .and_then(|value| number(Some(value)))
        .unwrap_or(0.0);
    Some((vertical_x, horizontal_y, z))
}

/// Choose the view direction exactly as Titan's camera-follow HUD does.
/// Once a camera angle has arrived, a sparse packet must keep that angle
/// instead of making the arrow jump to the dinosaur body's yaw.
pub fn preferred_live_heading(
    last_camera_heading: &mut Option<(f64, i64)>,
    sample: &TitanLiveSample,
    now_ms: i64,
) -> Option<f64> {
    if let Some(camera) = sample.camera_heading_deg {
        *last_camera_heading = Some((camera, now_ms));
    }
    // Sparse packets can omit cam briefly, but a dead character's camera
    // must not be promoted to a fresh heading by every subsequent body packet.
    if let Some((camera, at_ms)) = *last_camera_heading {
        if (0..=1_000).contains(&now_ms.saturating_sub(at_ms)) {
            return Some(camera);
        }
        *last_camera_heading = None;
    }
    sample.body_heading_deg
}

/// Parse one `data:` payload from Titan's official overlay SSE stream. The
/// server sends roughly 20 samples per second as `{loc, yaw, cam}`.
pub fn parse_stream_sample(data: &str) -> Option<TitanLiveSample> {
    let value: Value = serde_json::from_str(data).ok()?;
    let loc = value.get("loc")?.as_array()?;
    if loc.len() < 3 {
        return None;
    }
    let position_cm = titan_loc_to_world(loc)?;
    let camera_heading_deg = number(value.get("cam")).and_then(map_yaw_to_bearing_deg);
    let body_heading_deg = number(value.get("yaw")).and_then(map_yaw_to_bearing_deg);
    Some(TitanLiveSample {
        position_cm,
        camera_heading_deg,
        body_heading_deg,
    })
}

fn exact_origin(origin: &str) -> Result<Url, TitanError> {
    let url = Url::parse(origin).map_err(|_| TitanError::InvalidOrigin)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || !matches!(
            url.host_str().map(str::to_ascii_lowercase).as_deref(),
            Some("therealservervn.com" | "www.therealservervn.com")
        )
    {
        return Err(TitanError::InvalidOrigin);
    }
    let mut clean = url;
    clean.set_path("/");
    clean.set_query(None);
    clean.set_fragment(None);
    Ok(clean)
}

fn client() -> Result<Client, TitanError> {
    Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| TitanError::Temporary)
}

fn get(client: &Client, url: Url, cookie: &str) -> Result<Response, TitanError> {
    let cookie = HeaderValue::from_str(cookie).map_err(|_| TitanError::LoginRequired)?;
    client
        .get(url)
        .header(ACCEPT, "text/html,application/json")
        .header(COOKIE, cookie)
        .send()
        .map_err(|_| TitanError::Temporary)
}

fn response_text(response: Response) -> Result<String, TitanError> {
    if response.status() == StatusCode::UNAUTHORIZED || response.status().is_redirection() {
        return Err(TitanError::LoginRequired);
    }
    if response.status().is_server_error() {
        return Err(TitanError::Temporary);
    }
    if !response.status().is_success() {
        return Err(TitanError::InvalidResponse);
    }
    response.text().map_err(|_| TitanError::InvalidResponse)
}

pub fn validate(cookie: &str, origin: &str) -> Result<Option<String>, TitanError> {
    let mut url = exact_origin(origin)?;
    url.set_path("/nguoi-choi");
    let body = response_text(get(&client()?, url, cookie)?)?;
    match discover_server_id(&body) {
        Ok(server_id) => Ok(Some(server_id)),
        Err(_) if body.contains("type=\"password\"") || body.contains("action=\"/login") => {
            Err(TitanError::LoginRequired)
        }
        Err(_) => Ok(None),
    }
}

fn feature_json_get(cookie: &str, origin: &str, path: &str) -> Result<Value, String> {
    let mut url = exact_origin(origin).map_err(|error| error.to_string())?;
    url.set_path(path);
    let response = get(&client().map_err(|error| error.to_string())?, url, cookie)
        .map_err(|error| error.to_string())?;
    let body = response_text(response).map_err(|error| error.to_string())?;
    serde_json::from_str(&body).map_err(|_| "Dữ liệu Titan không hợp lệ.".to_string())
}

fn feature_json_post(
    cookie: &str,
    origin: &str,
    path: &str,
    body: &Value,
) -> Result<Value, String> {
    let mut url = exact_origin(origin).map_err(|error| error.to_string())?;
    url.set_path(path);
    let cookie =
        HeaderValue::from_str(cookie).map_err(|_| TitanError::LoginRequired.to_string())?;
    let response = client()
        .map_err(|error| error.to_string())?
        .post(url)
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/json")
        .header(COOKIE, cookie)
        .body(serde_json::to_string(body).map_err(|_| "Dữ liệu Titan không hợp lệ.".to_string())?)
        .send()
        .map_err(|_| TitanError::Temporary.to_string())?;
    if response.status() == StatusCode::UNAUTHORIZED || response.status().is_redirection() {
        return Err(TitanError::LoginRequired.to_string());
    }
    let text = response
        .text()
        .map_err(|_| "Dữ liệu Titan không hợp lệ.".to_string())?;
    serde_json::from_str(&text).map_err(|_| "Dữ liệu Titan không hợp lệ.".to_string())
}

pub fn garage_fetch(cookie: &str, origin: &str) -> Result<Value, String> {
    feature_json_get(cookie, origin, "/api/nguoi-choi/garage")
}

pub fn garage_action(
    cookie: &str,
    origin: &str,
    action: &str,
    slot: Option<usize>,
) -> Result<Value, String> {
    let path = match action {
        "park" => "/api/nguoi-choi/garage/store",
        "restore" => "/api/nguoi-choi/garage/load",
        "delete" => "/api/nguoi-choi/garage/delete",
        "cancel" => "/api/nguoi-choi/garage/cancel",
        _ => return Err("Thao tác Garage Titan không hợp lệ.".to_string()),
    };
    let body = if action == "cancel" {
        serde_json::json!({})
    } else {
        let slot = slot
            .filter(|slot| *slot < 5)
            .ok_or_else(|| "Slot Garage Titan phải nằm trong khoảng 1 đến 5.".to_string())?;
        serde_json::json!({ "slot": slot })
    };
    feature_json_post(cookie, origin, path, &body)
}

pub fn skin_state(cookie: &str, origin: &str) -> Result<Value, String> {
    feature_json_get(cookie, origin, "/api/nguoi-choi/skin")
}

pub fn skin_apply(
    cookie: &str,
    origin: &str,
    server_id: &str,
    colors: &[String],
    variation: f64,
) -> Result<Value, String> {
    const FIELDS: [&str; 7] = [
        "MaleDisplayColor",
        "MarkingsColor",
        "BodyColor",
        "FlankColor",
        "UnderbellyColor",
        "Detail1Color",
        "EyesColor",
    ];
    if colors.len() != FIELDS.len() || colors.iter().any(|color| !valid_hex_color(color)) {
        return Err("Bảng màu Titan phải có đúng 7 mã #RRGGBB.".to_string());
    }
    if server_id.trim().is_empty() || server_id.len() > 512 {
        return Err(TitanError::MissingServer.to_string());
    }
    if !variation.is_finite() {
        return Err("Biến thể skin Titan không hợp lệ.".to_string());
    }
    let colors = FIELDS
        .into_iter()
        .zip(colors.iter().cloned())
        .map(|(field, color)| (field.to_string(), Value::String(color)))
        .collect::<serde_json::Map<String, Value>>();
    feature_json_post(
        cookie,
        origin,
        "/api/nguoi-choi/skin/apply",
        &serde_json::json!({
            "sv": server_id,
            "colors": colors,
            "variation": variation.clamp(0.0, 1.0),
        }),
    )
}

fn valid_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

pub fn poll(cookie: &str, origin: &str, server_id: &str) -> Result<ProviderSnapshot, TitanError> {
    if server_id.trim().is_empty() || server_id.len() > 512 {
        return Err(TitanError::MissingServer);
    }
    let mut url = exact_origin(origin)?;
    // Same endpoint as Titan's downloadable HUD. Unlike the slower account
    // panel endpoint, this response includes live yaw/camera fallback data.
    url.set_path("/api/nguoi-choi/overlay");
    url.query_pairs_mut().append_pair("sv", server_id);
    let body = response_text(get(&client()?, url, cookie)?)?;
    let value = serde_json::from_str::<Value>(&body).map_err(|_| TitanError::InvalidResponse)?;
    let mut snapshot = normalize(&value, chrono::Utc::now().timestamp_millis())?;
    snapshot.server_id = Some(server_id.to_string());
    Ok(snapshot)
}

/// Consume Titan's official high-frequency location/camera stream. Returning
/// `None` lets the caller cancel even on heartbeat chunks. A per-read timeout
/// reconnects a silent socket without periodically killing a healthy stream.
pub fn stream<F>(cookie: &str, origin: &str, mut on_sample: F) -> Result<(), TitanError>
where
    F: FnMut(Option<TitanLiveSample>) -> bool,
{
    let mut url = exact_origin(origin)?;
    url.set_path("/api/nguoi-choi/overlay/stream");
    let cookie = HeaderValue::from_str(cookie).map_err(|_| TitanError::LoginRequired)?;
    tauri::async_runtime::block_on(async {
    let mut response = reqwest::Client::builder()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(5))
        .build()
        .map_err(|_| TitanError::Temporary)?
        .get(url)
        .header(ACCEPT, "text/event-stream")
        .header(COOKIE, cookie)
        .send()
        .await
        .map_err(|_| TitanError::Temporary)?;

    if response.status() == StatusCode::UNAUTHORIZED || response.status().is_redirection() {
        return Err(TitanError::LoginRequired);
    }
    if !response.status().is_success() {
        return Err(TitanError::Temporary);
    }

    let mut pending = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| TitanError::Temporary)? {
        if !on_sample(None) { return Ok(()); }
        pending.extend_from_slice(&chunk);
        // Bound a broken server's unterminated event buffer.
        if pending.len() > 128 * 1024 { return Err(TitanError::InvalidResponse); }
        while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
            let line = String::from_utf8_lossy(&pending[..end]);
            let sample = line.strip_prefix("data:").and_then(|data| parse_stream_sample(data.trim()));
            if let Some(sample) = sample {
                if !on_sample(Some(sample)) {
                    return Ok(());
                }
            }
            pending.drain(..=end);
        }
    }
    Err(TitanError::Temporary)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn titan_extracts_server_id_and_full_player_fields() {
        assert_eq!(
            discover_server_id(r#"<div id="dino" data-sv="server-opaque-id"></div>"#).unwrap(),
            "server-opaque-id"
        );
        let value = json!({
            "ok": true,
            "online": true,
            "server": "Synthetic TRS",
            "dino": {
                "species": "Tyrannosaurus",
                "growth": 0.27,
                "stamina": 1.0,
                "hunger": 0.66,
                "thirst": 0.89,
                "health": 0.74,
                "hp": [44.0, 59.0],
                "hp_that": true,
                "mutations": ["SyntheticMutation"],
                "loc": [-238000.0, 85000.0],
                "yaw": -30.0,
                "cam": 45.0,
                "prime_dk": [false, false, false, false, false, false, true, true, true, true]
            },
            "friends": [
                {"n": 2, "name": "Friend Two", "species": "Stegosaurus", "loc": [-120000.0, 340000.0, 800.0]},
                {"n": 4, "name": "Friend Four", "loc": [90000.0, -210000.0, 400.0]}
            ]
        });
        let snapshot = normalize(&value, 1000).unwrap();
        let player = snapshot.player.unwrap();
        assert_eq!(player.growth_pct, Some(27.0));
        assert_eq!(player.mutations, vec!["SyntheticMutation"]);
        assert_eq!(player.prime_quests.len(), 10);
        assert_eq!(
            player.prime_quests[0].text_vi.as_deref(),
            Some("Ghé Sanctuary khi còn nhỏ")
        );
        assert_eq!(
            player.prime_quests[9].text_vi.as_deref(),
            Some("Chơi loài nhỏ (Hypsi · Troodon · Beipi · Dryo · Deino)")
        );
        assert_eq!(snapshot.position_cm, Some((85000.0, -238000.0, 0.0)));
        assert_eq!(
            snapshot.heading_deg,
            Some(135.0),
            "camera yaw must win over body yaw"
        );
        assert_eq!(snapshot.friends.len(), 2);
        assert_eq!(snapshot.friends[0].slot, Some(2));
        assert_eq!(snapshot.friends[0].name, "Friend Two");
        assert_eq!(
            snapshot.friends[0].position_cm,
            Some((340000.0, -120000.0, 800.0)),
            "friend coordinates must use the same canonical axis order as the player"
        );
        assert!(snapshot.friends[0].position_px.is_none());
    }

    #[test]
    fn titan_stream_sample_contains_live_camera_direction() {
        let sample =
            parse_stream_sample(r#"{"ts":1234,"loc":[100.0,200.0,300.0],"yaw":-30.0,"cam":-10.0}"#)
                .expect("valid live sample");
        assert_eq!(sample.position_cm, (200.0, 100.0, 300.0));
        assert_eq!(sample.camera_heading_deg, Some(80.0));
        assert_eq!(sample.body_heading_deg, Some(60.0));
    }

    #[test]
    fn titan_keeps_last_camera_heading_when_a_packet_only_has_body_yaw() {
        let mut last_camera = None;
        let first =
            parse_stream_sample(r#"{"loc":[1.0,2.0,3.0],"yaw":-30.0,"cam":-10.0}"#).unwrap();
        assert_eq!(preferred_live_heading(&mut last_camera, &first, 0), Some(80.0));

        let sparse = parse_stream_sample(r#"{"loc":[1.0,2.0,3.0],"yaw":120.0}"#).unwrap();
        assert_eq!(
            preferred_live_heading(&mut last_camera, &sparse, 500),
            Some(80.0),
            "body yaw must not replace the player's view direction"
        );
        assert_eq!(preferred_live_heading(&mut last_camera, &sparse, 1_001), Some(210.0));
        assert_eq!(last_camera, None, "expired camera must not survive a respawn");
        let fresh = parse_stream_sample(r#"{"loc":[4,5,6],"cam":0}"#).unwrap();
        assert_eq!(preferred_live_heading(&mut last_camera, &fresh, 1_050), Some(90.0));
    }

    #[test]
    fn titan_rejects_missing_server_id_and_invalid_fraction_fields() {
        assert!(discover_server_id(r#"<div id="dino"></div>"#).is_err());
        assert!(discover_server_id(r#"<div id="dino" data-sv=""></div>"#).is_err());

        let value = json!({"ok": true, "online": true, "dino": {
            "growth": 27.0, "health": -0.1, "hp": [44.0]
        }});
        let player = normalize(&value, 1).unwrap().player.unwrap();
        assert_eq!(player.growth_pct, None);
        assert_eq!(player.health, None);
    }

    #[test]
    fn titan_offline_snapshot_never_publishes_cached_position() {
        let value = json!({"ok": true, "online": false, "server": "Synthetic TRS", "dino": {
            "species": "Stegosaurus", "growth": 0.5, "loc": [10.0, 20.0]
        }});
        let snapshot = normalize(&value, 1).unwrap();
        assert_eq!(snapshot.status, ConnectionStatus::AuthenticatedOffline);
        assert!(snapshot.position_cm.is_none());
        assert!(snapshot.friends.is_empty());
        assert_eq!(snapshot.player.unwrap().growth_pct, Some(50.0));
    }

    #[test]
    fn titan_ignores_malformed_prime_and_health_arrays() {
        let value = json!({"ok": true, "online": true, "dino": {
            "health": 0.5, "hp": [25.0, 0.0], "prime_dk": [true, false]
        }});
        let player = normalize(&value, 1).unwrap().player.unwrap();
        assert_eq!(player.health.unwrap().current, None);
        assert!(player.prime_quests.is_empty());
    }

    #[test]
    fn titan_feature_commands_reject_stale_client_schemas_before_network_io() {
        assert!(garage_action("", "", "unknown", None).is_err());
        assert!(garage_action("", "", "park", Some(5)).is_err());
        assert!(skin_apply("", "", "server", &vec!["#112233".to_string(); 6], 0.0).is_err());
        assert!(skin_apply("", "", "server", &vec!["not-a-color".to_string(); 7], 0.0).is_err());
        assert!(skin_apply("", "", "server", &vec!["#112233".to_string(); 7], f64::NAN).is_err());
        assert!(valid_hex_color("#A1b2C3"));
        assert!(!valid_hex_color("112233"));
    }
}
