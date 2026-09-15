mod catalog;
mod locres;
mod steam;

use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{settings, win};

const PACK_VERSION: &str = "1.0.0";
const OWNERSHIP_SCHEMA: u32 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MutationLocaleState {
    GameNotFound,
    NotInstalled,
    Installed,
    UpdateAvailable,
    Incompatible,
    Corrupt,
    GameRunning,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationLocaleStatus {
    pub state: MutationLocaleState,
    pub game_path: Option<String>,
    pub pack_version: String,
    pub matched: usize,
    pub total: usize,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OwnedFile {
    relative_path: String,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceFile {
    relative_path: String,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OwnershipManifest {
    schema: u32,
    pack_version: String,
    game_path: String,
    source_fingerprint: String,
    previous_culture: Option<String>,
    files: Vec<OwnedFile>,
    sources: Vec<SourceFile>,
    matched_sources: Vec<String>,
}

fn mutation_locale_dir() -> PathBuf {
    settings::local_dir().join("mutation-locale")
}

fn ownership_path() -> PathBuf {
    mutation_locale_dir().join("ownership.json")
}

fn total() -> usize {
    catalog::total_descriptions()
}

fn status(
    state: MutationLocaleState,
    game_path: Option<&Path>,
    matched: usize,
    message: Option<String>,
) -> MutationLocaleStatus {
    MutationLocaleStatus {
        state,
        game_path: game_path.map(|path| path.to_string_lossy().into_owned()),
        pack_version: PACK_VERSION.to_string(),
        matched,
        total: total(),
        message,
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("Không thể đọc {}: {error}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

fn read_ownership() -> Option<OwnershipManifest> {
    let bytes = fs::read(ownership_path()).ok()?;
    let manifest: OwnershipManifest = serde_json::from_slice(&bytes).ok()?;
    (manifest.schema == OWNERSHIP_SCHEMA).then_some(manifest)
}

fn write_ownership(manifest: &OwnershipManifest) -> Result<(), String> {
    fs::create_dir_all(mutation_locale_dir())
        .map_err(|error| format!("Không thể tạo thư mục trạng thái Việt hoá: {error}"))?;
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("Không thể ghi manifest Việt hoá: {error}"))?;
    write_atomic(&ownership_path(), &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Đường dẫn đích không hợp lệ.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Không thể tạo {}: {error}", parent.display()))?;
    let temp = path.with_extension(format!(
        "{}.hubtmp",
        path.extension().and_then(|value| value.to_str()).unwrap_or("tmp")
    ));
    fs::write(&temp, bytes)
        .map_err(|error| format!("Không thể ghi file tạm {}: {error}", temp.display()))?;
    replace_from_temp(&temp, path)
}

fn replace_from_temp(temp: &Path, target: &Path) -> Result<(), String> {
    if !target.exists() {
        return fs::rename(temp, target)
            .map_err(|error| format!("Không thể cài {}: {error}", target.display()));
    }

    let backup = target.with_extension(format!(
        "{}.hubold",
        target.extension().and_then(|value| value.to_str()).unwrap_or("bak")
    ));
    let _ = fs::remove_file(&backup);
    fs::rename(target, &backup)
        .map_err(|error| format!("Không thể chuẩn bị cập nhật {}: {error}", target.display()))?;
    match fs::rename(temp, target) {
        Ok(()) => {
            let _ = fs::remove_file(backup);
            Ok(())
        }
        Err(error) => {
            let _ = fs::rename(&backup, target);
            Err(format!("Không thể cập nhật {}: {error}", target.display()))
        }
    }
}

fn localization_root(game_root: &Path) -> PathBuf {
    game_root.join("TheIsle").join("Content").join("Localization")
}

fn collect_english_locres(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir)
        .map_err(|error| format!("Không thể đọc {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("Không thể đọc localization: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("Không thể đọc loại file: {error}"))?;
        let path = entry.path();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_english_locres(&path, out)?;
        } else if file_type.is_file()
            && path.extension().and_then(|value| value.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("locres"))
            && path.components().any(|component| {
                let value = component.as_os_str().to_string_lossy();
                value.eq_ignore_ascii_case("en") || value.eq_ignore_ascii_case("en-US")
            })
        {
            out.push(path);
        }
    }
    Ok(())
}

fn destination_for_source(root: &Path, source: &Path) -> Option<PathBuf> {
    let relative = source.strip_prefix(root).ok()?;
    let mut out = PathBuf::new();
    let mut replaced = false;
    for component in relative.components() {
        match component {
            Component::Normal(value) if !replaced => {
                let text = value.to_string_lossy();
                if text.eq_ignore_ascii_case("en") || text.eq_ignore_ascii_case("en-US") {
                    out.push("vi");
                    replaced = true;
                } else {
                    out.push(value);
                }
            }
            Component::Normal(value) => out.push(value),
            _ => return None,
        }
    }
    replaced.then(|| root.join(out))
}

fn config_text() -> String {
    fs::read_to_string(settings::game_config_path()).unwrap_or_default()
}

fn read_culture(text: &str) -> Option<String> {
    let mut in_section = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line.eq_ignore_ascii_case("[Internationalization]");
            continue;
        }
        if in_section {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim().eq_ignore_ascii_case("Culture") {
                    return Some(value.trim().to_string());
                }
            }
        }
    }
    None
}

fn set_culture_text(text: &str, culture: Option<&str>) -> String {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let had_trailing_newline = text.ends_with('\n');
    let mut lines = text.lines().map(str::to_string).collect::<Vec<_>>();
    let mut section_start = None;
    let mut section_end = lines.len();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if section_start.is_some() {
                section_end = index;
                break;
            }
            if trimmed.eq_ignore_ascii_case("[Internationalization]") {
                section_start = Some(index);
            }
        }
    }

    if let Some(start) = section_start {
        let culture_line = (start + 1..section_end).find(|&index| {
            lines[index]
                .split_once('=')
                .is_some_and(|(key, _)| key.trim().eq_ignore_ascii_case("Culture"))
        });
        match (culture_line, culture) {
            (Some(index), Some(value)) => lines[index] = format!("Culture={value}"),
            (Some(index), None) => {
                lines.remove(index);
            }
            (None, Some(value)) => lines.insert(start + 1, format!("Culture={value}")),
            (None, None) => {}
        }
    } else if let Some(value) = culture {
        if !lines.is_empty() && !lines.last().is_some_and(|line| line.is_empty()) {
            lines.push(String::new());
        }
        lines.push("[Internationalization]".to_string());
        lines.push(format!("Culture={value}"));
    }

    let mut output = lines.join(newline);
    if had_trailing_newline || (!output.is_empty() && culture.is_some()) {
        output.push_str(newline);
    }
    output
}

fn write_culture(culture: Option<&str>) -> Result<(), String> {
    let path = settings::game_config_path();
    let current = config_text();
    let next = set_culture_text(&current, culture);
    write_atomic(&path, next.as_bytes())
}

fn manifest_game_root(manifest: &OwnershipManifest) -> Option<PathBuf> {
    steam::validate_game_root(Path::new(&manifest.game_path)).ok()
}

fn relative_owned_path(game_root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(game_root)
        .map_err(|_| "File Việt hoá nằm ngoài thư mục The Isle.".to_string())?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn resolve_owned_path(game_root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("Manifest Việt hoá chứa đường dẫn không an toàn.".to_string());
    }
    Ok(game_root.join(relative_path))
}

fn fingerprint_sources(game_root: &Path, sources: &[SourceFile]) -> Result<String, String> {
    let mut hasher = Sha256::new();
    let mut sorted = sources.to_vec();
    sorted.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    for source in sorted {
        let path = resolve_owned_path(game_root, &source.relative_path)?;
        let bytes = fs::read(&path)
            .map_err(|error| format!("Không thể kiểm tra {}: {error}", path.display()))?;
        hasher.update(source.relative_path.as_bytes());
        hasher.update([0]);
        hasher.update(&bytes);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn installed_status(manifest: &OwnershipManifest, game_root: &Path) -> MutationLocaleStatus {
    let matched = manifest.matched_sources.len();
    if manifest.pack_version != PACK_VERSION {
        return status(
            MutationLocaleState::UpdateAvailable,
            Some(game_root),
            matched,
            Some("Có phiên bản gói Việt hoá mới.".to_string()),
        );
    }

    let fingerprint = match fingerprint_sources(game_root, &manifest.sources) {
        Ok(value) => value,
        Err(message) => {
            return status(
                MutationLocaleState::Incompatible,
                Some(game_root),
                matched,
                Some(message),
            )
        }
    };
    if fingerprint != manifest.source_fingerprint {
        return status(
            MutationLocaleState::Incompatible,
            Some(game_root),
            matched,
            Some("The Isle đã thay đổi. Cần cập nhật lại gói Việt hoá trước khi cài tiếp.".to_string()),
        );
    }

    for owned in &manifest.files {
        let Ok(path) = resolve_owned_path(game_root, &owned.relative_path) else {
            return status(
                MutationLocaleState::Corrupt,
                Some(game_root),
                matched,
                Some("Manifest Việt hoá không hợp lệ.".to_string()),
            );
        };
        if !path.is_file() || sha256_file(&path).ok().as_deref() != Some(owned.sha256.as_str()) {
            return status(
                MutationLocaleState::Corrupt,
                Some(game_root),
                matched,
                Some("Một file Việt hoá đã bị thay đổi hoặc bị thiếu.".to_string()),
            );
        }
    }

    status(
        MutationLocaleState::Installed,
        Some(game_root),
        matched,
        None,
    )
}

#[tauri::command]
pub fn mutation_locale_status() -> Result<MutationLocaleStatus, String> {
    if let Some(manifest) = read_ownership() {
        if let Some(game_root) = manifest_game_root(&manifest) {
            return Ok(installed_status(&manifest, &game_root));
        }
    }
    if let Some(game_root) = steam::detect_game_root() {
        return Ok(status(
            MutationLocaleState::NotInstalled,
            Some(&game_root),
            0,
            None,
        ));
    }
    Ok(status(
        MutationLocaleState::GameNotFound,
        None,
        0,
        Some("Không tìm thấy The Isle EVRIMA trong thư viện Steam.".to_string()),
    ))
}

#[tauri::command]
pub fn mutation_locale_install() -> Result<MutationLocaleStatus, String> {
    if win::game_window::find_game_window(settings::GAME_PROCESS_NAME).is_some() {
        return Ok(status(
            MutationLocaleState::GameRunning,
            steam::detect_game_root().as_deref(),
            0,
            Some("Hãy đóng The Isle trước khi cài Việt hoá.".to_string()),
        ));
    }

    let game_root = steam::detect_game_root().ok_or_else(|| {
        "Không tìm thấy The Isle EVRIMA. Hãy kiểm tra lại thư viện Steam.".to_string()
    })?;
    let root = localization_root(&game_root);
    let mut sources = Vec::new();
    collect_english_locres(&root, &mut sources)?;
    sources.sort();
    if sources.is_empty() {
        return Ok(status(
            MutationLocaleState::Incompatible,
            Some(&game_root),
            0,
            Some("Không tìm thấy localization tiếng Anh của The Isle hiện tại.".to_string()),
        ));
    }

    let existing = read_ownership().filter(|manifest| Path::new(&manifest.game_path) == game_root);
    let existing_owned: HashSet<String> = existing
        .as_ref()
        .map(|manifest| manifest.files.iter().map(|file| file.relative_path.clone()).collect())
        .unwrap_or_default();
    let previous_culture = existing
        .as_ref()
        .and_then(|manifest| manifest.previous_culture.clone())
        .or_else(|| read_culture(&config_text()));

    let replacements = catalog::description_replacements();
    let mut matched = BTreeSet::new();
    let mut staged = Vec::<(PathBuf, PathBuf, Vec<u8>)>::new();
    let mut source_records = Vec::<SourceFile>::new();

    for source in sources {
        let bytes = fs::read(&source)
            .map_err(|error| format!("Không thể đọc {}: {error}", source.display()))?;
        let patched = match locres::patch_locres(&bytes, &replacements) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if patched.replaced == 0 {
            continue;
        }
        let target = destination_for_source(&root, &source)
            .ok_or_else(|| format!("Không xác định được đích Việt hoá cho {}", source.display()))?;
        let target_relative = relative_owned_path(&game_root, &target)?;
        if target.exists() && !existing_owned.contains(&target_relative) {
            return Err(format!(
                "{} đã tồn tại và không thuộc gói Việt hoá của hub; không ghi đè file này.",
                target.display()
            ));
        }
        // Parse the generated bytes once more before any game file is touched.
        locres::patch_locres(&patched.bytes, &HashMap::new())
            .map_err(|error| format!("Gói Việt hoá tạo ra không hợp lệ: {error}"))?;
        matched.extend(patched.matched_sources.iter().cloned());
        let source_relative = relative_owned_path(&game_root, &source)?;
        source_records.push(SourceFile {
            relative_path: source_relative,
            sha256: sha256_bytes(&bytes),
        });
        let temp = target.with_extension(format!(
            "{}.hubtmp",
            target.extension().and_then(|value| value.to_str()).unwrap_or("locres")
        ));
        staged.push((temp, target, patched.bytes));
    }

    if matched.is_empty() {
        return Ok(status(
            MutationLocaleState::Incompatible,
            Some(&game_root),
            0,
            Some("Bản The Isle hiện tại không còn khớp các chuỗi Mutation đã kiểm chứng; chưa cài gì vào game.".to_string()),
        ));
    }

    for (temp, target, bytes) in &staged {
        let parent = target
            .parent()
            .ok_or_else(|| "Đường dẫn localization không hợp lệ.".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Không thể tạo {}: {error}", parent.display()))?;
        fs::write(temp, bytes)
            .map_err(|error| format!("Không thể ghi {}: {error}", temp.display()))?;
    }

    let mut owned_files = Vec::new();
    for (temp, target, bytes) in &staged {
        replace_from_temp(temp, target)?;
        owned_files.push(OwnedFile {
            relative_path: relative_owned_path(&game_root, target)?,
            sha256: sha256_bytes(bytes),
        });
    }

    write_culture(Some("vi"))?;
    let source_fingerprint = fingerprint_sources(&game_root, &source_records)?;
    let manifest = OwnershipManifest {
        schema: OWNERSHIP_SCHEMA,
        pack_version: PACK_VERSION.to_string(),
        game_path: game_root.to_string_lossy().into_owned(),
        source_fingerprint,
        previous_culture,
        files: owned_files,
        sources: source_records,
        matched_sources: matched.into_iter().collect(),
    };
    write_ownership(&manifest)?;
    Ok(installed_status(&manifest, &game_root))
}

#[tauri::command]
pub fn mutation_locale_uninstall() -> Result<MutationLocaleStatus, String> {
    if win::game_window::find_game_window(settings::GAME_PROCESS_NAME).is_some() {
        return Ok(status(
            MutationLocaleState::GameRunning,
            read_ownership().and_then(|manifest| manifest_game_root(&manifest)).as_deref(),
            0,
            Some("Hãy đóng The Isle trước khi gỡ Việt hoá.".to_string()),
        ));
    }

    let Some(manifest) = read_ownership() else {
        return mutation_locale_status();
    };
    let game_root = manifest_game_root(&manifest)
        .ok_or_else(|| "Không còn tìm thấy bản cài The Isle đã được Việt hoá.".to_string())?;
    let mut modified = Vec::new();
    for owned in &manifest.files {
        let path = resolve_owned_path(&game_root, &owned.relative_path)?;
        if !path.exists() {
            continue;
        }
        if sha256_file(&path).ok().as_deref() != Some(owned.sha256.as_str()) {
            modified.push(path);
            continue;
        }
        fs::remove_file(&path)
            .map_err(|error| format!("Không thể xoá {}: {error}", path.display()))?;
        let mut parent = path.parent();
        let stop = localization_root(&game_root);
        while let Some(dir) = parent {
            if dir == stop || !dir.starts_with(&stop) {
                break;
            }
            if fs::remove_dir(dir).is_err() {
                break;
            }
            parent = dir.parent();
        }
    }

    if !modified.is_empty() {
        return Ok(status(
            MutationLocaleState::Corrupt,
            Some(&game_root),
            manifest.matched_sources.len(),
            Some("Có file Việt hoá đã được thay đổi sau khi cài nên hub giữ nguyên file đó để tránh mất dữ liệu.".to_string()),
        ));
    }

    if read_culture(&config_text()).as_deref() == Some("vi") {
        write_culture(manifest.previous_culture.as_deref())?;
    }
    let _ = fs::remove_file(ownership_path());
    let _ = fs::remove_dir(mutation_locale_dir());
    Ok(status(
        MutationLocaleState::NotInstalled,
        Some(&game_root),
        0,
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn culture_patch_adds_replaces_and_removes_only_culture_value() {
        let original = "[ScalabilityGroups]\r\nsg.ViewDistanceQuality=3\r\n";
        let installed = set_culture_text(original, Some("vi"));
        assert!(installed.contains("[Internationalization]\r\nCulture=vi"));
        assert_eq!(read_culture(&installed).as_deref(), Some("vi"));

        let switched = set_culture_text(&installed, Some("en"));
        assert_eq!(read_culture(&switched).as_deref(), Some("en"));
        assert_eq!(switched.matches("Culture=").count(), 1);

        let removed = set_culture_text(&installed, None);
        assert_eq!(read_culture(&removed), None);
        assert!(removed.contains("sg.ViewDistanceQuality=3"));
    }

    #[test]
    fn destination_changes_only_english_culture_component() {
        let root = Path::new(r"C:\Game\TheIsle\Content\Localization");
        let source = root.join("Game").join("en").join("Game.locres");
        assert_eq!(
            destination_for_source(root, &source),
            Some(root.join("Game").join("vi").join("Game.locres"))
        );
    }

    #[test]
    fn owned_paths_cannot_escape_game_root() {
        let root = Path::new(r"C:\Steam\The Isle");
        assert!(resolve_owned_path(root, "TheIsle/Content/Localization/Game/vi/Game.locres").is_ok());
        assert!(resolve_owned_path(root, "../other/file.locres").is_err());
        assert!(resolve_owned_path(root, r"C:\Windows\file.locres").is_err());
    }
}
