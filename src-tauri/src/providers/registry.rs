use reqwest::Url;

use super::model::ProviderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedProvider {
    pub id: ProviderId,
    pub origin: String,
}

pub fn detect_provider(input: &str) -> Result<DetectedProvider, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Hãy nhập website của server.".to_string());
    }

    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let url = Url::parse(&candidate).map_err(|_| "Địa chỉ website không hợp lệ.".to_string())?;

    if url.scheme() != "https" {
        return Err("Website phải sử dụng HTTPS.".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("Không nhập tài khoản hoặc mật khẩu trong địa chỉ website.".to_string());
    }
    if url.port().is_some() {
        return Err("Website có cổng tùy chỉnh chưa được hỗ trợ.".to_string());
    }

    let host = url
        .host_str()
        .ok_or_else(|| "Địa chỉ website không có tên miền.".to_string())?
        .to_ascii_lowercase();
    let id = match host.as_str() {
        "eragamingvn.net" => ProviderId::Era,
        "therealservervn.com" | "www.therealservervn.com" => ProviderId::Titan,
        "islepilot.eu" => ProviderId::IslePilot,
        _ if host.ends_with(".islepilot.eu") => ProviderId::IslePilot,
        _ => return Err("Server này chưa có bộ kết nối tương thích.".to_string()),
    };

    Ok(DetectedProvider {
        id,
        origin: format!("https://{host}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_verified_origins_and_rejects_lookalikes() {
        assert_eq!(
            detect_provider("eragamingvn.net/live-map").unwrap().id,
            ProviderId::Era
        );
        assert_eq!(
            detect_provider("https://www.therealservervn.com/nguoi-choi")
                .unwrap()
                .id,
            ProviderId::Titan
        );
        assert_eq!(
            detect_provider("https://mixi.islepilot.eu").unwrap().id,
            ProviderId::IslePilot
        );
        assert!(detect_provider("https://eragamingvn.net.example.org").is_err());
        assert!(detect_provider("http://eragamingvn.net").is_err());
        assert!(detect_provider("https://user:pass@eragamingvn.net").is_err());
        assert!(detect_provider("https://eragamingvn.net:8443").is_err());
        assert!(detect_provider("https://fakeislepilot.eu").is_err());
    }
}
