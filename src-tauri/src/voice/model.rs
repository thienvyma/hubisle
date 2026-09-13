use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceAuthMethod {
    AuthorizationCodePkce,
    ServerSession,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceProviderDescriptor {
    pub id: String,
    pub display_name: String,
    pub server_origins: Vec<String>,
    pub auth_method: VoiceAuthMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceServerContext {
    pub origin: String,
    pub server_id: Option<String>,
    pub server_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginSecurity {
    pub state: String,
    pub nonce: String,
    pub pkce_verifier: String,
    pub callback_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAuthCallback {
    pub provider_id: String,
    pub code: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSession {
    pub provider_id: String,
    pub origin: String,
    pub player_name: Option<String>,
    pub steam_id: Option<String>,
    pub expires_at_ms: Option<i64>,
    pub access_token: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceState {
    NotConfigured,
    LoginRequired,
    Authorizing,
    Ready,
    Unavailable,
    Blocked,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceStatus {
    pub state: VoiceState,
    pub server_origin: Option<String>,
    pub server_name: Option<String>,
    pub provider: Option<VoiceProviderDescriptor>,
    pub session: Option<VoiceSessionSummary>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSessionSummary {
    pub provider_id: String,
    pub player_name: Option<String>,
    pub steam_id: Option<String>,
    pub expires_at_ms: Option<i64>,
}

impl From<&VoiceSession> for VoiceSessionSummary {
    fn from(session: &VoiceSession) -> Self {
        Self {
            provider_id: session.provider_id.clone(),
            player_name: session.player_name.clone(),
            steam_id: session.steam_id.clone(),
            expires_at_ms: session.expires_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAuthStart {
    pub provider_id: String,
    pub authorization_url: String,
    pub callback_scheme: String,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceGrant {
    pub server_url: String,
    pub access_token: String,
    pub room: String,
    pub identity: String,
    pub expires_at_ms: i64,
}
