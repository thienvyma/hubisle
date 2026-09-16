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

fn value_after_key<'a>(tokens: &'a [String], key: &str) -> Option<&'a str> {
    tokens
        .windows(2)
        .find(|pair| pair[0].eq_ignore_ascii_case(key))
        .map(|pair| pair[1].as_str())
}

pub(crate) fn parse_steam_library_paths(input: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for line in input.lines() {
        let tokens = quoted_tokens(line);
        if let Some(value) = value_after_key(&tokens, "path") {
            paths.push(PathBuf::from(value));
        }
    }
    paths
}

pub(crate) fn parse_install_dir(input: &str) -> Option<String> {
    for line in input.lines() {
        let tokens = quoted_tokens(line);
        if let Some(raw) = value_after_key(&tokens, "installdir") {
            let value = raw.trim();
            if !value.is_empty()
                && !value.contains('/')
                && !value.contains('\\')
                && value != "."
                && value != ".."
            {
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

    // Steam itself is often installed outside Program Files. Probe only the
    // conventional client roots; this is a bounded set of cheap existence
    // checks and does not recursively scan drives.
    for letter in b'C'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        let drive = PathBuf::from(drive);
        roots.insert(drive.join("Steam"));
        roots.insert(drive.join("Program Files").join("Steam"));
        roots.insert(drive.join("Program Files (x86)").join("Steam"));
    }
    roots.into_iter().collect()
}

fn direct_library_candidates() -> Vec<PathBuf> {
    let mut libraries = BTreeSet::new();
    for letter in b'C'..=b'Z' {
        let drive = PathBuf::from(format!("{}:\\", letter as char));
        libraries.insert(drive.join("SteamLibrary"));
        libraries.insert(drive.join("Steam"));
    }
    libraries.into_iter().collect()
}

fn canonical_child(root: &Path, child: &Path) -> Option<PathBuf> {
    let root = root.canonicalize().ok()?;
    let child = child.canonicalize().ok()?;
    child.starts_with(&root).then_some(child)
}

fn has_packaged_content(paks: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(paks) else {
        return false;
    };
    entries.filter_map(Result::ok).any(|entry| {
        if !entry.file_type().is_ok_and(|kind| kind.is_file()) {
            return false;
        }
        entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|ext| {
                ext.eq_ignore_ascii_case("pak") || ext.eq_ignore_ascii_case("utoc")
            })
    })
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
    let paks = root.join("TheIsle").join("Content").join("Paks");
    if !exe.is_file() || !paks.is_dir() || !has_packaged_content(&paks) {
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
    let mut libraries = BTreeSet::new();
    for steam_root in steam_root_candidates() {
        libraries.extend(libraries_from_root(&steam_root));
    }
    for direct in direct_library_candidates() {
        if let Ok(canonical) = direct.canonicalize() {
            libraries.insert(canonical);
        }
    }

    libraries
        .into_iter()
        .find_map(|library| game_from_library(&library))
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
    fn validates_packaged_evrima_markers_without_loose_localization() {
        let root = temp_root("valid-packaged-game");
        let exe_dir = root.join("TheIsle/Binaries/Win64");
        let paks_dir = root.join("TheIsle/Content/Paks");
        fs::create_dir_all(&exe_dir).unwrap();
        fs::create_dir_all(&paks_dir).unwrap();
        fs::write(exe_dir.join("TheIsleClient-Win64-Shipping.exe"), b"").unwrap();
        fs::write(paks_dir.join("TheIsle-WindowsClient.pak"), b"fixture").unwrap();
        assert!(validate_game_root(&root).is_ok());
        let _ = fs::remove_dir_all(root);
    }
}
