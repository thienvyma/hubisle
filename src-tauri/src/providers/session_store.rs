use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    islepilot::cookies::{dpapi_protect, dpapi_unprotect},
    settings,
};

use super::model::ProviderId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct SessionRecord {
    provider: ProviderId,
    origin: String,
    cookie: String,
}

fn store_path() -> PathBuf {
    settings::local_dir().join("provider_sessions.bin")
}

fn load_records() -> Vec<SessionRecord> {
    let Ok(sealed) = std::fs::read(store_path()) else {
        return Vec::new();
    };
    dpapi_unprotect(&sealed)
        .ok()
        .and_then(|plain| serde_json::from_slice(&plain).ok())
        .unwrap_or_default()
}

fn save_records(records: &[SessionRecord]) -> Result<(), String> {
    let plain = serde_json::to_vec(records).map_err(|_| "Không thể mã hóa phiên đăng nhập.")?;
    let sealed = dpapi_protect(&plain)?;
    let path = store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temp = path.with_extension("bin.tmp");
    std::fs::write(&temp, sealed).map_err(|e| e.to_string())?;
    std::fs::rename(temp, path).map_err(|e| e.to_string())
}

fn find_cookie<'a>(
    records: &'a [SessionRecord],
    provider: ProviderId,
    origin: &str,
) -> Option<&'a str> {
    records
        .iter()
        .find(|record| record.provider == provider && record.origin == origin)
        .map(|record| record.cookie.as_str())
}

pub struct ProviderSessionStore;

impl ProviderSessionStore {
    pub fn get(provider: ProviderId, origin: &str) -> Option<String> {
        find_cookie(&load_records(), provider, origin).map(str::to_string)
    }

    pub fn set(provider: ProviderId, origin: &str, cookie: &str) -> Result<(), String> {
        let mut records = load_records();
        if let Some(record) = records
            .iter_mut()
            .find(|record| record.provider == provider && record.origin == origin)
        {
            record.cookie = cookie.to_string();
        } else {
            records.push(SessionRecord {
                provider,
                origin: origin.to_string(),
                cookie: cookie.to_string(),
            });
        }
        save_records(&records)
    }

    pub fn remove(provider: ProviderId, origin: &str) -> Result<(), String> {
        let mut records = load_records();
        records.retain(|record| record.provider != provider || record.origin != origin);
        save_records(&records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_are_bound_to_provider_and_exact_origin() {
        let records = vec![SessionRecord {
            provider: ProviderId::Era,
            origin: "https://eragamingvn.net".to_string(),
            cookie: "synthetic=session".to_string(),
        }];
        assert_eq!(
            find_cookie(&records, ProviderId::Era, "https://eragamingvn.net"),
            Some("synthetic=session")
        );
        assert_eq!(
            find_cookie(&records, ProviderId::Titan, "https://eragamingvn.net"),
            None
        );
        assert_eq!(
            find_cookie(&records, ProviderId::Era, "https://www.eragamingvn.net"),
            None
        );
    }
}
