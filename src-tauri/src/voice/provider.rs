use std::fmt;

use serde::Serialize;

use super::model::{
    LoginSecurity, VoiceAuthCallback, VoiceAuthStart, VoiceGrant, VoiceProviderDescriptor,
    VoiceServerContext, VoiceSession, VoiceSessionSummary,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceErrorCode {
    InvalidOrigin,
    InvalidProvider,
    DuplicateProvider,
    NotConfigured,
    LoginRequired,
    InvalidCallback,
    LoginExpired,
    ProviderUnavailable,
    StorageUnavailable,
    ClientAuthenticationRequired,
    InvalidResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceError {
    pub code: VoiceErrorCode,
}

impl VoiceError {
    pub const fn new(code: VoiceErrorCode) -> Self {
        Self { code }
    }

    pub const fn as_str(&self) -> &'static str {
        match self.code {
            VoiceErrorCode::InvalidOrigin => "invalid-origin",
            VoiceErrorCode::InvalidProvider => "invalid-provider",
            VoiceErrorCode::DuplicateProvider => "duplicate-provider",
            VoiceErrorCode::NotConfigured => "not-configured",
            VoiceErrorCode::LoginRequired => "login-required",
            VoiceErrorCode::InvalidCallback => "invalid-callback",
            VoiceErrorCode::LoginExpired => "login-expired",
            VoiceErrorCode::ProviderUnavailable => "provider-unavailable",
            VoiceErrorCode::StorageUnavailable => "storage-unavailable",
            VoiceErrorCode::ClientAuthenticationRequired => "client-authentication-required",
            VoiceErrorCode::InvalidResponse => "invalid-response",
        }
    }
}

impl fmt::Display for VoiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::error::Error for VoiceError {}

pub trait VoiceProvider: Send + Sync {
    fn descriptor(&self) -> VoiceProviderDescriptor;

    fn start_login(
        &self,
        context: &VoiceServerContext,
        security: &LoginSecurity,
        expires_at_ms: i64,
    ) -> Result<VoiceAuthStart, VoiceError>;

    fn finish_login(
        &self,
        context: &VoiceServerContext,
        callback: &VoiceAuthCallback,
        security: &LoginSecurity,
    ) -> Result<VoiceSession, VoiceError>;

    fn request_grant(
        &self,
        context: &VoiceServerContext,
        session: &VoiceSession,
    ) -> Result<VoiceGrant, VoiceError>;

    fn session_status(
        &self,
        _context: &VoiceServerContext,
        session: &VoiceSession,
    ) -> Result<VoiceSessionSummary, VoiceError> {
        Ok(VoiceSessionSummary::from(session))
    }

    fn logout(
        &self,
        _context: &VoiceServerContext,
        _session: &VoiceSession,
    ) -> Result<(), VoiceError> {
        Ok(())
    }
}
