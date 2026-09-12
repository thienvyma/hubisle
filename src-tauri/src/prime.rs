//! Provider-neutral Prime progress notifications.
//!
//! The first snapshot is only a baseline. A popup is emitted when a later
//! snapshot from the same provider/server changes a quest from pending to
//! complete, so reconnects and app restarts do not replay old progress.

use serde::Serialize;

use crate::providers::model::{ConnectionStatus, ProviderSnapshot};

pub const PRIME_PROGRESS: &str = "prime://progress";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrimeNotificationKind {
    Quest,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimeProgressNotification {
    pub id: String,
    pub kind: PrimeNotificationKind,
    pub quest_text: Option<String>,
    pub quest_text_vi: Option<String>,
    pub completed: usize,
    pub total: usize,
}

fn same_session(previous: &ProviderSnapshot, current: &ProviderSnapshot) -> bool {
    if previous.provider != current.provider
        || previous.status != ConnectionStatus::AuthenticatedOnline
        || current.status != ConnectionStatus::AuthenticatedOnline
        || current.received_at_ms <= previous.received_at_ms
    {
        return false;
    }
    if let (Some(previous_id), Some(current_id)) = (&previous.server_id, &current.server_id) {
        return previous_id == current_id;
    }
    if let (Some(previous_name), Some(current_name)) = (&previous.server_name, &current.server_name)
    {
        return previous_name == current_name;
    }
    true
}

pub fn progress_between(
    previous: &ProviderSnapshot,
    current: &ProviderSnapshot,
) -> Vec<PrimeProgressNotification> {
    if !same_session(previous, current) {
        return Vec::new();
    }
    let (Some(previous_player), Some(current_player)) =
        (previous.player.as_ref(), current.player.as_ref())
    else {
        return Vec::new();
    };
    if previous_player.dino_name != current_player.dino_name
        || previous_player.prime_quests.is_empty()
        || current_player.prime_quests.is_empty()
    {
        return Vec::new();
    }

    let completed = current_player
        .prime_quests
        .iter()
        .filter(|quest| quest.completed)
        .count();
    let previous_completed = previous_player
        .prime_quests
        .iter()
        .filter(|quest| quest.completed)
        .count();
    let total = current_player.prime_quests.len();
    let mut notifications = Vec::new();

    for (index, quest) in current_player.prime_quests.iter().enumerate() {
        if !quest.completed {
            continue;
        }
        let was_completed = previous_player
            .prime_quests
            .iter()
            .find(|candidate| candidate.text == quest.text)
            .or_else(|| {
                (previous_player.prime_quests.len() == total)
                    .then(|| previous_player.prime_quests.get(index))
                    .flatten()
            })
            .is_some_and(|candidate| candidate.completed);
        if was_completed {
            continue;
        }
        notifications.push(PrimeProgressNotification {
            id: format!(
                "prime:{:?}:{}:{}:{index}",
                current.provider,
                current.received_at_ms,
                current.server_id.as_deref().unwrap_or_default()
            ),
            kind: PrimeNotificationKind::Quest,
            quest_text: Some(quest.text.clone()),
            quest_text_vi: quest.text_vi.clone(),
            completed,
            total,
        });
    }

    if completed == total && previous_completed < total && !notifications.is_empty() {
        notifications.push(PrimeProgressNotification {
            id: format!(
                "prime-complete:{:?}:{}:{}",
                current.provider,
                current.received_at_ms,
                current.server_id.as_deref().unwrap_or_default()
            ),
            kind: PrimeNotificationKind::Complete,
            quest_text: None,
            quest_text_vi: None,
            completed,
            total,
        });
    }
    notifications
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        islepilot::parser::QuestStatus,
        providers::model::{ProviderId, SharedPlayer},
    };

    fn snapshot(provider: ProviderId, time: i64, states: &[bool]) -> ProviderSnapshot {
        ProviderSnapshot {
            provider,
            status: ConnectionStatus::AuthenticatedOnline,
            server_id: Some("server-a".to_string()),
            server_name: Some("Server A".to_string()),
            received_at_ms: time,
            source_timestamp_ms: Some(time),
            player: Some(SharedPlayer {
                name: None,
                dino_name: Some("Utahraptor".to_string()),
                female: None,
                growth_pct: None,
                health: None,
                stamina: None,
                hunger: None,
                thirst: None,
                mutations: Vec::new(),
                nutrition: None,
                prime_quests: states
                    .iter()
                    .enumerate()
                    .map(|(index, completed)| QuestStatus {
                        text: format!("Quest {index}"),
                        text_vi: Some(format!("Nhiệm vụ {index}")),
                        completed: *completed,
                    })
                    .collect(),
            }),
            friends: Vec::new(),
            position_cm: None,
            heading_deg: None,
        }
    }

    #[test]
    fn reports_new_progress_for_every_provider() {
        for provider in [ProviderId::Era, ProviderId::Titan, ProviderId::IslePilot] {
            let events = progress_between(
                &snapshot(provider, 1, &[false, false]),
                &snapshot(provider, 2, &[true, false]),
            );
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].kind, PrimeNotificationKind::Quest);
            assert_eq!(events[0].completed, 1);
        }
    }

    #[test]
    fn reports_the_last_quest_and_whole_set_completion() {
        let events = progress_between(
            &snapshot(ProviderId::Titan, 1, &[true, false]),
            &snapshot(ProviderId::Titan, 2, &[true, true]),
        );
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, PrimeNotificationKind::Quest);
        assert_eq!(events[1].kind, PrimeNotificationKind::Complete);
    }

    #[test]
    fn ignores_replayed_and_cross_server_snapshots() {
        let previous = snapshot(ProviderId::Era, 1, &[false]);
        let already_done = snapshot(ProviderId::Era, 2, &[true]);
        assert!(progress_between(&already_done, &already_done).is_empty());

        let mut other_server = already_done;
        other_server.server_id = Some("server-b".to_string());
        assert!(progress_between(&previous, &other_server).is_empty());
    }
}
