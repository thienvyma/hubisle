use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

const RETOC_VERSION: &str = "0.1.5";
const RETOC_ZIP_URL: &str = "https://github.com/trumank/retoc/releases/download/v0.1.5/retoc_cli-x86_64-pc-windows-msvc.zip";
pub(crate) const RETOC_ZIP_SHA256: &str =
    "cc036b06ad3bdcf7003690b00d82719980c374e48a95bf0654f9959148d263aa";

#[derive(Debug, Clone)]
pub(crate) struct IoStoreLocres {
    pub virtual_path: String,
    pub bytes: Vec<u8>,
    pub utoc_path: PathBuf,
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn escape_powershell_literal(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

fn find_named_file(root: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                stack.push(path);
            } else if kind.is_file()
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case(name))
            {
                return Some(path);
            }
        }
    }
    None
}

fn find_game_oodle(game_root: &Path) -> Option<PathBuf> {
    let candidates = [
        game_root
            .join("TheIsle")
            .join("Binaries")
            .join("Win64")
            .join("oo2core_9_win64.dll"),
        game_root
            .join("Engine")
            .join("Binaries")
            .join("ThirdParty")
            .join("Oodle")
            .join("Win64")
            .join("oo2core_9_win64.dll"),
    ];
    candidates.into_iter().find(|path| path.is_file()).or_else(|| {
        let binaries = game_root.join("TheIsle").join("Binaries");
        find_named_file(&binaries, "oo2core_9_win64.dll")
    })
}

fn ensure_retoc(cache_root: &Path, game_root: &Path) -> Result<PathBuf, String> {
    let tool_root = cache_root.join(format!("retoc-{RETOC_VERSION}"));
    fs::create_dir_all(&tool_root)
        .map_err(|error| format!("Không thể tạo cache công cụ IoStore: {error}"))?;

    if let Some(retoc) = find_named_file(&tool_root, "retoc.exe") {
        ensure_oodle_next_to_retoc(&retoc, game_root)?;
        return Ok(retoc);
    }

    let zip_path = tool_root.join("retoc.zip");
    let bytes = reqwest::blocking::get(RETOC_ZIP_URL)
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Không thể tải công cụ đọc IoStore retoc: {error}"))?
        .bytes()
        .map_err(|error| format!("Không thể đọc gói retoc đã tải: {error}"))?;

    let found_hash = sha256_bytes(&bytes);
    if found_hash != RETOC_ZIP_SHA256 {
        return Err(format!(
            "Gói retoc không đúng checksum. Mong đợi {RETOC_ZIP_SHA256}, nhận {found_hash}."
        ));
    }
    fs::write(&zip_path, &bytes)
        .map_err(|error| format!("Không thể lưu gói retoc: {error}"))?;

    let extract_dir = tool_root.join("bin");
    let _ = fs::remove_dir_all(&extract_dir);
    fs::create_dir_all(&extract_dir)
        .map_err(|error| format!("Không thể tạo thư mục giải nén retoc: {error}"))?;
    let command = format!(
        "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
        escape_powershell_literal(&zip_path),
        escape_powershell_literal(&extract_dir)
    );
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &command,
        ])
        .output()
        .map_err(|error| format!("Không thể chạy PowerShell để giải nén retoc: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Không thể giải nén retoc: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let retoc = find_named_file(&extract_dir, "retoc.exe")
        .ok_or_else(|| "Gói retoc không chứa retoc.exe như mong đợi.".to_string())?;
    ensure_oodle_next_to_retoc(&retoc, game_root)?;
    Ok(retoc)
}

fn ensure_oodle_next_to_retoc(retoc: &Path, game_root: &Path) -> Result<(), String> {
    let Some(dir) = retoc.parent() else {
        return Err("Đường dẫn retoc không hợp lệ.".to_string());
    };
    let target = dir.join("oo2core_9_win64.dll");
    if target.is_file() {
        return Ok(());
    }
    if let Some(source) = find_game_oodle(game_root) {
        fs::copy(&source, &target).map_err(|error| {
            format!(
                "Không thể chuẩn bị Oodle từ {}: {error}",
                source.display()
            )
        })?;
    }
    Ok(())
}

fn run_retoc(retoc: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new(retoc)
        .args(args)
        .output()
        .map_err(|error| format!("Không thể chạy retoc: {error}"))
}

fn parse_list_line(line: &str) -> Option<(String, String)> {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 4 {
        return None;
    }
    let chunk = parts
        .iter()
        .copied()
        .find(|part| part.len() == 24 && part.chars().all(|ch| ch.is_ascii_hexdigit()))?;
    let path = *parts.last()?;
    if path == "-" || !path.to_ascii_lowercase().ends_with(".locres") {
        return None;
    }
    Some((chunk.to_string(), path.to_string()))
}

fn is_english_game_locres(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    lower.ends_with(".locres")
        && lower.contains("theisle/content/localization/")
        && (lower.contains("/en/") || lower.contains("/en-us/"))
}

fn game_paks_root(game_root: &Path) -> PathBuf {
    game_root.join("TheIsle").join("Content").join("Paks")
}

pub(crate) fn destination_for_virtual_source(
    game_root: &Path,
    virtual_path: &str,
) -> Option<PathBuf> {
    let normalized = virtual_path.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    let marker = "theisle/content/";
    let start = lower.find(marker)? + marker.len();
    let relative = &normalized[start..];
    let mut output = PathBuf::new();
    let mut replaced = false;

    for component in relative.split('/') {
        if component.is_empty() || component == "." || component == ".." {
            if component == ".." {
                return None;
            }
            continue;
        }
        if !replaced && (component.eq_ignore_ascii_case("en") || component.eq_ignore_ascii_case("en-US")) {
            output.push("vi");
            replaced = true;
        } else {
            output.push(component);
        }
    }

    replaced.then(|| game_root.join("TheIsle").join("Content").join(output))
}

pub(crate) fn collect_iostore_locres(
    game_root: &Path,
    cache_root: &Path,
) -> Result<Vec<IoStoreLocres>, String> {
    let paks = game_paks_root(game_root);
    let mut utocs = fs::read_dir(&paks)
        .map_err(|error| format!("Không thể đọc {}: {error}", paks.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("utoc"))
                && path.with_extension("ucas").is_file()
        })
        .collect::<Vec<_>>();
    utocs.sort();
    if utocs.is_empty() {
        return Err("Không tìm thấy container IoStore .utoc/.ucas của The Isle.".to_string());
    }

    let retoc = ensure_retoc(cache_root, game_root)?;
    let extract_dir = cache_root.join("iostore-extract");
    fs::create_dir_all(&extract_dir)
        .map_err(|error| format!("Không thể tạo cache localization: {error}"))?;

    let mut found = BTreeMap::<String, IoStoreLocres>::new();
    let mut list_errors = Vec::new();

    for utoc in utocs {
        let utoc_text = utoc.to_string_lossy();
        let list = run_retoc(&retoc, &["list", &utoc_text, "--path"])?;
        if !list.status.success() {
            list_errors.push(format!(
                "{}: {}",
                utoc.file_name().and_then(|value| value.to_str()).unwrap_or("container"),
                String::from_utf8_lossy(&list.stderr).trim()
            ));
            continue;
        }

        let stdout = String::from_utf8_lossy(&list.stdout);
        for line in stdout.lines() {
            let Some((chunk_id, virtual_path)) = parse_list_line(line) else {
                continue;
            };
            if !is_english_game_locres(&virtual_path) {
                continue;
            }

            let temp = extract_dir.join(format!("{chunk_id}.locres"));
            let temp_text = temp.to_string_lossy();
            let get = run_retoc(&retoc, &["get", &utoc_text, &chunk_id, &temp_text])?;
            if !get.status.success() {
                list_errors.push(format!(
                    "Không thể đọc {virtual_path}: {}",
                    String::from_utf8_lossy(&get.stderr).trim()
                ));
                let _ = fs::remove_file(&temp);
                continue;
            }
            let bytes = fs::read(&temp)
                .map_err(|error| format!("Không thể đọc localization đã trích xuất: {error}"))?;
            let _ = fs::remove_file(&temp);
            found.insert(
                virtual_path.to_ascii_lowercase(),
                IoStoreLocres {
                    virtual_path,
                    bytes,
                    utoc_path: utoc.clone(),
                },
            );
        }
    }

    if found.is_empty() && !list_errors.is_empty() {
        return Err(format!(
            "Không đọc được localization trong IoStore của The Isle. {}",
            list_errors.join(" | ")
        ));
    }
    Ok(found.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_retoc_list_line_for_locres() {
        let line = "pakchunk0-WindowsClient 0123456789abcdef01234567 BulkData ../../../TheIsle/Content/Localization/Game/en/Game.locres";
        let (chunk, path) = parse_list_line(line).expect("locres row");
        assert_eq!(chunk, "0123456789abcdef01234567");
        assert!(path.ends_with("Game.locres"));
    }

    #[test]
    fn ignores_non_locres_list_rows() {
        let line = "pakchunk0-WindowsClient 0123456789abcdef01234567 BulkData ../../../TheIsle/Content/UI/Icon.uasset";
        assert!(parse_list_line(line).is_none());
    }

    #[test]
    fn maps_virtual_english_localization_to_loose_vietnamese_target() {
        let root = Path::new(r"C:\Steam\steamapps\common\The Isle");
        let target = destination_for_virtual_source(
            root,
            "../../../TheIsle/Content/Localization/Game/en/Game.locres",
        )
        .expect("target");
        assert_eq!(
            target,
            root.join("TheIsle/Content/Localization/Game/vi/Game.locres")
        );
    }
}
