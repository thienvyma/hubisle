use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use reqwest::Url;

use super::provider::{VoiceError, VoiceErrorCode, VoiceProvider};

pub struct VoiceRegistry {
    providers: Vec<Arc<dyn VoiceProvider>>,
    by_origin: HashMap<String, usize>,
}

pub fn normalize_origin(input: &str) -> Result<String, VoiceError> {
    let url =
        Url::parse(input.trim()).map_err(|_| VoiceError::new(VoiceErrorCode::InvalidOrigin))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(VoiceError::new(VoiceErrorCode::InvalidOrigin));
    }
    Ok(url.origin().ascii_serialization())
}

impl VoiceRegistry {
    pub fn empty() -> Self {
        Self {
            providers: Vec::new(),
            by_origin: HashMap::new(),
        }
    }

    pub fn try_new(providers: Vec<Arc<dyn VoiceProvider>>) -> Result<Self, VoiceError> {
        let mut ids = HashSet::new();
        let mut by_origin = HashMap::new();
        for (index, provider) in providers.iter().enumerate() {
            let descriptor = provider.descriptor();
            if descriptor.id.trim().is_empty() || !ids.insert(descriptor.id) {
                return Err(VoiceError::new(VoiceErrorCode::DuplicateProvider));
            }
            for raw_origin in descriptor.server_origins {
                let origin = normalize_origin(&raw_origin)?;
                if by_origin.insert(origin, index).is_some() {
                    return Err(VoiceError::new(VoiceErrorCode::DuplicateProvider));
                }
            }
        }
        Ok(Self {
            providers,
            by_origin,
        })
    }

    pub fn find(&self, origin: &str) -> Result<Option<Arc<dyn VoiceProvider>>, VoiceError> {
        let origin = normalize_origin(origin)?;
        Ok(self
            .by_origin
            .get(&origin)
            .map(|index| Arc::clone(&self.providers[*index])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice::{
        model::{
            LoginSecurity, VoiceAuthCallback, VoiceAuthMethod, VoiceAuthStart, VoiceGrant,
            VoiceProviderDescriptor, VoiceServerContext, VoiceSession,
        },
        provider::VoiceError,
    };

    struct FakeProvider {
        id: &'static str,
        origins: &'static [&'static str],
    }

    impl VoiceProvider for FakeProvider {
        fn descriptor(&self) -> VoiceProviderDescriptor {
            VoiceProviderDescriptor {
                id: self.id.to_string(),
                display_name: self.id.to_string(),
                server_origins: self.origins.iter().map(|value| value.to_string()).collect(),
                auth_method: VoiceAuthMethod::AuthorizationCodePkce,
            }
        }

        fn start_login(
            &self,
            _context: &VoiceServerContext,
            _security: &LoginSecurity,
            _expires_at_ms: i64,
        ) -> Result<VoiceAuthStart, VoiceError> {
            unreachable!()
        }

        fn finish_login(
            &self,
            _context: &VoiceServerContext,
            _callback: &VoiceAuthCallback,
            _security: &LoginSecurity,
        ) -> Result<VoiceSession, VoiceError> {
            unreachable!()
        }

        fn request_grant(
            &self,
            _context: &VoiceServerContext,
            _session: &VoiceSession,
        ) -> Result<VoiceGrant, VoiceError> {
            unreachable!()
        }
    }

    fn provider(id: &'static str, origins: &'static [&'static str]) -> Arc<dyn VoiceProvider> {
        Arc::new(FakeProvider { id, origins })
    }

    #[test]
    fn normalizes_an_https_url_to_its_exact_origin() {
        assert_eq!(
            normalize_origin(" HTTPS://Voice.Example.test/path?q=1#fragment ").unwrap(),
            "https://voice.example.test"
        );
        assert_eq!(
            normalize_origin("https://voice.example.test:8443/path").unwrap(),
            "https://voice.example.test:8443"
        );
    }

    #[test]
    fn rejects_unsafe_or_ambiguous_origins() {
        for invalid in [
            "http://voice.example.test",
            "https://user:pass@voice.example.test",
            "https://",
            "voice.example.test",
        ] {
            assert_eq!(
                normalize_origin(invalid).unwrap_err().code,
                VoiceErrorCode::InvalidOrigin
            );
        }
    }

    #[test]
    fn registry_matches_only_the_exact_normalized_origin() {
        let registry = VoiceRegistry::try_new(vec![provider(
            "server-one",
            &["https://voice.example.test/panel"],
        )])
        .unwrap();

        assert_eq!(
            registry
                .find("https://voice.example.test/map")
                .unwrap()
                .unwrap()
                .descriptor()
                .id,
            "server-one"
        );
        assert!(registry
            .find("https://voice.example.test.attacker.invalid")
            .unwrap()
            .is_none());
        assert!(registry
            .find("https://sub.voice.example.test")
            .unwrap()
            .is_none());
    }

    #[test]
    fn registry_rejects_duplicate_ids_and_origins() {
        assert_eq!(
            VoiceRegistry::try_new(vec![
                provider("same", &["https://one.example.test"]),
                provider("same", &["https://two.example.test"]),
            ])
            .err()
            .unwrap()
            .code,
            VoiceErrorCode::DuplicateProvider
        );
        assert_eq!(
            VoiceRegistry::try_new(vec![
                provider("one", &["https://same.example.test/a"]),
                provider("two", &["https://same.example.test/b"]),
            ])
            .err()
            .unwrap()
            .code,
            VoiceErrorCode::DuplicateProvider
        );
    }

    #[test]
    fn empty_registry_is_a_valid_not_configured_state() {
        assert!(VoiceRegistry::empty()
            .find("https://voice.example.test")
            .unwrap()
            .is_none());
    }
}
