use std::path::PathBuf;

use crate::{secure_store, settings};

use super::{
    model::VoiceSession,
    provider::{VoiceError, VoiceErrorCode},
};

pub trait VoiceSessionRepository: Send + Sync {
    fn get(&self, provider_id: &str, origin: &str) -> Option<VoiceSession>;
    fn set(&self, session: VoiceSession) -> Result<(), VoiceError>;
    fn remove(&self, provider_id: &str, origin: &str) -> Result<(), VoiceError>;
}

pub struct DpapiVoiceSessionStore {
    path: PathBuf,
}

impl DpapiVoiceSessionStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn in_app_data() -> Self {
        Self::new(settings::local_dir().join("voice_sessions.bin"))
    }

    fn load(&self) -> Vec<VoiceSession> {
        let Ok(sealed) = std::fs::read(&self.path) else {
            return Vec::new();
        };
        secure_store::dpapi_unprotect(&sealed)
            .ok()
            .and_then(|plain| serde_json::from_slice(&plain).ok())
            .unwrap_or_default()
    }

    fn save(&self, records: &[VoiceSession]) -> Result<(), VoiceError> {
        let plain = serde_json::to_vec(records)
            .map_err(|_| VoiceError::new(VoiceErrorCode::StorageUnavailable))?;
        let sealed = secure_store::dpapi_protect(&plain)
            .map_err(|_| VoiceError::new(VoiceErrorCode::StorageUnavailable))?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| VoiceError::new(VoiceErrorCode::StorageUnavailable))?;
        }
        std::fs::write(&self.path, sealed)
            .map_err(|_| VoiceError::new(VoiceErrorCode::StorageUnavailable))
    }
}

impl VoiceSessionRepository for DpapiVoiceSessionStore {
    fn get(&self, provider_id: &str, origin: &str) -> Option<VoiceSession> {
        find_session(&self.load(), provider_id, origin).cloned()
    }

    fn set(&self, session: VoiceSession) -> Result<(), VoiceError> {
        let mut records = self.load();
        records.retain(|record| {
            record.provider_id != session.provider_id || record.origin != session.origin
        });
        records.push(session);
        self.save(&records)
    }

    fn remove(&self, provider_id: &str, origin: &str) -> Result<(), VoiceError> {
        let mut records = self.load();
        records.retain(|record| record.provider_id != provider_id || record.origin != origin);
        self.save(&records)
    }
}

fn find_session<'a>(
    records: &'a [VoiceSession],
    provider_id: &str,
    origin: &str,
) -> Option<&'a VoiceSession> {
    records
        .iter()
        .find(|record| record.provider_id == provider_id && record.origin == origin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice::model::VoiceSessionSummary;

    fn session(provider: &str, origin: &str, token: &str) -> VoiceSession {
        VoiceSession {
            provider_id: provider.to_string(),
            origin: origin.to_string(),
            player_name: Some("Player".to_string()),
            steam_id: Some("76561198000000000".to_string()),
            expires_at_ms: Some(9_999_999),
            access_token: token.to_string(),
        }
    }

    #[test]
    fn record_lookup_is_scoped_to_provider_and_exact_origin() {
        let records = vec![session(
            "voice-a",
            "https://server.example.test",
            "session-secret",
        )];
        assert!(find_session(&records, "voice-a", "https://server.example.test").is_some());
        assert!(find_session(&records, "voice-b", "https://server.example.test").is_none());
        assert!(find_session(&records, "voice-a", "https://other.example.test").is_none());
    }

    #[test]
    fn serialized_summary_never_contains_the_session_secret() {
        let source = session(
            "voice-a",
            "https://server.example.test",
            "do-not-serialize-this",
        );
        let json = serde_json::to_string(&VoiceSessionSummary::from(&source)).unwrap();
        assert!(!json.contains("do-not-serialize-this"));
        assert!(!json.contains("accessToken"));
        assert!(json.contains("76561198000000000"));
    }

    #[test]
    fn dpapi_store_round_trips_scoped_sessions_without_plaintext_on_disk() {
        let path = std::env::temp_dir().join(format!(
            "islemap-voice-session-test-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = DpapiVoiceSessionStore::new(path.clone());
        let source = session(
            "voice-a",
            "https://server.example.test",
            "protected-session-secret",
        );
        store.set(source.clone()).unwrap();

        assert_eq!(
            store.get("voice-a", "https://server.example.test"),
            Some(source)
        );
        assert!(store
            .get("voice-b", "https://server.example.test")
            .is_none());
        assert!(!String::from_utf8_lossy(&std::fs::read(&path).unwrap())
            .contains("protected-session-secret"));

        store
            .remove("voice-a", "https://server.example.test")
            .unwrap();
        assert!(store
            .get("voice-a", "https://server.example.test")
            .is_none());
        std::fs::remove_file(path).unwrap();
    }
}
