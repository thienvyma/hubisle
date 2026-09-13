use std::fmt;

use serde::Serialize;

use super::model::{
    LoginSecurity, VoiceAuthCallback, VoiceAuthStart, VoiceGrant, VoiceProviderDescriptor,
    VoiceServerContext, VoiceSession,
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
}

impl fmt::Display for VoiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", serde_json::to_value(self.code).unwrap())
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

    fn logout(
        &self,
        _context: &VoiceServerContext,
        _session: &VoiceSession,
    ) -> Result<(), VoiceError> {
        Ok(())
    }
}
