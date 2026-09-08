use std::{fmt, time::Duration};

use reqwest::{
    blocking::Client,
    header::{HeaderValue, ACCEPT, COOKIE},
    redirect::Policy,
    StatusCode,
};
use serde_json::Value;

use super::model::{ConnectionStatus, ProviderId, ProviderSnapshot, SharedPlayer, SharedStatBar};

const ENDPOINT: &str = "https://eragamingvn.net/api/theisle/map";

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

    let (player, position_cm) = if online {
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
            prime_quests: Vec::new(),
        };
        (Some(player), position)
    } else {
        (None, None)
    };

    let server_name = optional_string(value.get("server"))
        .or_else(|| optional_string(value.get("server").and_then(|server| server.get("name"))));
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
        source_timestamp_ms: number(value.get("timestamp")).map(|value| value as i64),
        player,
        position_cm,
    })
}

fn client() -> Result<Client, EraError> {
    Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| EraError::Temporary)
}

pub fn poll(cookie: &str) -> Result<ProviderSnapshot, EraError> {
    let cookie = HeaderValue::from_str(cookie).map_err(|_| EraError::LoginRequired)?;
    let response = client()?
        .get(ENDPOINT)
        .header(ACCEPT, "application/json")
        .header(COOKIE, cookie)
        .send()
        .map_err(|_| EraError::Temporary)?;

    if response.status() == StatusCode::UNAUTHORIZED || response.status().is_redirection() {
        return Err(EraError::LoginRequired);
    }
    if response.status().is_server_error() {
        return Err(EraError::Temporary);
    }
    if !response.status().is_success() {
        return Err(EraError::InvalidResponse);
    }
    let body = response.text().map_err(|_| EraError::InvalidResponse)?;
    let value = serde_json::from_str::<Value>(&body).map_err(|_| EraError::InvalidResponse)?;
    normalize(&value, chrono::Utc::now().timestamp_millis())
}

pub fn validate(cookie: &str) -> Result<ProviderSnapshot, EraError> {
    poll(cookie)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
            }
        });

        let snapshot = normalize(&value, 1000).unwrap();
        assert_eq!(snapshot.position_cm, Some((317900.0, 45100.0, 20900.0)));
        let health = snapshot.player.unwrap().health.unwrap();
        assert_eq!(health.percent, 0.0);
        assert_eq!(health.current, None);
        assert_eq!(health.max, None);
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
}
