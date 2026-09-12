//! Persistent combat history shared by the main hub and the minimap.
//!
//! Every supported provider exposes the signed-in player's health, so health
//! loss can be recorded consistently across Era, Titan, and IslePilot. Some
//! Era deployments can additionally attach a `combatEvents` array. We accept
//! that server-owned data when present. An inferred health drop never
//! fabricates an attacker identity: falls, bleeding and starvation can all
//! reduce health too.

use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use crate::{
    providers::model::{ConnectionStatus, ProviderSnapshot},
    settings,
};

pub const COMBAT_NEW: &str = "combat://new";
const MAX_HISTORY: usize = 500;
const LIVE_EVENT_MAX_AGE_MS: i64 = 120_000;
static HISTORY_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CombatDirection {
    Incoming,
    Outgoing,
    Death,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CombatEvent {
    pub id: String,
    pub timestamp_ms: i64,
    pub direction: CombatDirection,
    pub self_species: Option<String>,
    pub opponent_name: Option<String>,
    pub opponent_species: Option<String>,
    pub damage: Option<f64>,
    /// `era-server` is authoritative provider data. `health-delta` is only an
    /// observed loss of health and deliberately carries no opponent identity.
    pub source: String,
    pub server_name: Option<String>,
}

fn history_path() -> PathBuf {
    settings::local_dir().join("combat-history.json")
}

fn read_history_unlocked() -> Vec<CombatEvent> {
    std::fs::read(history_path())
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn write_history_unlocked(events: &[CombatEvent]) -> Result<(), String> {
    let path = history_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(events).map_err(|error| error.to_string())?;
    std::fs::write(path, bytes).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn combat_history() -> Vec<CombatEvent> {
    let _guard = HISTORY_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut events = read_history_unlocked();
    events.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
    events.truncate(MAX_HISTORY);
    events
}

/// Merge, persist, and broadcast only genuinely new recent events. Old server
/// history is still imported into the hub but does not flash on the minimap.
pub fn ingest(app: &AppHandle, incoming: Vec<CombatEvent>) {
    if incoming.is_empty() {
        return;
    }
    let now = chrono::Utc::now().timestamp_millis();
    let added = {
        let _guard = HISTORY_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut events = read_history_unlocked();
        let mut seen = events
            .iter()
            .map(|event| event.id.clone())
            .collect::<HashSet<_>>();
        let mut added = Vec::new();
        for event in incoming {
            if seen.insert(event.id.clone()) {
                added.push(event.clone());
                events.push(event);
            }
        }
        if added.is_empty() {
            return;
        }
        events.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
        events.truncate(MAX_HISTORY);
        if let Err(error) = write_history_unlocked(&events) {
            log::warn!("save combat history failed: {error}");
        }
        added
    };
    for event in added {
        if now.saturating_sub(event.timestamp_ms).abs() <= LIVE_EVENT_MAX_AGE_MS {
            crate::events::emit_all(app, COMBAT_NEW, event);
        }
    }
}

fn clean_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn first_string(object: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| clean_string(object.get(*key)))
}

fn number(object: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| {
        object
            .get(*key)
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite())
    })
}

fn timestamp_ms(item: &Value, fallback: i64) -> i64 {
    let Some(value) = item
        .get("timestampMs")
        .or_else(|| item.get("timestamp_ms"))
        .or_else(|| item.get("timestamp"))
        .or_else(|| item.get("ts"))
    else {
        return fallback;
    };
    if let Some(number) = value.as_i64() {
        return if number.abs() < 10_000_000_000 {
            number.saturating_mul(1000)
        } else {
            number
        };
    }
    value
        .as_str()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.timestamp_millis())
        .unwrap_or(fallback)
}

fn parse_direction(item: &Value) -> Option<CombatDirection> {
    let kind = first_string(item, &["direction", "event", "type", "action"])?
        .to_ascii_lowercase()
        .replace('_', "-");
    if matches!(
        kind.as_str(),
        "incoming" | "attacked-me" | "damage-received" | "hit-received" | "received"
    ) {
        Some(CombatDirection::Incoming)
    } else if matches!(
        kind.as_str(),
        "outgoing" | "i-attacked" | "damage-dealt" | "hit-dealt" | "dealt"
    ) {
        Some(CombatDirection::Outgoing)
    } else if matches!(kind.as_str(), "death" | "died" | "killed") {
        Some(CombatDirection::Death)
    } else {
        None
    }
}

fn stable_id(parts: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    parts.hash(&mut hasher);
    format!("era-{:016x}", hasher.finish())
}

/// Parse the optional provider-native combat event contract. This is kept
/// deliberately strict: unknown event kinds are ignored, and identity fields
/// are copied only when the Era backend sends them explicitly.
pub fn parse_era_events(
    root: &Value,
    fallback_timestamp_ms: i64,
    server_name: Option<&str>,
    self_species: Option<&str>,
) -> Vec<CombatEvent> {
    let arrays = [
        root.get("combatEvents"),
        root.get("combat_events"),
        root.get("combatHistory"),
        root.get("player")
            .and_then(|player| player.get("combatEvents")),
    ];
    let Some(items) = arrays.into_iter().flatten().find_map(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let direction = parse_direction(item)?;
            let timestamp_ms = timestamp_ms(item, fallback_timestamp_ms);
            let opponent_name = match direction {
                CombatDirection::Incoming | CombatDirection::Death => first_string(
                    item,
                    &[
                        "opponentName",
                        "opponent_name",
                        "attackerName",
                        "attacker_name",
                    ],
                ),
                CombatDirection::Outgoing => first_string(
                    item,
                    &[
                        "opponentName",
                        "opponent_name",
                        "targetName",
                        "target_name",
                        "victimName",
                    ],
                ),
            };
            let opponent_species = match direction {
                CombatDirection::Incoming | CombatDirection::Death => first_string(
                    item,
                    &[
                        "opponentSpecies",
                        "opponent_species",
                        "attackerSpecies",
                        "attacker_species",
                    ],
                ),
                CombatDirection::Outgoing => first_string(
                    item,
                    &[
                        "opponentSpecies",
                        "opponent_species",
                        "targetSpecies",
                        "target_species",
                        "victimSpecies",
                    ],
                ),
            };
            let own = first_string(item, &["selfSpecies", "self_species", "playerSpecies"])
                .or_else(|| self_species.map(str::to_string));
            let id = first_string(item, &["id", "eventId", "event_id"]).unwrap_or_else(|| {
                stable_id(&[
                    timestamp_ms.to_string(),
                    format!("{direction:?}"),
                    own.clone().unwrap_or_default(),
                    opponent_name.clone().unwrap_or_default(),
                    opponent_species.clone().unwrap_or_default(),
                ])
            });
            Some(CombatEvent {
                id: format!("era-server:{id}"),
                timestamp_ms,
                direction,
                self_species: own,
                opponent_name,
                opponent_species,
                damage: number(item, &["damage", "amount", "damageAmount"]),
                source: "era-server".to_string(),
                server_name: server_name.map(str::to_string),
            })
        })
        .collect()
}

/// Infer a single health-loss record between consecutive samples from the
/// same active provider session. This works with every provider that reports
/// health while staying honest about the lack of an attacker identity.
pub fn infer_health_drop(
    previous: &ProviderSnapshot,
    current: &ProviderSnapshot,
) -> Option<CombatEvent> {
    if previous.provider != current.provider
        || previous.status != ConnectionStatus::AuthenticatedOnline
        || current.status != ConnectionStatus::AuthenticatedOnline
        || current.received_at_ms <= previous.received_at_ms
    {
        return None;
    }
    if let (Some(previous_id), Some(current_id)) = (&previous.server_id, &current.server_id) {
        if previous_id != current_id {
            return None;
        }
    } else if let (Some(previous_name), Some(current_name)) =
        (&previous.server_name, &current.server_name)
    {
        if previous_name != current_name {
            return None;
        }
    }
    let previous_player = previous.player.as_ref()?;
    let current_player = current.player.as_ref()?;
    if previous_player.dino_name != current_player.dino_name {
        return None;
    }
    let before = previous_player.health.as_ref()?.percent;
    let after = current_player.health.as_ref()?.percent;
    let damage = before - after;
    if !damage.is_finite() || damage < 0.1 {
        return None;
    }
    Some(CombatEvent {
        id: stable_id(&[
            "health-delta".to_string(),
            format!("{:?}", current.provider),
            current
                .server_id
                .as_deref()
                .or(current.server_name.as_deref())
                .unwrap_or_default()
                .to_string(),
            current.received_at_ms.to_string(),
            format!("{after:.3}"),
        ]),
        timestamp_ms: current.received_at_ms,
        direction: CombatDirection::Incoming,
        self_species: current_player.dino_name.clone(),
        opponent_name: None,
        opponent_species: None,
        damage: Some(damage),
        source: "health-delta".to_string(),
        server_name: current.server_name.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::model::{ConnectionStatus, ProviderId, SharedPlayer, SharedStatBar};
    use serde_json::json;

    fn snapshot(health: f64) -> ProviderSnapshot {
        ProviderSnapshot {
            provider: ProviderId::Era,
            status: ConnectionStatus::AuthenticatedOnline,
            server_id: None,
            server_name: Some("ERA".to_string()),
            received_at_ms: 1_700_000_000_000,
            source_timestamp_ms: None,
            player: Some(SharedPlayer {
                name: Some("Me".to_string()),
                dino_name: Some("Stegosaurus".to_string()),
                female: None,
                growth_pct: None,
                health: Some(SharedStatBar::from_percent(health)),
                stamina: None,
                hunger: None,
                thirst: None,
                mutations: Vec::new(),
                nutrition: None,
                prime_quests: Vec::new(),
            }),
            friends: Vec::new(),
            position_cm: None,
            heading_deg: None,
        }
    }

    #[test]
    fn parses_authoritative_era_identity_fields() {
        let value = json!({
            "combatEvents": [{
                "id": "event-42",
                "direction": "incoming",
                "timestamp": 1_700_000_000,
                "attackerName": "RaptorVN",
                "attackerSpecies": "Omniraptor",
                "damage": 12.5
            }]
        });
        let events = parse_era_events(&value, 0, Some("ERA"), Some("Stegosaurus"));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].opponent_name.as_deref(), Some("RaptorVN"));
        assert_eq!(events[0].opponent_species.as_deref(), Some("Omniraptor"));
        assert_eq!(events[0].timestamp_ms, 1_700_000_000_000);
        assert_eq!(events[0].source, "era-server");
    }

    #[test]
    fn health_drop_never_invents_an_opponent() {
        let before = snapshot(88.0);
        let mut after = snapshot(73.5);
        after.received_at_ms += 5_000;
        let event = infer_health_drop(&before, &after).unwrap();
        assert_eq!(event.damage, Some(14.5));
        assert_eq!(event.self_species.as_deref(), Some("Stegosaurus"));
        assert_eq!(event.opponent_name, None);
        assert_eq!(event.opponent_species, None);
        assert_eq!(event.source, "health-delta");
    }

    #[test]
    fn stable_health_does_not_create_history() {
        assert!(infer_health_drop(&snapshot(88.0), &snapshot(88.0)).is_none());
        assert!(infer_health_drop(&snapshot(88.0), &snapshot(90.0)).is_none());
    }

    #[test]
    fn health_drop_is_provider_generic_and_session_scoped() {
        let before = snapshot(90.0);
        let mut after = snapshot(65.0);
        after.provider = ProviderId::Titan;
        after.received_at_ms += 1_000;
        assert!(infer_health_drop(&before, &after).is_none());

        let mut before = before;
        before.provider = ProviderId::Titan;
        let event = infer_health_drop(&before, &after).expect("Titan health loss");
        assert_eq!(event.damage, Some(25.0));
        assert_eq!(event.source, "health-delta");
    }
}
