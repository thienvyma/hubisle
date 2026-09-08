use std::{fmt, time::Duration};

use reqwest::{
    blocking::{Client, Response},
    header::{HeaderValue, ACCEPT, COOKIE},
    redirect::Policy,
    StatusCode, Url,
};
use scraper::{Html, Selector};
use serde_json::Value;

use crate::islepilot::parser::QuestStatus;

use super::model::{ConnectionStatus, ProviderId, ProviderSnapshot, SharedPlayer, SharedStatBar};

const PRIME_EN: [&str; 10] = [
    "Prime condition 1",
    "Prime condition 2",
    "Prime condition 3",
    "Prime condition 4",
    "Prime condition 5",
    "Prime condition 6",
    "Prime condition 7",
    "Prime condition 8",
    "Prime condition 9",
    "Prime condition 10",
];
const PRIME_VI: [&str; 10] = [
    "Điều kiện Prime 1",
    "Điều kiện Prime 2",
    "Điều kiện Prime 3",
    "Điều kiện Prime 4",
    "Điều kiện Prime 5",
    "Điều kiện Prime 6",
    "Điều kiện Prime 7",
    "Điều kiện Prime 8",
    "Điều kiện Prime 9",
    "Điều kiện Prime 10",
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

    let position_cm = if online {
        dino.and_then(|dino| dino.get("loc"))
            .and_then(Value::as_array)
            .filter(|values| values.len() == 2)
            .and_then(
                |values| match (number(values.first()), number(values.get(1))) {
                    (Some(x), Some(y)) => Some((x, y, 0.0)),
                    _ => None,
                },
            )
    } else {
        None
    };

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
        position_cm,
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

pub fn poll(cookie: &str, origin: &str, server_id: &str) -> Result<ProviderSnapshot, TitanError> {
    if server_id.trim().is_empty() || server_id.len() > 512 {
        return Err(TitanError::MissingServer);
    }
    let mut url = exact_origin(origin)?;
    url.set_path("/api/nguoi-choi/dino");
    url.query_pairs_mut().append_pair("sv", server_id);
    let body = response_text(get(&client()?, url, cookie)?)?;
    let value = serde_json::from_str::<Value>(&body).map_err(|_| TitanError::InvalidResponse)?;
    let mut snapshot = normalize(&value, chrono::Utc::now().timestamp_millis())?;
    snapshot.server_id = Some(server_id.to_string());
    Ok(snapshot)
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
                "prime_dk": [false, false, false, false, false, false, true, true, true, true]
            }
        });
        let snapshot = normalize(&value, 1000).unwrap();
        let player = snapshot.player.unwrap();
        assert_eq!(player.growth_pct, Some(27.0));
        assert_eq!(player.mutations, vec!["SyntheticMutation"]);
        assert_eq!(player.prime_quests.len(), 10);
        assert_eq!(snapshot.position_cm, Some((-238000.0, 85000.0, 0.0)));
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
}
