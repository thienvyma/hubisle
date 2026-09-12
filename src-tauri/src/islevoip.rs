//! Official IsleVOIP desktop launcher integration.
//!
//! This module only locates and starts the official GUI. Running-state
//! detection uses a read-only Toolhelp process snapshot and never opens a
//! process handle or interacts with IsleVOIP's private protocol.

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

use serde::Serialize;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

const EXECUTABLE_NAME: &str = "IsleVOIP.exe";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsleVoipStatus {
    pub installed: bool,
    pub running: bool,
    pub executable_path: Option<String>,
}

fn candidate_paths_from_env(
    local_app_data: Option<&OsStr>,
    program_files: Option<&OsStr>,
    program_files_x86: Option<&OsStr>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(root) = local_app_data {
        let root = PathBuf::from(root);
        candidates.push(
            root.join("Programs")
                .join("isle-voip-overlay")
                .join(EXECUTABLE_NAME),
        );
        candidates.push(root.join("Programs").join("IsleVOIP").join(EXECUTABLE_NAME));
        candidates.push(root.join("isle-voip-overlay").join(EXECUTABLE_NAME));
    }

    for root in [program_files, program_files_x86].into_iter().flatten() {
        let root = PathBuf::from(root);
        candidates.push(root.join("isle-voip-overlay").join(EXECUTABLE_NAME));
        candidates.push(root.join("IsleVOIP").join(EXECUTABLE_NAME));
    }

    candidates
}

fn executable_candidates() -> Vec<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA");
    let program_files = std::env::var_os("ProgramFiles");
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)");
    candidate_paths_from_env(
        local_app_data.as_deref(),
        program_files.as_deref(),
        program_files_x86.as_deref(),
    )
}

fn first_installed_candidate(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|path| path.is_file()).cloned()
}

fn is_islevoip_process_name(name: &str) -> bool {
    name.eq_ignore_ascii_case(EXECUTABLE_NAME)
}

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&ch| ch == 0).unwrap_or(buf.len());
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
                if is_islevoip_process_name(&wide_to_string(&entry.szExeFile)) {
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

pub fn status() -> IsleVoipStatus {
    let executable = first_installed_candidate(&executable_candidates());
    IsleVoipStatus {
        installed: executable.is_some(),
        running: is_running(),
        executable_path: executable.map(|path| path.to_string_lossy().into_owned()),
    }
}

pub fn launch() -> Result<IsleVoipStatus, String> {
    let current = status();
    if current.running {
        return Ok(current);
    }

    let executable = current
        .executable_path
        .as_ref()
        .ok_or_else(|| "IsleVOIP is not installed".to_string())?;
    Command::new(executable)
        .spawn()
        .map_err(|error| format!("failed to launch IsleVOIP: {error}"))?;

    Ok(IsleVoipStatus {
        running: true,
        ..current
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn standard_per_user_install_path_has_precedence() {
        let candidates = candidate_paths_from_env(
            Some(OsStr::new(r"C:\Users\player\AppData\Local")),
            Some(OsStr::new(r"C:\Program Files")),
            Some(OsStr::new(r"C:\Program Files (x86)")),
        );

        assert_eq!(
            candidates.first(),
            Some(&PathBuf::from(
                r"C:\Users\player\AppData\Local\Programs\isle-voip-overlay\IsleVOIP.exe"
            ))
        );
        assert!(
            candidates
                .iter()
                .position(|path| path == &PathBuf::from(r"C:\Program Files\IsleVOIP\IsleVOIP.exe"))
                .unwrap()
                > 0
        );
    }

    #[test]
    fn executable_discovery_selects_first_existing_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "islemap-thienvyma-islevoip-{}-{unique}",
            std::process::id()
        ));
        let missing = root.join("missing").join("IsleVOIP.exe");
        let preferred = root.join("preferred").join("IsleVOIP.exe");
        let fallback = root.join("fallback").join("IsleVOIP.exe");
        fs::create_dir_all(preferred.parent().unwrap()).unwrap();
        fs::create_dir_all(fallback.parent().unwrap()).unwrap();
        fs::write(&preferred, b"").unwrap();
        fs::write(&fallback, b"").unwrap();

        let selected = first_installed_candidate(&[missing, preferred.clone(), fallback]);

        fs::remove_dir_all(&root).unwrap();
        assert_eq!(selected, Some(preferred));
    }

    #[test]
    fn process_name_matching_is_exact_and_case_insensitive() {
        assert!(is_islevoip_process_name("IsleVOIP.exe"));
        assert!(is_islevoip_process_name("islevoip.EXE"));
        assert!(!is_islevoip_process_name("IsleVOIP-helper.exe"));
        assert!(!is_islevoip_process_name(
            r"C:\Program Files\IsleVOIP\IsleVOIP.exe"
        ));
    }

    #[test]
    #[ignore = "requires the official IsleVOIP desktop app"]
    fn installed_launcher_smoke() {
        let status = status();
        let executable = status
            .executable_path
            .as_deref()
            .expect("official IsleVOIP installation was not found");

        assert!(status.installed);
        assert!(PathBuf::from(executable).is_file());
        assert_eq!(
            PathBuf::from(executable).file_name(),
            Some(OsStr::new(EXECUTABLE_NAME))
        );
    }
}
