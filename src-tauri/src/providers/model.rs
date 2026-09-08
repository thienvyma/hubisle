use serde::{Deserialize, Serialize};

use crate::islepilot::parser::{Nutrition, QuestStatus};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderId {
    IslePilot,
    Era,
    Titan,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub id: ProviderId,
    pub website: String,
    pub automatic_position: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionStatus {
    Unconfigured,
    Detecting,
    LoginRequired,
    Validating,
    AuthenticatedOnline,
    AuthenticatedOffline,
    TemporaryError,
    Unsupported,
    LoggedOut,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SharedStatBar {
    pub percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    pub raw: String,
}

impl SharedStatBar {
    pub fn from_percent(percent: f64) -> Self {
        Self {
            percent,
            current: None,
            max: None,
            raw: format!("{}%", round1(percent)),
        }
    }

    pub fn from_values(current: f64, max: f64) -> Self {
        Self {
            percent: current / max * 100.0,
            current: Some(current),
            max: Some(max),
            raw: format!("{} / {}", round1(current), round1(max)),
        }
    }
}

fn round1(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded:.1}")
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SharedPlayer {
    pub name: Option<String>,
    pub dino_name: Option<String>,
    pub female: Option<bool>,
    pub growth_pct: Option<f64>,
    pub health: Option<SharedStatBar>,
    pub stamina: Option<SharedStatBar>,
    pub hunger: Option<SharedStatBar>,
    pub thirst: Option<SharedStatBar>,
    pub mutations: Vec<String>,
    pub nutrition: Option<Nutrition>,
    pub prime_quests: Vec<QuestStatus>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSnapshot {
    pub provider: ProviderId,
    pub status: ConnectionStatus,
    pub server_id: Option<String>,
    pub server_name: Option<String>,
    pub received_at_ms: i64,
    pub source_timestamp_ms: Option<i64>,
    pub player: Option<SharedPlayer>,
    pub position_cm: Option<(f64, f64, f64)>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderState {
    pub provider: Option<ProviderId>,
    pub website: Option<String>,
    pub status: ConnectionStatus,
    pub message: Option<String>,
    pub last_received_at_ms: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_only_bar_keeps_zero_without_fabricating_values() {
        let bar = SharedStatBar::from_percent(0.0);
        assert_eq!(bar.percent, 0.0);
        assert_eq!(bar.current, None);
        assert_eq!(bar.max, None);

        let value = serde_json::to_value(&bar).unwrap();
        assert_eq!(value["percent"], 0.0);
        assert!(value.get("current").is_none());
        assert!(value.get("max").is_none());
    }
}
