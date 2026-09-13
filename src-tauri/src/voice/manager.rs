use std::sync::{Arc, Mutex};

use reqwest::Url;
use uuid::Uuid;

use super::{
    model::{
        VoiceAuthCallback, VoiceAuthStart, VoiceGrant, VoiceServerContext, VoiceSession,
        VoiceState, VoiceStatus,
    },
    provider::{VoiceError, VoiceErrorCode},
    registry::VoiceRegistry,
    session_store::VoiceSessionRepository,
};

pub const LOGIN_TTL_MS: i64 = 5 * 60 * 1000;
pub const CALLBACK_URI: &str = "islemap-thienvyma://voice/callback";

#[derive(Clone)]
struct PendingLogin {
    context: VoiceServerContext,
    provider_id: String,
    security: super::model::LoginSecurity,
    expires_at_ms: i64,
}

pub struct VoiceManager {
    registry: VoiceRegistry,
    sessions: Arc<dyn VoiceSessionRepository>,
    pending: Mutex<Option<PendingLogin>>,
}

impl VoiceManager {
    pub fn new(registry: VoiceRegistry, sessions: Arc<dyn VoiceSessionRepository>) -> Self {
        Self {
            registry,
            sessions,
            pending: Mutex::new(None),
        }
    }

    pub fn status_at(&self, context: Option<VoiceServerContext>, now_ms: i64) -> VoiceStatus {
        let Some(context) = context else {
            return empty_status(None, None, VoiceState::NotConfigured, None);
        };
        let context = match normalized_context(context) {
            Ok(context) => context,
            Err(error) => {
                return empty_status(
                    None,
                    None,
                    VoiceState::Error,
                    Some(error.as_str().to_string()),
                )
            }
        };
        let provider = match self.registry.find(&context.origin) {
            Ok(Some(provider)) => provider,
            Ok(None) => {
                return empty_status(
                    Some(context.origin),
                    context.server_name,
                    VoiceState::NotConfigured,
                    None,
                )
            }
            Err(error) => {
                return empty_status(
                    Some(context.origin),
                    context.server_name,
                    VoiceState::Error,
                    Some(error.as_str().to_string()),
                )
            }
        };
        let descriptor = provider.descriptor();
        let is_authorizing = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|pending| {
                pending.provider_id == descriptor.id
                    && pending.context.origin == context.origin
                    && pending.expires_at_ms >= now_ms
            });
        if is_authorizing {
            return VoiceStatus {
                state: VoiceState::Authorizing,
                server_origin: Some(context.origin),
                server_name: context.server_name,
                provider: Some(descriptor),
                session: None,
                reason: None,
            };
        }
        let Some(session) = self.sessions.get(&descriptor.id, &context.origin) else {
            return VoiceStatus {
                state: VoiceState::LoginRequired,
                server_origin: Some(context.origin),
                server_name: context.server_name,
                provider: Some(descriptor),
                session: None,
                reason: None,
            };
        };
        if session.expires_at_ms.is_some_and(|expiry| expiry <= now_ms) {
            let _ = self.sessions.remove(&descriptor.id, &context.origin);
            return VoiceStatus {
                state: VoiceState::LoginRequired,
                server_origin: Some(context.origin),
                server_name: context.server_name,
                provider: Some(descriptor),
                session: None,
                reason: None,
            };
        }
        match provider.session_status(&context, &session) {
            Ok(summary) => VoiceStatus {
                state: VoiceState::Ready,
                server_origin: Some(context.origin),
                server_name: context.server_name,
                provider: Some(descriptor),
                session: Some(summary),
                reason: None,
            },
            Err(error) => {
                if error.code == VoiceErrorCode::LoginRequired {
                    let _ = self.sessions.remove(&descriptor.id, &context.origin);
                }
                VoiceStatus {
                    state: state_for_error(error.code),
                    server_origin: Some(context.origin),
                    server_name: context.server_name,
                    provider: Some(descriptor),
                    session: None,
                    reason: Some(error.as_str().to_string()),
                }
            }
        }
    }

    pub fn status(&self, context: Option<VoiceServerContext>) -> VoiceStatus {
        self.status_at(context, unix_time_ms())
    }

    pub fn start_login(&self, context: VoiceServerContext) -> Result<VoiceAuthStart, VoiceError> {
        self.start_login_at(context, unix_time_ms())
    }

    pub fn handle_callback(&self, callback_url: &str) -> Result<(), VoiceError> {
        self.handle_callback_at(callback_url, unix_time_ms())
    }

    pub fn cancel_login(&self) {
        self.clear_pending();
    }

    pub fn start_login_at(
        &self,
        context: VoiceServerContext,
        now_ms: i64,
    ) -> Result<VoiceAuthStart, VoiceError> {
        let context = normalized_context(context)?;
        let provider = self
            .registry
            .find(&context.origin)?
            .ok_or_else(|| VoiceError::new(VoiceErrorCode::NotConfigured))?;
        let descriptor = provider.descriptor();
        let security = super::model::LoginSecurity {
            state: Uuid::new_v4().to_string(),
            nonce: Uuid::new_v4().to_string(),
            pkce_verifier: format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple()),
            callback_uri: CALLBACK_URI.to_string(),
        };
        let expires_at_ms = now_ms + LOGIN_TTL_MS;
        let start = provider.start_login(&context, &security, expires_at_ms)?;
        validate_auth_start(&start, &descriptor.id, expires_at_ms)?;
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(PendingLogin {
            context,
            provider_id: descriptor.id,
            security,
            expires_at_ms,
        });
        Ok(start)
    }

    pub fn handle_callback_at(&self, callback_url: &str, now_ms: i64) -> Result<(), VoiceError> {
        let callback = parse_callback(callback_url)?;
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or_else(|| VoiceError::new(VoiceErrorCode::InvalidCallback))?;
        if pending.expires_at_ms < now_ms {
            self.clear_pending();
            return Err(VoiceError::new(VoiceErrorCode::LoginExpired));
        }
        if callback.provider_id != pending.provider_id
            || !constant_time_eq(callback.state.as_bytes(), pending.security.state.as_bytes())
        {
            return Err(VoiceError::new(VoiceErrorCode::InvalidCallback));
        }
        let provider = self
            .registry
            .find(&pending.context.origin)?
            .filter(|provider| provider.descriptor().id == pending.provider_id)
            .ok_or_else(|| VoiceError::new(VoiceErrorCode::InvalidProvider))?;
        self.clear_pending();
        let session = provider.finish_login(&pending.context, &callback, &pending.security)?;
        validate_session(&session, &pending, now_ms)?;
        self.sessions.set(session)
    }

    pub fn request_grant_at(
        &self,
        context: VoiceServerContext,
        now_ms: i64,
    ) -> Result<VoiceGrant, VoiceError> {
        let context = normalized_context(context)?;
        let provider = self
            .registry
            .find(&context.origin)?
            .ok_or_else(|| VoiceError::new(VoiceErrorCode::NotConfigured))?;
        let descriptor = provider.descriptor();
        let session = self
            .sessions
            .get(&descriptor.id, &context.origin)
            .ok_or_else(|| VoiceError::new(VoiceErrorCode::LoginRequired))?;
        if session.expires_at_ms.is_some_and(|expiry| expiry <= now_ms) {
            let _ = self.sessions.remove(&descriptor.id, &context.origin);
            return Err(VoiceError::new(VoiceErrorCode::LoginRequired));
        }
        let grant = provider.request_grant(&context, &session)?;
        validate_grant(&grant, now_ms)?;
        Ok(grant)
    }

    pub fn logout(&self, context: VoiceServerContext) -> Result<(), VoiceError> {
        let context = normalized_context(context)?;
        let provider = self
            .registry
            .find(&context.origin)?
            .ok_or_else(|| VoiceError::new(VoiceErrorCode::NotConfigured))?;
        let descriptor = provider.descriptor();
        let result = self
            .sessions
            .get(&descriptor.id, &context.origin)
            .map(|session| provider.logout(&context, &session))
            .unwrap_or(Ok(()));
        self.sessions.remove(&descriptor.id, &context.origin)?;
        result
    }

    fn clear_pending(&self) {
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}

fn unix_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn empty_status(
    server_origin: Option<String>,
    server_name: Option<String>,
    state: VoiceState,
    reason: Option<String>,
) -> VoiceStatus {
    VoiceStatus {
        state,
        server_origin,
        server_name,
        provider: None,
        session: None,
        reason,
    }
}

fn normalized_context(mut context: VoiceServerContext) -> Result<VoiceServerContext, VoiceError> {
    context.origin = super::registry::normalize_origin(&context.origin)?;
    Ok(context)
}

fn validate_auth_start(
    start: &VoiceAuthStart,
    provider_id: &str,
    expires_at_ms: i64,
) -> Result<(), VoiceError> {
    let url = Url::parse(&start.authorization_url)
        .map_err(|_| VoiceError::new(VoiceErrorCode::InvalidResponse))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || start.provider_id != provider_id
        || start.callback_scheme != "islemap-thienvyma"
        || start.expires_at_ms != expires_at_ms
    {
        return Err(VoiceError::new(VoiceErrorCode::InvalidResponse));
    }
    Ok(())
}

fn parse_callback(callback_url: &str) -> Result<VoiceAuthCallback, VoiceError> {
    let url =
        Url::parse(callback_url).map_err(|_| VoiceError::new(VoiceErrorCode::InvalidCallback))?;
    if url.scheme() != "islemap-thienvyma"
        || url.host_str() != Some("voice")
        || url.path() != "/callback"
        || url.fragment().is_some()
    {
        return Err(VoiceError::new(VoiceErrorCode::InvalidCallback));
    }
    let value = |name: &str| {
        let values: Vec<_> = url
            .query_pairs()
            .filter(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
            .collect();
        (values.len() == 1 && !values[0].trim().is_empty()).then(|| values[0].clone())
    };
    Ok(VoiceAuthCallback {
        provider_id: value("provider")
            .ok_or_else(|| VoiceError::new(VoiceErrorCode::InvalidCallback))?,
        code: value("code").ok_or_else(|| VoiceError::new(VoiceErrorCode::InvalidCallback))?,
        state: value("state").ok_or_else(|| VoiceError::new(VoiceErrorCode::InvalidCallback))?,
    })
}

fn validate_session(
    session: &VoiceSession,
    pending: &PendingLogin,
    now_ms: i64,
) -> Result<(), VoiceError> {
    if session.provider_id != pending.provider_id
        || session.origin != pending.context.origin
        || session.access_token.trim().is_empty()
        || session.expires_at_ms.is_some_and(|expiry| expiry <= now_ms)
    {
        return Err(VoiceError::new(VoiceErrorCode::InvalidResponse));
    }
    Ok(())
}

fn validate_grant(grant: &VoiceGrant, now_ms: i64) -> Result<(), VoiceError> {
    let url = Url::parse(&grant.server_url)
        .map_err(|_| VoiceError::new(VoiceErrorCode::InvalidResponse))?;
    if !matches!(url.scheme(), "wss" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || grant.access_token.trim().is_empty()
        || grant.room.trim().is_empty()
        || grant.identity.trim().is_empty()
        || grant.expires_at_ms <= now_ms
    {
        return Err(VoiceError::new(VoiceErrorCode::InvalidResponse));
    }
    Ok(())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

fn state_for_error(code: VoiceErrorCode) -> VoiceState {
    match code {
        VoiceErrorCode::LoginRequired => VoiceState::LoginRequired,
        VoiceErrorCode::ProviderUnavailable => VoiceState::Unavailable,
        VoiceErrorCode::ClientAuthenticationRequired => VoiceState::Blocked,
        _ => VoiceState::Error,
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use super::*;
    use crate::voice::{
        model::{LoginSecurity, VoiceAuthMethod, VoiceProviderDescriptor, VoiceSessionSummary},
        provider::VoiceProvider,
    };

    const NOW: i64 = 1_000_000;
    const ORIGIN: &str = "https://server.example.test";
    const PROVIDER_ID: &str = "synthetic-voice";

    struct MemorySessions(Mutex<HashMap<(String, String), VoiceSession>>);

    impl MemorySessions {
        fn empty() -> Self {
            Self(Mutex::new(HashMap::new()))
        }
    }

    impl VoiceSessionRepository for MemorySessions {
        fn get(&self, provider_id: &str, origin: &str) -> Option<VoiceSession> {
            self.0
                .lock()
                .unwrap()
                .get(&(provider_id.to_string(), origin.to_string()))
                .cloned()
        }

        fn set(&self, session: VoiceSession) -> Result<(), VoiceError> {
            self.0.lock().unwrap().insert(
                (session.provider_id.clone(), session.origin.clone()),
                session,
            );
            Ok(())
        }

        fn remove(&self, provider_id: &str, origin: &str) -> Result<(), VoiceError> {
            self.0
                .lock()
                .unwrap()
                .remove(&(provider_id.to_string(), origin.to_string()));
            Ok(())
        }
    }

    struct SyntheticProvider;

    impl VoiceProvider for SyntheticProvider {
        fn descriptor(&self) -> VoiceProviderDescriptor {
            VoiceProviderDescriptor {
                id: PROVIDER_ID.to_string(),
                display_name: "Synthetic Voice".to_string(),
                server_origins: vec![ORIGIN.to_string()],
                auth_method: VoiceAuthMethod::AuthorizationCodePkce,
            }
        }

        fn start_login(
            &self,
            _context: &VoiceServerContext,
            security: &LoginSecurity,
            expires_at_ms: i64,
        ) -> Result<VoiceAuthStart, VoiceError> {
            Ok(VoiceAuthStart {
                provider_id: PROVIDER_ID.to_string(),
                authorization_url: format!(
                    "https://auth.example.test/start?state={}",
                    security.state
                ),
                callback_scheme: "islemap-thienvyma".to_string(),
                expires_at_ms,
            })
        }

        fn finish_login(
            &self,
            context: &VoiceServerContext,
            callback: &VoiceAuthCallback,
            _security: &LoginSecurity,
        ) -> Result<VoiceSession, VoiceError> {
            if callback.code != "valid-code" {
                return Err(VoiceError::new(VoiceErrorCode::InvalidCallback));
            }
            Ok(VoiceSession {
                provider_id: PROVIDER_ID.to_string(),
                origin: context.origin.clone(),
                player_name: Some("Player".to_string()),
                steam_id: Some("76561198000000000".to_string()),
                expires_at_ms: Some(NOW + 60_000),
                access_token: "synthetic-session-secret".to_string(),
            })
        }

        fn request_grant(
            &self,
            _context: &VoiceServerContext,
            _session: &VoiceSession,
        ) -> Result<VoiceGrant, VoiceError> {
            Ok(VoiceGrant {
                server_url: "wss://voice.example.test".to_string(),
                access_token: "short-lived-grant".to_string(),
                room: "room-one".to_string(),
                identity: "player-one".to_string(),
                expires_at_ms: NOW + 30_000,
            })
        }
    }

    fn context(origin: &str) -> VoiceServerContext {
        VoiceServerContext {
            origin: origin.to_string(),
            server_id: Some("server-1".to_string()),
            server_name: Some("Test Server".to_string()),
        }
    }

    fn manager() -> (VoiceManager, Arc<MemorySessions>) {
        let sessions = Arc::new(MemorySessions::empty());
        let provider: Arc<dyn VoiceProvider> = Arc::new(SyntheticProvider);
        let registry = VoiceRegistry::try_new(vec![provider]).unwrap();
        (VoiceManager::new(registry, sessions.clone()), sessions)
    }

    fn callback(start: &VoiceAuthStart, state: &str) -> String {
        format!(
            "{CALLBACK_URI}?provider={PROVIDER_ID}&code=valid-code&state={}",
            if state.is_empty() {
                reqwest::Url::parse(&start.authorization_url)
                    .unwrap()
                    .query_pairs()
                    .find(|(key, _)| key == "state")
                    .unwrap()
                    .1
                    .into_owned()
            } else {
                state.to_string()
            }
        )
    }

    #[test]
    fn empty_or_unknown_server_is_not_configured() {
        let sessions = Arc::new(MemorySessions::empty());
        let manager = VoiceManager::new(VoiceRegistry::empty(), sessions);
        assert_eq!(
            manager.status_at(None, NOW).state,
            VoiceState::NotConfigured
        );
        assert_eq!(
            manager
                .status_at(Some(context("https://unknown.example.test")), NOW)
                .state,
            VoiceState::NotConfigured
        );
    }

    #[test]
    fn login_callback_is_bound_to_state_provider_and_single_use() {
        let (manager, _) = manager();
        assert_eq!(
            manager.status_at(Some(context(ORIGIN)), NOW).state,
            VoiceState::LoginRequired
        );
        let start = manager.start_login_at(context(ORIGIN), NOW).unwrap();
        assert_eq!(
            manager.status_at(Some(context(ORIGIN)), NOW).state,
            VoiceState::Authorizing
        );
        assert_eq!(
            manager
                .handle_callback_at(&callback(&start, "wrong-state"), NOW)
                .unwrap_err()
                .code,
            VoiceErrorCode::InvalidCallback
        );
        manager
            .handle_callback_at(&callback(&start, ""), NOW)
            .unwrap();
        assert_eq!(
            manager.status_at(Some(context(ORIGIN)), NOW).state,
            VoiceState::Ready
        );
        assert_eq!(
            manager
                .handle_callback_at(&callback(&start, ""), NOW)
                .unwrap_err()
                .code,
            VoiceErrorCode::InvalidCallback
        );
    }

    #[test]
    fn expired_and_wrong_provider_callbacks_are_rejected() {
        let (manager, _) = manager();
        let start = manager.start_login_at(context(ORIGIN), NOW).unwrap();
        let wrong_provider = callback(&start, "").replace(PROVIDER_ID, "other-provider");
        assert_eq!(
            manager
                .handle_callback_at(&wrong_provider, NOW)
                .unwrap_err()
                .code,
            VoiceErrorCode::InvalidCallback
        );
        assert_eq!(
            manager
                .handle_callback_at(&callback(&start, ""), NOW + LOGIN_TTL_MS + 1)
                .unwrap_err()
                .code,
            VoiceErrorCode::LoginExpired
        );
    }

    #[test]
    fn expired_session_is_removed_and_requires_login() {
        let (manager, sessions) = manager();
        sessions
            .set(VoiceSession {
                provider_id: PROVIDER_ID.to_string(),
                origin: ORIGIN.to_string(),
                player_name: None,
                steam_id: None,
                expires_at_ms: Some(NOW - 1),
                access_token: "expired".to_string(),
            })
            .unwrap();
        assert_eq!(
            manager.status_at(Some(context(ORIGIN)), NOW).state,
            VoiceState::LoginRequired
        );
        assert!(sessions.get(PROVIDER_ID, ORIGIN).is_none());
    }

    #[test]
    fn authenticated_session_can_request_a_valid_normalized_grant() {
        let (manager, _) = manager();
        let start = manager.start_login_at(context(ORIGIN), NOW).unwrap();
        manager
            .handle_callback_at(&callback(&start, ""), NOW)
            .unwrap();
        let grant = manager.request_grant_at(context(ORIGIN), NOW).unwrap();
        assert_eq!(grant.server_url, "wss://voice.example.test");
        assert_eq!(grant.room, "room-one");
        assert!(grant.expires_at_ms > NOW);
    }

    #[test]
    fn logout_removes_only_the_current_provider_session() {
        let (manager, sessions) = manager();
        let start = manager.start_login_at(context(ORIGIN), NOW).unwrap();
        manager
            .handle_callback_at(&callback(&start, ""), NOW)
            .unwrap();
        manager.logout(context(ORIGIN)).unwrap();
        assert!(sessions.get(PROVIDER_ID, ORIGIN).is_none());
        assert_eq!(
            manager.status_at(Some(context(ORIGIN)), NOW).state,
            VoiceState::LoginRequired
        );
    }

    #[test]
    fn public_errors_and_session_summaries_are_redacted() {
        let error = VoiceError::new(VoiceErrorCode::InvalidCallback).to_string();
        assert_eq!(error, "invalid-callback");
        let session = VoiceSession {
            provider_id: PROVIDER_ID.to_string(),
            origin: ORIGIN.to_string(),
            player_name: Some("Player".to_string()),
            steam_id: None,
            expires_at_ms: None,
            access_token: "must-not-leak".to_string(),
        };
        let json = serde_json::to_string(&VoiceSessionSummary::from(&session)).unwrap();
        assert!(!json.contains("must-not-leak"));
    }
}
