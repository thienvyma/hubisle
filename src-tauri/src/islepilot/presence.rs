//! Optional hub-to-hub friend-position relay.
//!
//! Some IslePilot servers expose the authenticated player's own position but
//! disable their server-wide Live Map, so the official friends endpoint has
//! no coordinates to merge. When location sharing is enabled, this client
//! sends the already-normalized own position to our Worker. The Worker uses
//! the IslePilot bearer token to verify identity, accepted relationships and
//! current server before returning any short-lived friend position.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::state::LockExt;

use super::FriendUpdate;

const REFRESH_MS: u64 = 20_000;
const CACHE_MAX_AGE: Duration = Duration::from_secs(70);
const HTTP_TIMEOUT: Duration = Duration::from_secs(3);
const RELEASE_API_BASE: &str = "https://islemap-thienvyma-api.islemap-thienvyma-api.workers.dev";

static NEXT_ATTEMPT_MS: AtomicU64 = AtomicU64::new(0);
static IN_FLIGHT: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);
static CACHE: LazyLock<Mutex<Option<PresenceCache>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelayFriend {
    steam_id: String,
    position_cm: (f64, f64, f64),
    age_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelayResponse {
    server: Option<String>,
    ttl_seconds: u64,
    friends: Vec<RelayFriend>,
}

#[derive(Clone)]
struct PresenceCache {
    server_key: String,
    received: Instant,
    friends: Vec<RelayFriend>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn server_key(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn api_base() -> Option<String> {
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("OV_PRESENCE_API_BASE") {
        let value = value.trim().trim_end_matches('/');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    let value = RELEASE_API_BASE.trim().trim_end_matches('/');
    (!value.is_empty()).then(|| value.to_string())
}

fn valid_position(position: (f64, f64, f64)) -> bool {
    [position.0, position.1, position.2]
        .into_iter()
        .all(|value| value.is_finite() && value.abs() <= 10_000_000.0)
}

fn fetch(base: &str, token: &str, position_cm: (f64, f64, f64)) -> Result<RelayResponse, String> {
    let body = serde_json::to_vec(&serde_json::json!({ "positionCm": position_cm }))
        .map_err(|error| error.to_string())?;
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("islemap-thienvyma/", env!("CARGO_PKG_VERSION")))
        .timeout(HTTP_TIMEOUT)
        .connect_timeout(Duration::from_secs(2))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .post(format!("{base}/v1/presence"))
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .header("x-ov-ver", env!("CARGO_PKG_VERSION"))
        .body(body)
        .send()
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    let text = response.text().map_err(|error| error.to_string())?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn sanitize(mut response: RelayResponse, expected_server: &str) -> Option<PresenceCache> {
    let response_server = response.server.as_deref()?;
    let expected_key = server_key(expected_server);
    if expected_key.is_empty() || server_key(response_server) != expected_key {
        return None;
    }
    let ttl = response.ttl_seconds.clamp(1, CACHE_MAX_AGE.as_secs());
    response.friends.retain(|friend| {
        friend.steam_id.len() == 17
            && friend.steam_id.bytes().all(|byte| byte.is_ascii_digit())
            && friend.age_seconds <= ttl
            && valid_position(friend.position_cm)
    });
    Some(PresenceCache {
        server_key: expected_key,
        received: Instant::now(),
        friends: response.friends,
    })
}

pub fn clear() {
    GENERATION.fetch_add(1, Ordering::AcqRel);
    *CACHE.lock_safe() = None;
}

/// Queue a refresh without delaying IslePilot's normal `/me`, `/friends`,
/// Garage, Skin or Prime updates. The current poll publishes immediately and
/// consumes the relay result on the next poll.
pub fn queue_refresh(
    token: &str,
    server: Option<&str>,
    share_location: bool,
    online: bool,
    position_cm: Option<(f64, f64, f64)>,
    has_friend_without_position: bool,
) {
    let Some(base) = api_base() else {
        return;
    };
    let Some(server) = server.map(str::trim).filter(|value| !value.is_empty()) else {
        clear();
        return;
    };
    let Some(position_cm) = position_cm.filter(|value| valid_position(*value)) else {
        clear();
        return;
    };
    if !share_location || !online || !has_friend_without_position {
        clear();
        return;
    }

    let now = now_ms();
    if now < NEXT_ATTEMPT_MS.load(Ordering::Relaxed) {
        return;
    }
    NEXT_ATTEMPT_MS.store(now + REFRESH_MS, Ordering::Relaxed);
    if IN_FLIGHT.swap(true, Ordering::AcqRel) {
        return;
    }

    let token = token.to_string();
    let expected_server = server.to_string();
    let generation = GENERATION.load(Ordering::Acquire);
    std::thread::spawn(move || {
        let result = fetch(&base, &token, position_cm)
            .ok()
            .and_then(|response| sanitize(response, &expected_server));
        if GENERATION.load(Ordering::Acquire) == generation {
            match result {
                Some(cache) => *CACHE.lock_safe() = Some(cache),
                None => log::debug!("friend presence relay unavailable"),
            }
        }
        IN_FLIGHT.store(false, Ordering::Release);
    });
}

/// Fill only missing positions. Official IslePilot map coordinates keep
/// priority on DinoVietnam and on any future server that enables Live Map.
pub fn merge_cached(server: Option<&str>, friends: &mut [FriendUpdate]) {
    let Some(expected_key) = server.map(server_key).filter(|value| !value.is_empty()) else {
        return;
    };
    let cache = CACHE.lock_safe().clone();
    let Some(cache) = cache.filter(|cache| {
        cache.server_key == expected_key && cache.received.elapsed() <= CACHE_MAX_AGE
    }) else {
        return;
    };
    for friend in friends
        .iter_mut()
        .filter(|friend| friend.position_cm.is_none())
    {
        let Some(steam_id) = friend.steam_id.as_deref() else {
            continue;
        };
        if let Some(relay) = cache.friends.iter().find(|item| item.steam_id == steam_id) {
            friend.position_cm = Some(relay.position_cm);
            friend.online = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_fills_missing_position_without_overwriting_official_map() {
        *CACHE.lock_safe() = Some(PresenceCache {
            server_key: server_key("TiTan  Isle Vietnam"),
            received: Instant::now(),
            friends: vec![RelayFriend {
                steam_id: "76561198000000002".into(),
                position_cm: (11.0, 22.0, 33.0),
                age_seconds: 1,
            }],
        });
        let mut friends = vec![
            FriendUpdate {
                slot: Some(1),
                steam_id: Some("76561198000000002".into()),
                name: "Titan friend".into(),
                dino_name: None,
                online: true,
                position_cm: None,
            },
            FriendUpdate {
                slot: Some(2),
                steam_id: Some("76561198000000003".into()),
                name: "DinoVietnam friend".into(),
                dino_name: None,
                online: true,
                position_cm: Some((90.0, 80.0, 70.0)),
            },
        ];

        merge_cached(Some("titan isle vietnam"), &mut friends);
        assert_eq!(friends[0].position_cm, Some((11.0, 22.0, 33.0)));
        assert_eq!(friends[1].position_cm, Some((90.0, 80.0, 70.0)));
        clear();
    }
}
