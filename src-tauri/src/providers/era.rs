use std::{fmt, sync::LazyLock, time::Duration};

use reqwest::{
    blocking::{Client, RequestBuilder},
    header::{HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE, COOKIE, ORIGIN, REFERER},
    redirect::Policy,
    StatusCode,
};
use serde_json::Value;

use overlay_core::map_yaw_to_bearing_deg;

use crate::{combat::CombatEvent, islepilot::parser::QuestStatus};

use super::model::{
    ConnectionStatus, ProviderId, ProviderSnapshot, SharedFriend, SharedPlayer, SharedStatBar,
};

const ENDPOINT: &str = "https://eragamingvn.net/api/theisle/map";
const GARAGE_ENDPOINT: &str = "https://eragamingvn.net/api/theisle/garage";
const SKIN_ENDPOINT: &str = "https://eragamingvn.net/api/theisle/skin";
const PRIME_EN: [&str; 10] = [
    "Visit a Sanctuary while juvenile",
    "Be born from a nest",
    "Achieve a perfect diet (at least 1% of each nutrient)",
    "Visit a Mass Migration zone",
    "Visit 2 Migration zones",
    "Visit 4 Patrol zones",
    "Never become Infertile",
    "Never get Muscle Spasms",
    "Raise offspring to Subadult",
    "Play as Hypsi, Troodon, Beipi, Dryo, or Deino",
];
const PRIME_VI: [&str; 10] = [
    "Ghé Khu bảo tồn (Sanctuary) khi còn non",
    "Được sinh ra từ tổ (nest)",
    "Đạt chế độ ăn hoàn hảo (mỗi loại ít nhất 1%)",
    "Ghé khu Bãi di cư (Mass Migration)",
    "Ghé 2 khu Di cư (Migration)",
    "Ghé 4 khu Tuần tra (Patrol)",
    "Không bao giờ bị Vô sinh (Infertile)",
    "Không bao giờ bị Co thắt cơ (Muscle Spasms)",
    "Nuôi con đến Subadult",
    "Chơi Hypsi, Troodon, Beipi, Dryo hoặc Deino",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EraError {
    LoginRequired,
    Temporary,
    InvalidResponse,
}

impl fmt::Display for EraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::LoginRequired => "Cần đăng nhập Era Gaming VN.",
            Self::Temporary => "Tạm thời không thể kết nối Era Gaming VN.",
            Self::InvalidResponse => "Dữ liệu Era Gaming VN không hợp lệ.",
        };
        f.write_str(message)
    }
}

fn number(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn percent(value: Option<&Value>) -> Option<f64> {
    number(value).filter(|value| (0.0..=100.0).contains(value))
}

fn exact_bar(
    percent: Option<f64>,
    current: Option<f64>,
    max: Option<f64>,
) -> Option<SharedStatBar> {
    let pct = percent.filter(|value| value.is_finite() && (0.0..=100.0).contains(value))?;
    match (current, max) {
        (Some(cur), Some(cap))
            if cur.is_finite()
                && cap.is_finite()
                && cur >= 0.0
                && cap > 0.0
                && ((cur / cap * 100.0) - pct).abs() <= 5.0 =>
        {
            Some(SharedStatBar::from_values(cur, cap))
        }
        _ => Some(SharedStatBar::from_percent(pct)),
    }
}

fn stat(player: &Value, name: &str, percent_key: &str) -> Option<SharedStatBar> {
    let exact = player.get("exactVitals").and_then(|value| value.get(name));
    exact_bar(
        percent(player.get(percent_key)),
        number(exact.and_then(|value| value.get("current"))),
        number(exact.and_then(|value| value.get("max"))),
    )
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn source_timestamp_ms(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(number) => number.as_i64(),
        Value::String(value) => chrono::DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|timestamp| timestamp.timestamp_millis()),
        _ => None,
    }
}

fn prime_quests(player: &Value) -> Vec<QuestStatus> {
    let Some(prime) = player.get("prime").and_then(Value::as_object) else {
        return Vec::new();
    };
    if prime.get("available").and_then(Value::as_bool) != Some(true) {
        return Vec::new();
    }

    let conditions = prime.get("conditions").and_then(Value::as_array);
    let completed = (1..=10)
        .map(|id| {
            conditions
                .and_then(|items| {
                    items
                        .iter()
                        .find(|item| item.get("id").and_then(Value::as_u64) == Some(id as u64))
                })
                .and_then(|item| item.get("complete"))
                .and_then(Value::as_bool)
                .or_else(|| prime.get(&format!("cond{id}")).and_then(Value::as_bool))
        })
        .collect::<Option<Vec<_>>>();
    let Some(completed) = completed else {
        return Vec::new();
    };

    completed
        .into_iter()
        .enumerate()
        .map(|(index, completed)| QuestStatus {
            text: PRIME_EN[index].to_string(),
            text_vi: Some(PRIME_VI[index].to_string()),
            completed,
        })
        .collect()
}

fn shared_friends(value: &Value) -> Vec<SharedFriend> {
    value
        .get("friends")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|friend| {
            let name = optional_string(friend.get("name"))?;
            let online = friend.get("online").and_then(Value::as_bool) == Some(true);
            let location = friend.get("location");
            // Era's RCON object uses x for the horizontal HUD coordinate and
            // y for the vertical coordinate. Convert to the overlay's
            // canonical (vertical game X, horizontal game Y) ordering.
            let position_cm = if online {
                match (
                    number(location.and_then(|item| item.get("x"))),
                    number(location.and_then(|item| item.get("y"))),
                    number(location.and_then(|item| item.get("z"))),
                ) {
                    (Some(x), Some(y), z) => Some((y, x, z.unwrap_or(0.0))),
                    _ => None,
                }
            } else {
                None
            };
            Some(SharedFriend {
                slot: None,
                name,
                dino_name: optional_string(friend.get("class")),
                online: position_cm.is_some(),
                position_cm,
                position_px: None,
            })
        })
        .collect()
}

pub fn normalize(value: &Value, received_at_ms: i64) -> Result<ProviderSnapshot, EraError> {
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        return Err(EraError::InvalidResponse);
    }

    let server_online = value
        .get("serverOnline")
        .and_then(Value::as_bool)
        .ok_or(EraError::InvalidResponse)?;
    let player_online = value
        .get("playerOnline")
        .and_then(Value::as_bool)
        .ok_or(EraError::InvalidResponse)?;
    let online = server_online && player_online;

    let (player, position_cm, heading_deg) = if online {
        let source = value
            .get("player")
            .and_then(Value::as_object)
            .ok_or(EraError::InvalidResponse)?;
        let source = Value::Object(source.clone());
        let location = source.get("location");
        let position = match (
            number(location.and_then(|item| item.get("x"))),
            number(location.and_then(|item| item.get("y"))),
            number(location.and_then(|item| item.get("z"))),
        ) {
            (Some(x), Some(y), Some(z)) => Some((y, x, z)),
            _ => None,
        };
        // Era now exposes viewYaw; older bridge versions use camera/yaw aliases.
        // Missing/null view data must keep the movement fallback available.
        let heading = ["viewYaw", "cam", "cameraYaw", "camera_yaw", "yaw"]
            .into_iter()
            .find_map(|key| number(source.get(key)))
            .or_else(|| {
                source
                    .get("rotation")
                    .and_then(|rotation| number(rotation.get("yaw")))
            })
            .and_then(map_yaw_to_bearing_deg);
        let player = SharedPlayer {
            name: optional_string(source.get("name")),
            dino_name: optional_string(source.get("class")),
            female: None,
            growth_pct: percent(source.get("growthPercent")),
            health: stat(&source, "health", "healthPercent"),
            stamina: stat(&source, "stamina", "staminaPercent"),
            hunger: stat(&source, "hunger", "hungerPercent"),
            thirst: stat(&source, "thirst", "thirstPercent"),
            mutations: Vec::new(),
            nutrition: None,
            prime_quests: prime_quests(&source),
        };
        (Some(player), position, heading)
    } else {
        (None, None, None)
    };

    let server_name = optional_string(value.get("server"))
        .or_else(|| optional_string(value.get("server").and_then(|server| server.get("name"))));
    let friends = shared_friends(value);
    Ok(ProviderSnapshot {
        provider: ProviderId::Era,
        status: if online {
            ConnectionStatus::AuthenticatedOnline
        } else {
            ConnectionStatus::AuthenticatedOffline
        },
        server_id: optional_string(value.get("serverId")),
        server_name,
        received_at_ms,
        source_timestamp_ms: source_timestamp_ms(
            value.get("updatedAt").or_else(|| value.get("timestamp")),
        ),
        player,
        friends,
        position_cm,
        heading_deg,
    })
}

static CLIENT: LazyLock<Result<Client, EraError>> = LazyLock::new(|| {
    Client::builder()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(35))
        .user_agent(concat!("islemap-thienvyma/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| EraError::Temporary)
});

fn client() -> Result<Client, EraError> {
    CLIENT.clone()
}

/// Match the official map's 12s refresh cadence and back off during outages.
pub fn retry_delay_secs(failures: u32) -> u64 {
    match failures {
        0 => 12,
        1 => 20,
        2 => 40,
        _ => 60,
    }
}

pub fn may_retain_snapshot(last_received_at_ms: Option<i64>, now_ms: i64) -> bool {
    last_received_at_ms.is_some_and(|last| (0..=90_000).contains(&now_ms.saturating_sub(last)))
}

fn poll_value(cookie: &str) -> Result<(Value, i64), EraError> {
    let cookie = HeaderValue::from_str(cookie).map_err(|_| EraError::LoginRequired)?;
    let response = client()?
        .get(ENDPOINT)
        .header(ACCEPT, "application/json")
        .header(COOKIE, cookie)
        .header(ORIGIN, "https://eragamingvn.net")
        .header(REFERER, "https://eragamingvn.net/live-map")
        .header(reqwest::header::CACHE_CONTROL, "no-cache")
        .send()
        .map_err(|error| {
            log::warn!(
                "Era map request failed (timeout={}, connect={})",
                error.is_timeout(),
                error.is_connect()
            );
            EraError::Temporary
        })?;

    if response.status() == StatusCode::UNAUTHORIZED || response.status().is_redirection() {
        return Err(EraError::LoginRequired);
    }
    if response.status().is_server_error()
        || matches!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS | StatusCode::REQUEST_TIMEOUT
        )
    {
        log::warn!(
            "Era map temporarily unavailable (HTTP {})",
            response.status().as_u16()
        );
        return Err(EraError::Temporary);
    }
    if !response.status().is_success() {
        return Err(EraError::InvalidResponse);
    }
    let body = response.text().map_err(|_| EraError::InvalidResponse)?;
    let value = serde_json::from_str::<Value>(&body).map_err(|_| EraError::InvalidResponse)?;
    Ok((value, chrono::Utc::now().timestamp_millis()))
}

pub fn poll_with_events(cookie: &str) -> Result<(ProviderSnapshot, Vec<CombatEvent>), EraError> {
    let (value, received_at_ms) = poll_value(cookie)?;
    let snapshot = normalize(&value, received_at_ms).map_err(|error| {
        log::warn!("Era map returned an unrecognized snapshot schema");
        error
    })?;
    let events = crate::combat::parse_era_events(
        &value,
        snapshot.source_timestamp_ms.unwrap_or(received_at_ms),
        snapshot.server_name.as_deref(),
        snapshot
            .player
            .as_ref()
            .and_then(|player| player.dino_name.as_deref()),
    );
    Ok((snapshot, events))
}

pub fn poll(cookie: &str) -> Result<ProviderSnapshot, EraError> {
    poll_with_events(cookie).map(|(snapshot, _)| snapshot)
}

pub fn validate(cookie: &str) -> Result<ProviderSnapshot, EraError> {
    poll(cookie)
}

fn authenticated_feature_request(request: RequestBuilder, cookie: HeaderValue) -> RequestBuilder {
    // Era's live-map JavaScript sends Garage/Skin mutations from this exact
    // same-origin page. The server validates that browser context for writes;
    // a raw Cookie header alone can be rejected as unauthenticated, especially
    // for the two VIP Garage slots.
    request
        .header(ACCEPT, "application/json")
        .header(COOKIE, cookie)
        .header(ORIGIN, "https://eragamingvn.net")
        .header(REFERER, "https://eragamingvn.net/live-map")
}

fn feature_response(request: RequestBuilder, cookie: &str) -> Result<(StatusCode, Value), String> {
    let cookie = HeaderValue::from_str(cookie).map_err(|_| EraError::LoginRequired.to_string())?;
    let response = authenticated_feature_request(request, cookie)
        .send()
        .map_err(|_| EraError::Temporary.to_string())?;
    let status = response.status();
    if status == StatusCode::UNAUTHORIZED || status.is_redirection() {
        return Err(EraError::LoginRequired.to_string());
    }
    let text = response
        .text()
        .map_err(|_| "Dữ liệu Era không hợp lệ.".to_string())?;
    let value = serde_json::from_str(&text).map_err(|_| "Dữ liệu Era không hợp lệ.".to_string())?;
    Ok((status, value))
}

pub fn garage_fetch(cookie: &str) -> Result<Value, String> {
    let request = client()
        .map_err(|error| error.to_string())?
        .get(GARAGE_ENDPOINT)
        .header(HeaderName::from_static("x-era-slot"), "1");
    feature_response(request, cookie).map(|(_, value)| value)
}

pub fn garage_action(
    cookie: &str,
    action: &str,
    slot: usize,
    state_hash: Option<&str>,
) -> Result<Value, String> {
    if !matches!(action, "park" | "restore" | "delete") || !(1..=5).contains(&slot) {
        return Err("Thao tác Garage Era không hợp lệ.".to_string());
    }
    let client = client().map_err(|error| error.to_string())?;
    let mut request = client
        .post(GARAGE_ENDPOINT)
        .header(HeaderName::from_static("x-era-slot"), slot.to_string())
        .header(HeaderName::from_static("x-era-action"), action)
        .header(
            HeaderName::from_static("x-era-operation-id"),
            uuid::Uuid::new_v4().simple().to_string(),
        );
    if action == "delete" {
        let hash = state_hash
            .filter(|hash| hash.len() == 64 && hash.as_bytes().iter().all(u8::is_ascii_hexdigit))
            .ok_or_else(|| "Thiếu mã xác nhận Dino trong slot Era.".to_string())?;
        request = request
            .header(
                HeaderName::from_static("x-era-confirm-delete"),
                format!("slot:{slot}"),
            )
            .header(HeaderName::from_static("x-era-state-hash"), hash);
    }
    let (_, mut value) = feature_response(request, cookie)?;
    let Some(job_id) = value
        .get("jobId")
        .and_then(Value::as_str)
        .filter(|_| value.get("queued").and_then(Value::as_bool) == Some(true))
        .map(str::to_string)
    else {
        return Ok(value);
    };
    for _ in 0..120 {
        std::thread::sleep(Duration::from_millis(500));
        let request = client
            .get(GARAGE_ENDPOINT)
            .header(HeaderName::from_static("x-era-slot"), slot.to_string())
            .header(HeaderName::from_static("x-era-job-id"), &job_id);
        let (_, next) = feature_response(request, cookie)?;
        value = next;
        if !matches!(
            value.get("state").and_then(Value::as_str),
            Some("queued" | "running")
        ) {
            return Ok(value);
        }
    }
    Err("Garage Era xử lý quá lâu; hãy làm mới để kiểm tra trạng thái.".to_string())
}

pub fn skin_state(cookie: &str) -> Result<Value, String> {
    feature_response(
        client()
            .map_err(|error| error.to_string())?
            .get(SKIN_ENDPOINT),
        cookie,
    )
    .map(|(_, value)| value)
}

pub fn skin_apply(cookie: &str, colors: &[String]) -> Result<Value, String> {
    // Era's official live-map editor currently exposes seven color zones and
    // serializes them as color1..color7. Keep this strict so a stale UI cannot
    // send an ambiguous partial palette after the provider changes its schema.
    if colors.len() != 7 || colors.iter().any(|color| !valid_hex_color(color)) {
        return Err("Bảng màu Era phải có đúng 7 mã #RRGGBB.".to_string());
    }
    let body = colors
        .iter()
        .enumerate()
        .map(|(index, color)| (format!("color{}", index + 1), Value::String(color.clone())))
        .collect::<serde_json::Map<String, Value>>();
    let client = client().map_err(|error| error.to_string())?;
    let (_, queued) = feature_response(
        client
            .post(SKIN_ENDPOINT)
            .header(HeaderName::from_static("x-era-action"), "skin")
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_string(&body)
                    .map_err(|_| "Dữ liệu đổi skin Era không hợp lệ.".to_string())?,
            ),
        cookie,
    )?;
    let Some(operation_id) = queued
        .get("operationId")
        .and_then(Value::as_str)
        .map(str::to_string)
    else {
        return Ok(queued);
    };
    for _ in 0..18 {
        std::thread::sleep(Duration::from_secs(1));
        let (_, result) = feature_response(
            client
                .get(SKIN_ENDPOINT)
                .header(HeaderName::from_static("x-era-operation"), &operation_id),
            cookie,
        )?;
        if result.get("state").and_then(Value::as_str) != Some("queued") {
            return Ok(result);
        }
    }
    Err("Chưa nhận được kết quả đổi skin Era; hãy kiểm tra lại sau.".to_string())
}

fn valid_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn era_accepts_new_view_yaw_and_null_prime_without_losing_player() {
        let mut value = json!({
            "success": true, "serverOnline": true, "playerOnline": true,
            "player": {
                "location": {"x": 10.0, "y": 20.0, "z": 30.0},
                "viewYaw": -20.0, "cameraYaw": 40.0, "prime": null
            }
        });
        let snapshot = normalize(&value, 1).unwrap();
        assert_eq!(snapshot.status, ConnectionStatus::AuthenticatedOnline);
        assert_eq!(snapshot.heading_deg, Some(70.0));
        assert_eq!(snapshot.position_cm, Some((20.0, 10.0, 30.0)));
        assert!(snapshot.player.unwrap().prime_quests.is_empty());
        value["player"]["viewYaw"] = Value::Null;
        assert_eq!(normalize(&value, 2).unwrap().heading_deg, Some(130.0));
        value["player"]["cameraYaw"] = Value::Null;
        assert_eq!(normalize(&value, 3).unwrap().heading_deg, None);
        value["player"]["viewYaw"] = json!(0.0);
        assert_eq!(normalize(&value, 4).unwrap().heading_deg, Some(90.0));
    }

    #[test]
    fn era_backoff_is_bounded_and_retained_data_expires() {
        assert_eq!(
            (0..5).map(retry_delay_secs).collect::<Vec<_>>(),
            vec![12, 20, 40, 60, 60]
        );
        assert_eq!(retry_delay_secs(u32::MAX), 60);
        assert!(may_retain_snapshot(Some(1000), 91_000));
        assert!(!may_retain_snapshot(Some(1000), 91_001));
        assert!(!may_retain_snapshot(None, 1000));
        assert!(!may_retain_snapshot(Some(2000), 1000));
    }

    #[test]
    fn era_swaps_axes_and_preserves_percent_only_vitals() {
        let value = json!({
            "success": true,
            "serverOnline": true,
            "playerOnline": true,
            "player": {
                "class": "Stegosaurus",
                "location": {"x": 45100.0, "y": 317900.0, "z": 20900.0},
                "growthPercent": 54.0,
                "healthPercent": 0.0,
                "staminaPercent": 96.0
            },
            "friends": [
                {"online": true, "name": "Era Friend", "class": "Diabloceratops", "location": {"x": -120000.0, "y": 340000.0, "z": 800.0}},
                {"online": false, "name": "Offline Friend", "location": null}
            ]
        });

        let snapshot = normalize(&value, 1000).unwrap();
        assert_eq!(snapshot.position_cm, Some((317900.0, 45100.0, 20900.0)));
        let health = snapshot.player.unwrap().health.unwrap();
        assert_eq!(health.percent, 0.0);
        assert_eq!(health.current, None);
        assert_eq!(health.max, None);
        assert_eq!(snapshot.friends.len(), 2);
        assert_eq!(snapshot.friends[0].name, "Era Friend");
        assert_eq!(
            snapshot.friends[0].dino_name.as_deref(),
            Some("Diabloceratops")
        );
        assert_eq!(
            snapshot.friends[0].position_cm,
            Some((340000.0, -120000.0, 800.0))
        );
        assert!(!snapshot.friends[1].online);
        assert!(snapshot.friends[1].position_cm.is_none());
    }

    #[test]
    fn era_uses_camera_yaw_when_the_server_bridge_exposes_it() {
        let value = json!({
            "success": true,
            "serverOnline": true,
            "playerOnline": true,
            "player": {
                "location": {"x": 45100.0, "y": 317900.0, "z": 20900.0},
                "cameraYaw": -20.0,
                "yaw": 110.0
            }
        });
        assert_eq!(normalize(&value, 1).unwrap().heading_deg, Some(70.0));
    }

    #[test]
    fn era_clears_player_when_authenticated_but_offline() {
        let value = json!({
            "success": true,
            "serverOnline": true,
            "playerOnline": false,
            "player": null
        });
        let snapshot = normalize(&value, 1000).unwrap();
        assert_eq!(snapshot.status, ConnectionStatus::AuthenticatedOffline);
        assert!(snapshot.player.is_none());
        assert!(snapshot.position_cm.is_none());
    }

    #[test]
    fn era_accepts_consistent_exact_vitals_only() {
        let consistent = json!({
            "success": true, "serverOnline": true, "playerOnline": true,
            "player": {"healthPercent": 50.0, "exactVitals": {"health": {"current": 25.0, "max": 50.0}}}
        });
        let exact = normalize(&consistent, 1)
            .unwrap()
            .player
            .unwrap()
            .health
            .unwrap();
        assert_eq!((exact.current, exact.max), (Some(25.0), Some(50.0)));

        let inconsistent = json!({
            "success": true, "serverOnline": true, "playerOnline": true,
            "player": {"healthPercent": 90.0, "exactVitals": {"health": {"current": 25.0, "max": 50.0}}}
        });
        let percent_only = normalize(&inconsistent, 1)
            .unwrap()
            .player
            .unwrap()
            .health
            .unwrap();
        assert_eq!((percent_only.current, percent_only.max), (None, None));
        assert_eq!(percent_only.percent, 90.0);
    }

    #[test]
    fn era_rejects_malformed_success_and_ignores_invalid_coordinates() {
        assert!(normalize(&json!({"success": false}), 1).is_err());
        assert!(normalize(&json!({}), 1).is_err());

        let value = json!({
            "success": true, "serverOnline": true, "playerOnline": true,
            "player": {"location": {"x": null, "y": 20.0, "z": 30.0}}
        });
        assert!(normalize(&value, 1).unwrap().position_cm.is_none());
    }

    #[test]
    fn era_maps_live_prime_conditions_and_source_timestamp() {
        let value = json!({
            "success": true,
            "serverOnline": true,
            "playerOnline": true,
            "updatedAt": "2026-09-08T15:03:55.1860905+00:00",
            "player": {
                "class": "Stegosaurus",
                "location": {"x": 45100.0, "y": 317900.0, "z": 20900.0},
                "prime": {
                    "available": true,
                    "completed": 6,
                    "total": 10,
                    "conditions": [
                        {"id": 1, "complete": true},
                        {"id": 2, "complete": true},
                        {"id": 3, "complete": true},
                        {"id": 4, "complete": false},
                        {"id": 5, "complete": true},
                        {"id": 6, "complete": false},
                        {"id": 7, "complete": true},
                        {"id": 8, "complete": true},
                        {"id": 9, "complete": false},
                        {"id": 10, "complete": false}
                    ]
                }
            }
        });

        let snapshot = normalize(&value, 1).unwrap();
        assert_eq!(snapshot.source_timestamp_ms, Some(1_788_879_835_186));
        let quests = snapshot.player.unwrap().prime_quests;
        assert_eq!(quests.len(), 10);
        assert_eq!(quests.iter().filter(|quest| quest.completed).count(), 6);
        assert!(quests[0].text.contains("Sanctuary"));
        assert!(quests[0]
            .text_vi
            .as_deref()
            .unwrap()
            .contains("Khu bảo tồn"));
    }

    #[test]
    fn era_does_not_fabricate_prime_when_provider_marks_it_unavailable() {
        let value = json!({
            "success": true,
            "serverOnline": true,
            "playerOnline": true,
            "player": {"prime": {"available": false, "conditions": []}}
        });

        assert!(normalize(&value, 1)
            .unwrap()
            .player
            .unwrap()
            .prime_quests
            .is_empty());
    }

    #[test]
    fn era_feature_commands_reject_stale_client_schemas_before_network_io() {
        assert!(garage_action("", "unknown", 1, None).is_err());
        assert!(garage_action("", "park", 0, None).is_err());
        assert!(garage_action("", "delete", 1, Some("short")).is_err());
        assert!(skin_apply("", &vec!["#112233".to_string(); 6]).is_err());
        assert!(skin_apply("", &vec!["not-a-color".to_string(); 7]).is_err());
        assert!(valid_hex_color("#A1b2C3"));
        assert!(!valid_hex_color("#12345"));
    }

    #[test]
    fn era_mutation_requests_carry_the_same_origin_context_as_the_live_map() {
        let request = authenticated_feature_request(
            Client::new().post(GARAGE_ENDPOINT),
            HeaderValue::from_static("session=test"),
        )
        .build()
        .unwrap();

        assert_eq!(
            request.headers().get(ORIGIN).unwrap(),
            "https://eragamingvn.net"
        );
        assert_eq!(
            request.headers().get(REFERER).unwrap(),
            "https://eragamingvn.net/live-map"
        );
        assert_eq!(request.headers().get(COOKIE).unwrap(), "session=test");
    }
}
