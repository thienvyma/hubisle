use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const THE_ISLE_APP_ID: &str = "376210";

fn quoted_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }
        let mut token = String::new();
        while let Some(next) = chars.next() {
            match next {
                '"' => break,
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        token.push(match escaped {
                            '\\' => '\\',
                            '"' => '"',
                            other => other,
                        });
                    }
                }
                other => token.push(other),
            }
        }
        tokens.push(token);
    }
    tokens
}

pub(crate) fn parse_steam_library_paths(input: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for line in input.lines() {
        let tokens = quoted_tokens(line);
        if tokens.len() >= 2 && tokens[0].eq_ignore_ascii_case("path") {
            paths.push(PathBuf::from(&tokens[1]));
        }
    }
    paths
}

pub(crate) fn parse_install_dir(input: &str) -> Option<String> {
    for line in input.lines() {
        let tokens = quoted_tokens(line);
        if tokens.len() >= 2 && tokens[0].eq_ignore_ascii_case("installdir") {
            let value = tokens[1].trim();
            if !value.is_empty() && !value.contains('/') && !value.contains('\\') && value != "." && value != ".." {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn steam_root_candidates() -> Vec<PathBuf> {
    let mut roots = BTreeSet::new();
    for key in ["PROGRAMFILES(X86)", "PROGRAMFILES"] {
        if let Some(root) = std::env::var_os(key) {
            roots.insert(PathBuf::from(root).join("Steam"));
        }
    }
    if let Some(home) = std::env::var_os("USERPROFILE") {
        roots.insert(PathBuf::from(home).join("Steam"));
    }
    roots.into_iter().collect()
}

fn canonical_child(root: &Path, child: &Path) -> Option<PathBuf> {
    let root = root.canonicalize().ok()?;
    let child = child.canonicalize().ok()?;
    child.starts_with(&root).then_some(child)
}

pub(crate) fn validate_game_root(path: &Path) -> Result<PathBuf, String> {
    let root = path
        .canonicalize()
        .map_err(|_| "Không thể truy cập thư mục The Isle.".to_string())?;
    let exe = root
        .join("TheIsle")
        .join("Binaries")
        .join("Win64")
        .join("TheIsleClient-Win64-Shipping.exe");
    let localization = root.join("TheIsle").join("Content").join("Localization");
    if !exe.is_file() || !localization.is_dir() {
        return Err("Thư mục được tìm thấy không phải bản cài The Isle EVRIMA hợp lệ.".to_string());
    }
    Ok(root)
}

fn libraries_from_root(steam_root: &Path) -> Vec<PathBuf> {
    if !steam_root.is_dir() {
        return Vec::new();
    }
    let mut libraries = BTreeSet::new();
    if let Ok(canonical) = steam_root.canonicalize() {
        libraries.insert(canonical);
    }
    let vdf = steam_root.join("steamapps").join("libraryfolders.vdf");
    if let Ok(text) = std::fs::read_to_string(vdf) {
        for path in parse_steam_library_paths(&text) {
            if let Ok(canonical) = path.canonicalize() {
                libraries.insert(canonical);
            }
        }
    }
    libraries.into_iter().collect()
}

fn game_from_library(library: &Path) -> Option<PathBuf> {
    let steamapps = library.join("steamapps");
    let manifest_path = steamapps.join(format!("appmanifest_{THE_ISLE_APP_ID}.acf"));
    let manifest = std::fs::read_to_string(manifest_path).ok()?;
    let install_dir = parse_install_dir(&manifest)?;
    let common = steamapps.join("common");
    let candidate = common.join(install_dir);
    let candidate = canonical_child(&common, &candidate)?;
    validate_game_root(&candidate).ok()
}

pub(crate) fn detect_game_root() -> Option<PathBuf> {
    let mut seen = BTreeSet::new();
    for steam_root in steam_root_candidates() {
        for library in libraries_from_root(&steam_root) {
            if !seen.insert(library.clone()) {
                continue;
            }
            if let Some(game) = game_from_library(&library) {
                return Some(game);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("hubisle-{label}-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn parses_libraryfolders_and_manifest() {
        let vdf = r#"
        "libraryfolders"
        {
          "0" { "path" "C:\\Program Files (x86)\\Steam" }
          "1" { "path" "D:\\SteamLibrary" }
        }
        "#;
        let paths = parse_steam_library_paths(vdf);
        assert_eq!(paths.len(), 2);
        assert!(paths[1].to_string_lossy().contains("SteamLibrary"));

        let acf = r#""AppState" { "appid" "376210" "installdir" "The Isle" }"#;
        assert_eq!(parse_install_dir(acf), Some("The Isle".to_string()));
    }

    #[test]
    fn rejects_manifest_path_escape() {
        let acf = r#""AppState" { "installdir" "..\\Elsewhere" }"#;
        assert_eq!(parse_install_dir(acf), None);
    }

    #[test]
    fn validates_expected_evrima_markers() {
        let root = temp_root("valid-game");
        let exe_dir = root.join("TheIsle/Binaries/Win64");
        let loc_dir = root.join("TheIsle/Content/Localization/Game/en");
        fs::create_dir_all(&exe_dir).unwrap();
        fs::create_dir_all(&loc_dir).unwrap();
        fs::write(exe_dir.join("TheIsleClient-Win64-Shipping.exe"), b"").unwrap();
        fs::write(loc_dir.join("Game.locres"), b"fixture").unwrap();
        assert!(validate_game_root(&root).is_ok());
        let _ = fs::remove_dir_all(root);
    }
}
