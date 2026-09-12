//! Bridge to the official DinoVietnam Vietnamese translation launcher.
//!
//! The translation package is owned and distributed by DinoVietnam. We do
//! not copy its encrypted archive or embedded installer. When the official
//! hub has installed the launcher on this computer, this module validates the
//! local executable shape and starts that exact copy from our UI.

use std::{path::PathBuf, process::Command};

use serde::Serialize;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const OFFICIAL_RELEASE_URL: &str =
    "https://github.com/ricktanker/dinovietnam-overlay/releases/tag/viethoa";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VietHoaStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub executable_path: Option<String>,
    pub official_release_url: String,
}

fn install_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|root| root.join("dinovietnam-hud").join("viethoa"))
}

fn executable_path() -> Option<PathBuf> {
    install_dir().map(|dir| dir.join("The_Isle_VH.exe"))
}

fn looks_like_windows_executable(path: &std::path::Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() || metadata.len() < 256 * 1024 {
        return false;
    }
    std::fs::read(path)
        .map(|bytes| bytes.starts_with(b"MZ"))
        .unwrap_or(false)
}

fn status() -> VietHoaStatus {
    let executable = executable_path().filter(|path| looks_like_windows_executable(path));
    let version = install_dir()
        .and_then(|dir| std::fs::read_to_string(dir.join("version.txt")).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value.len() <= 64);
    VietHoaStatus {
        installed: executable.is_some(),
        version,
        executable_path: executable.map(|path| path.to_string_lossy().into_owned()),
        official_release_url: OFFICIAL_RELEASE_URL.to_string(),
    }
}

#[tauri::command]
pub fn viethoa_status() -> VietHoaStatus {
    status()
}

#[tauri::command]
pub fn viethoa_launch() -> Result<VietHoaStatus, String> {
    let path = executable_path().ok_or_else(|| "viethoa-not-installed".to_string())?;
    if !looks_like_windows_executable(&path) {
        return Err("viethoa-not-installed".to_string());
    }
    let mut command = Command::new(&path);
    if let Some(parent) = path.parent() {
        command.current_dir(parent);
    }
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
        .spawn()
        .map_err(|_| "viethoa-launch-failed".to_string())?;
    Ok(status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_release_uses_encrypted_transport() {
        assert!(OFFICIAL_RELEASE_URL.starts_with("https://github.com/ricktanker/"));
    }

    #[test]
    fn tiny_or_missing_files_are_not_executables() {
        let path = std::env::temp_dir().join(format!("viethoa-{}.exe", std::process::id()));
        std::fs::write(&path, b"MZ").unwrap();
        assert!(!looks_like_windows_executable(&path));
        std::fs::remove_file(path).unwrap();
    }
}
