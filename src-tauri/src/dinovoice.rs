//! Safe bridge to DinoVietnam's official Voice client.
//!
//! DinoVietnam issues LiveKit access only to its registered desktop client.
//! The hub therefore detects and opens that client, while never reading its
//! encrypted token or copying the private service credential into this app.

use std::{os::windows::process::CommandExt, path::PathBuf, process::Command};

use serde::Serialize;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

pub const OFFICIAL_RELEASE_URL: &str =
    "https://github.com/ricktanker/dinovietnam-overlay/releases/latest";
const EXECUTABLE_NAME: &str = "DinoVietNam.exe";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DinoVoiceStatus {
    pub installed: bool,
    pub running: bool,
    pub steam_authenticated: bool,
    pub executable_path: Option<String>,
    pub official_release_url: String,
}

fn looks_like_official_client(path: &std::path::Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file()
        || metadata.len() < 1_000_000
        || !path
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case(EXECUTABLE_NAME))
    {
        return false;
    }
    std::fs::read(path)
        .map(|bytes| bytes.starts_with(b"MZ"))
        .unwrap_or(false)
}

fn command_path(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    let path = if let Some(rest) = value.strip_prefix('"') {
        let end = rest.find('"')?;
        &rest[..end]
    } else {
        value.split_whitespace().next()?
    };
    let path = PathBuf::from(path);
    looks_like_official_client(&path).then_some(path)
}

fn protocol_executable() -> Option<PathBuf> {
    let output = Command::new("reg.exe")
        .args([
            "query",
            r"HKCU\Software\Classes\dinovietnam\shell\open\command",
            "/ve",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    text.lines().find_map(|line| {
        line.split_once("REG_SZ")
            .and_then(|(_, value)| command_path(value))
    })
}

fn fallback_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(root) = std::env::var_os("OneDrive") {
        let root = PathBuf::from(root);
        paths.push(root.join("Máy tính").join(EXECUTABLE_NAME));
        paths.push(root.join("Desktop").join(EXECUTABLE_NAME));
    }
    if let Some(root) = std::env::var_os("USERPROFILE") {
        paths.push(PathBuf::from(root).join("Desktop").join(EXECUTABLE_NAME));
    }
    paths
}

fn executable() -> Option<PathBuf> {
    protocol_executable().or_else(|| {
        fallback_candidates()
            .into_iter()
            .find(|path| looks_like_official_client(path))
    })
}

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf
        .iter()
        .position(|&value| value == 0)
        .unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

fn is_running() -> bool {
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return false;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut found = false;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if wide_to_string(&entry.szExeFile).eq_ignore_ascii_case(EXECUTABLE_NAME) {
                    found = true;
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        found
    }
}

fn steam_authenticated() -> bool {
    let Some(path) = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|root| root.join("dinovietnam-hud").join("settings.json"))
    else {
        return false;
    };
    let Some(settings) = std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
    else {
        return false;
    };
    let present = |key: &str| {
        settings
            .get(key)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
    };
    // Presence only: the encrypted session and SteamID are never returned,
    // decrypted, logged, or copied into islemap-thienvyma.
    present("overlayTokenEnc") && present("overlaySteamId")
}

fn status() -> DinoVoiceStatus {
    let executable = executable();
    DinoVoiceStatus {
        installed: executable.is_some(),
        running: is_running(),
        steam_authenticated: steam_authenticated(),
        executable_path: executable.map(|path| path.to_string_lossy().into_owned()),
        official_release_url: OFFICIAL_RELEASE_URL.to_string(),
    }
}

#[tauri::command]
pub fn dinovoice_status() -> DinoVoiceStatus {
    status()
}

#[tauri::command]
pub fn dinovoice_launch() -> Result<DinoVoiceStatus, String> {
    let current = status();
    let path = current
        .executable_path
        .as_deref()
        .map(PathBuf::from)
        .filter(|path| looks_like_official_client(path))
        .ok_or_else(|| "dinovoice-not-installed".to_string())?;

    if current.running {
        // Let the registered protocol activate the existing single instance.
        Command::new("explorer.exe")
            .arg("dinovietnam:")
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|_| "dinovoice-launch-failed".to_string())?;
    } else {
        Command::new(&path)
            .current_dir(path.parent().unwrap_or_else(|| std::path::Path::new(".")))
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|_| "dinovoice-launch-failed".to_string())?;
    }
    Ok(DinoVoiceStatus {
        running: true,
        ..current
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_quoted_protocol_executable() {
        let path = std::env::temp_dir().join(EXECUTABLE_NAME);
        let mut bytes = vec![0_u8; 1_000_001];
        bytes[0..2].copy_from_slice(b"MZ");
        std::fs::write(&path, bytes).unwrap();
        let parsed = command_path(&format!(r#""{}" "%1""#, path.display()));
        std::fs::remove_file(&path).unwrap();
        assert_eq!(parsed, Some(path));
    }

    #[test]
    fn official_release_is_https() {
        assert!(OFFICIAL_RELEASE_URL.starts_with("https://github.com/ricktanker/"));
    }
}
