use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::super::catalog;

const PUBLIC_MANIFEST_URL: &str = "https://isle.klong.dev/v1/releases/translation/latest";
const PUBLIC_ALLOWED_HOST: &str = "isle.klong.dev";
const PUBLIC_ARTIFACT_PREFIX: &str = "/assets/translation/";
const MAX_ARCHIVE_BYTES: usize = 64 * 1024 * 1024;
const MAX_LOCRES_BYTES: u64 = 8 * 1024 * 1024;
const LOCRES_MAGIC: [u8; 16] = [
    0x0E, 0x14, 0x74, 0x75, 0x67, 0x4A, 0x03, 0xFC, 0x4A, 0x15, 0x90, 0x9D, 0xC3, 0x37,
    0x7F, 0x1B,
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicManifest {
    version: String,
    published: bool,
    url: String,
    sha256: String,
    #[serde(default)]
    files: Vec<PublicTranslationFile>,
}

#[derive(Debug, Clone, Deserialize)]
struct PublicTranslationFile {
    source: String,
    path: String,
    sha256: String,
    size: u64,
}

#[derive(Debug, Clone)]
struct LocresEntry {
    key_hash: Option<u32>,
    key: String,
    source_hash: u32,
    value: String,
}

#[derive(Debug, Clone)]
struct LocresNamespace {
    name_hash: Option<u32>,
    name: String,
    entries: Vec<LocresEntry>,
}

#[derive(Debug, Clone)]
struct LocresFile {
    version: u8,
    namespaces: Vec<LocresNamespace>,
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn escape_powershell_literal(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

fn normalize_resource_path(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(normalized)
}

fn validate_public_url(url: &str, artifact: bool) -> Result<reqwest::Url, String> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|error| format!("Địa chỉ gói Việt hoá không hợp lệ: {error}"))?;
    if parsed.scheme() != "https"
        || parsed.host_str() != Some(PUBLIC_ALLOWED_HOST)
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || (artifact && !parsed.path().starts_with(PUBLIC_ARTIFACT_PREFIX))
    {
        return Err("Địa chỉ gói Việt hoá nằm ngoài HTTPS allowlist.".to_string());
    }
    Ok(parsed)
}

fn choose_game_locres(manifest: &PublicManifest) -> Option<PublicTranslationFile> {
    let mut fallback = None;
    for file in &manifest.files {
        let Some(path) = normalize_resource_path(&file.path) else {
            continue;
        };
        let lower = path.to_ascii_lowercase();
        if lower == "theisle/content/localization/game/vi/game.locres" {
            return Some(file.clone());
        }
        if lower == "theisle/content/localization/game/vi-vn/game.locres" {
            fallback = Some(file.clone());
        }
    }
    fallback
}

fn download_public_game_locres(cache_root: &Path) -> Result<(Vec<u8>, String), String> {
    fs::create_dir_all(cache_root)
        .map_err(|error| format!("Không thể tạo cache gói Việt hoá: {error}"))?;

    let manifest_url = validate_public_url(PUBLIC_MANIFEST_URL, false)?;
    let response = reqwest::blocking::get(manifest_url)
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Không thể tải manifest Việt hoá công khai: {error}"))?;
    let manifest: PublicManifest = response
        .json()
        .map_err(|error| format!("Manifest Việt hoá công khai không hợp lệ: {error}"))?;
    if !manifest.published {
        return Err("Gói Việt hoá công khai hiện chưa được phát hành.".to_string());
    }
    let game_file = choose_game_locres(&manifest)
        .ok_or_else(|| "Gói Việt hoá công khai không chứa Game.locres tiếng Việt.".to_string())?;
    if game_file.size == 0 || game_file.size > MAX_LOCRES_BYTES {
        return Err("Game.locres trong gói công khai có kích thước không hợp lệ.".to_string());
    }

    let artifact_url = validate_public_url(&manifest.url, true)?;
    let archive = reqwest::blocking::get(artifact_url)
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Không thể tải gói localization công khai: {error}"))?
        .bytes()
        .map_err(|error| format!("Không thể đọc gói localization công khai: {error}"))?;
    if archive.len() > MAX_ARCHIVE_BYTES {
        return Err("Gói localization công khai vượt giới hạn kích thước.".to_string());
    }
    let archive_hash = sha256_bytes(&archive);
    if !archive_hash.eq_ignore_ascii_case(&manifest.sha256) {
        return Err("SHA-256 của gói localization công khai không khớp manifest.".to_string());
    }

    let zip_path = cache_root.join("public-translation.zip");
    fs::write(&zip_path, &archive)
        .map_err(|error| format!("Không thể lưu gói localization công khai: {error}"))?;
    let extract_root = cache_root.join("public-translation");
    let _ = fs::remove_dir_all(&extract_root);
    fs::create_dir_all(&extract_root)
        .map_err(|error| format!("Không thể tạo thư mục giải nén localization: {error}"))?;
    let command = format!(
        "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
        escape_powershell_literal(&zip_path),
        escape_powershell_literal(&extract_root)
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
        .map_err(|error| format!("Không thể chạy PowerShell để giải nén localization: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Không thể giải nén gói localization công khai: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let source = normalize_resource_path(&game_file.source)
        .ok_or_else(|| "Đường dẫn Game.locres trong manifest không an toàn.".to_string())?;
    let extracted = source
        .split('/')
        .fold(extract_root.clone(), |path, part| path.join(part));
    let bytes = fs::read(&extracted)
        .map_err(|error| format!("Không thể đọc {}: {error}", extracted.display()))?;
    if bytes.len() as u64 != game_file.size
        || !sha256_bytes(&bytes).eq_ignore_ascii_case(&game_file.sha256)
    {
        return Err("Game.locres công khai không vượt qua kiểm tra SHA-256.".to_string());
    }

    Ok((bytes, manifest.version))
}

fn read_exact<const N: usize>(cursor: &mut Cursor<&[u8]>) -> Result<[u8; N], String> {
    let mut buf = [0u8; N];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| "locres truncated".to_string())?;
    Ok(buf)
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, String> {
    Ok(read_exact::<1>(cursor)?[0])
}

fn read_i32(cursor: &mut Cursor<&[u8]>) -> Result<i32, String> {
    Ok(i32::from_le_bytes(read_exact::<4>(cursor)?))
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, String> {
    Ok(u32::from_le_bytes(read_exact::<4>(cursor)?))
}

fn read_i64(cursor: &mut Cursor<&[u8]>) -> Result<i64, String> {
    Ok(i64::from_le_bytes(read_exact::<8>(cursor)?))
}

fn checked_count(value: i32, what: &str) -> Result<usize, String> {
    if value < 0 || value > 1_000_000 {
        return Err(format!("invalid {what} count"));
    }
    Ok(value as usize)
}

fn read_unreal_string(cursor: &mut Cursor<&[u8]>) -> Result<String, String> {
    let len = read_i32(cursor)?;
    if len == 0 {
        return Ok(String::new());
    }
    if len > 0 {
        let count = checked_count(len, "ansi string")?;
        let mut bytes = vec![0u8; count];
        cursor
            .read_exact(&mut bytes)
            .map_err(|_| "locres truncated string".to_string())?;
        if bytes.last() == Some(&0) {
            bytes.pop();
        }
        return Ok(String::from_utf8_lossy(&bytes).into_owned());
    }

    let units = len
        .checked_neg()
        .ok_or_else(|| "invalid utf16 string length".to_string())?;
    let count = checked_count(units, "utf16 string")?;
    let mut raw = vec![0u8; count.saturating_mul(2)];
    cursor
        .read_exact(&mut raw)
        .map_err(|_| "locres truncated utf16 string".to_string())?;
    let mut utf16 = raw
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    if utf16.last() == Some(&0) {
        utf16.pop();
    }
    String::from_utf16(&utf16).map_err(|_| "locres invalid utf16".to_string())
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_unreal_string(out: &mut Vec<u8>, value: &str) -> Result<(), String> {
    if value.is_ascii() {
        let len = value
            .len()
            .checked_add(1)
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| "locres string too large".to_string())?;
        put_i32(out, len);
        out.extend_from_slice(value.as_bytes());
        out.push(0);
        return Ok(());
    }

    let utf16 = value.encode_utf16().collect::<Vec<_>>();
    let units = utf16
        .len()
        .checked_add(1)
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| "locres string too large".to_string())?;
    put_i32(out, -units);
    for unit in utf16 {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out.extend_from_slice(&0u16.to_le_bytes());
    Ok(())
}

impl LocresFile {
    fn parse(input: &[u8]) -> Result<Self, String> {
        let mut cursor = Cursor::new(input);
        let version = if input.starts_with(&LOCRES_MAGIC) {
            cursor.set_position(LOCRES_MAGIC.len() as u64);
            let version = read_u8(&mut cursor)?;
            if version > 3 {
                return Err(format!("unsupported locres version {version}"));
            }
            version
        } else {
            0
        };

        let string_table = if version >= 1 {
            let offset = read_i64(&mut cursor)?;
            if offset < 0 || offset as usize >= input.len() {
                return Err("invalid locres string-table offset".to_string());
            }
            let return_pos = cursor.position();
            cursor.set_position(offset as u64);
            let count = checked_count(read_i32(&mut cursor)?, "string table")?;
            let mut values = Vec::with_capacity(count);
            for _ in 0..count {
                values.push(read_unreal_string(&mut cursor)?);
                if version >= 2 {
                    let _ = read_i32(&mut cursor)?;
                }
            }
            cursor.set_position(return_pos);
            Some(values)
        } else {
            None
        };

        if version >= 2 {
            let _ = read_i32(&mut cursor)?;
        }
        let namespace_count = checked_count(read_i32(&mut cursor)?, "namespace")?;
        let mut namespaces = Vec::with_capacity(namespace_count);
        for _ in 0..namespace_count {
            let name_hash = if version >= 2 {
                Some(read_u32(&mut cursor)?)
            } else {
                None
            };
            let name = read_unreal_string(&mut cursor)?;
            let entry_count = checked_count(read_i32(&mut cursor)?, "entry")?;
            let mut entries = Vec::with_capacity(entry_count);
            for _ in 0..entry_count {
                let key_hash = if version >= 2 {
                    Some(read_u32(&mut cursor)?)
                } else {
                    None
                };
                let key = read_unreal_string(&mut cursor)?;
                let source_hash = read_u32(&mut cursor)?;
                let value = if version >= 1 {
                    let index = read_i32(&mut cursor)?;
                    if index < 0 {
                        return Err("negative locres string-table index".to_string());
                    }
                    string_table
                        .as_ref()
                        .and_then(|table| table.get(index as usize))
                        .cloned()
                        .ok_or_else(|| "locres string-table index out of range".to_string())?
                } else {
                    read_unreal_string(&mut cursor)?
                };
                entries.push(LocresEntry {
                    key_hash,
                    key,
                    source_hash,
                    value,
                });
            }
            namespaces.push(LocresNamespace {
                name_hash,
                name,
                entries,
            });
        }
        Ok(Self {
            version,
            namespaces,
        })
    }

    fn write(&self) -> Result<Vec<u8>, String> {
        if self.version == 0 {
            let mut out = Vec::new();
            put_i32(&mut out, self.namespaces.len() as i32);
            for namespace in &self.namespaces {
                write_unreal_string(&mut out, &namespace.name)?;
                put_i32(&mut out, namespace.entries.len() as i32);
                for entry in &namespace.entries {
                    write_unreal_string(&mut out, &entry.key)?;
                    put_u32(&mut out, entry.source_hash);
                    write_unreal_string(&mut out, &entry.value)?;
                }
            }
            return Ok(out);
        }

        let mut table = Vec::<String>::new();
        let mut refs = Vec::<i32>::new();
        let mut entry_indexes = Vec::<Vec<usize>>::new();
        for namespace in &self.namespaces {
            let mut indexes = Vec::with_capacity(namespace.entries.len());
            for entry in &namespace.entries {
                let index = if let Some(index) = table.iter().position(|value| value == &entry.value) {
                    refs[index] += 1;
                    index
                } else {
                    table.push(entry.value.clone());
                    refs.push(1);
                    table.len() - 1
                };
                indexes.push(index);
            }
            entry_indexes.push(indexes);
        }

        let mut out = Vec::new();
        out.extend_from_slice(&LOCRES_MAGIC);
        out.push(self.version);
        let offset_position = out.len();
        put_i64(&mut out, 0);
        if self.version >= 2 {
            put_i32(
                &mut out,
                self.namespaces
                    .iter()
                    .map(|namespace| namespace.entries.len() as i32)
                    .sum(),
            );
        }
        put_i32(&mut out, self.namespaces.len() as i32);
        for (namespace_index, namespace) in self.namespaces.iter().enumerate() {
            if self.version >= 2 {
                put_u32(&mut out, namespace.name_hash.unwrap_or(0));
            }
            write_unreal_string(&mut out, &namespace.name)?;
            put_i32(&mut out, namespace.entries.len() as i32);
            for (entry_index, entry) in namespace.entries.iter().enumerate() {
                if self.version >= 2 {
                    put_u32(&mut out, entry.key_hash.unwrap_or(0));
                }
                write_unreal_string(&mut out, &entry.key)?;
                put_u32(&mut out, entry.source_hash);
                put_i32(&mut out, entry_indexes[namespace_index][entry_index] as i32);
            }
        }

        let table_offset = i64::try_from(out.len()).map_err(|_| "locres too large".to_string())?;
        out[offset_position..offset_position + 8].copy_from_slice(&table_offset.to_le_bytes());
        put_i32(&mut out, table.len() as i32);
        for (index, value) in table.iter().enumerate() {
            write_unreal_string(&mut out, value)?;
            if self.version >= 2 {
                put_i32(&mut out, refs[index]);
            }
        }
        Ok(out)
    }

    #[cfg(test)]
    fn values(&self) -> Vec<&str> {
        self.namespaces
            .iter()
            .flat_map(|namespace| namespace.entries.iter().map(|entry| entry.value.as_str()))
            .collect()
    }
}

fn filter_vietnamese_locres(vietnamese: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let reverse = catalog::DESCRIPTIONS
        .iter()
        .map(|entry| (entry.vi, entry.source))
        .collect::<HashMap<_, _>>();
    let mut file = LocresFile::parse(vietnamese)?;
    let mut retained = 0usize;

    for namespace in &mut file.namespaces {
        namespace.entries.retain_mut(|entry| {
            let Some(source) = reverse.get(entry.value.as_str()) else {
                return false;
            };
            entry.value = (*source).to_string();
            retained += 1;
            true
        });
    }
    file.namespaces.retain(|namespace| !namespace.entries.is_empty());

    if retained == 0 {
        return Err("Game.locres công khai không chứa mô tả Mutation tương thích.".to_string());
    }
    Ok((file.write()?, retained))
}

pub(super) fn build_english_mutation_locres(cache_root: &Path) -> Result<(Vec<u8>, usize), String> {
    let (vietnamese, _version) = download_public_game_locres(cache_root)?;
    filter_vietnamese_locres(&vietnamese)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<u8> {
        LocresFile {
            version: 3,
            namespaces: vec![LocresNamespace {
                name_hash: Some(0x1111_2222),
                name: "Mutation".to_string(),
                entries: vec![
                    LocresEntry {
                        key_hash: Some(0xAAAA_BBBB),
                        key: "Name".to_string(),
                        source_hash: 1,
                        value: "Tái Tạo Tế Bào".to_string(),
                    },
                    LocresEntry {
                        key_hash: Some(0xCCCC_DDDD),
                        key: "Description".to_string(),
                        source_hash: 2,
                        value: "Hồi máu nhanh hơn một chút.".to_string(),
                    },
                    LocresEntry {
                        key_hash: Some(0xEEEE_FFFF),
                        key: "Other".to_string(),
                        source_hash: 3,
                        value: "Một chuỗi giao diện khác".to_string(),
                    },
                ],
            }],
        }
        .write()
        .unwrap()
    }

    #[test]
    fn public_resource_is_reduced_to_mutation_descriptions_only() {
        let (bytes, retained) = filter_vietnamese_locres(&fixture()).unwrap();
        let parsed = LocresFile::parse(&bytes).unwrap();
        assert_eq!(retained, 1);
        assert_eq!(parsed.values(), vec!["Recovers health slightly faster"]);
        assert!(!parsed.values().contains(&"Tái Tạo Tế Bào"));
        assert!(!parsed.values().contains(&"Một chuỗi giao diện khác"));
    }

    #[test]
    fn manifest_prefers_vi_game_locres() {
        let manifest = PublicManifest {
            version: "test".to_string(),
            published: true,
            url: "https://isle.klong.dev/assets/translation/test.zip".to_string(),
            sha256: "00".repeat(32),
            files: vec![PublicTranslationFile {
                source: "Game.locres".to_string(),
                path: "TheIsle/Content/Localization/Game/vi/Game.locres".to_string(),
                sha256: "11".repeat(32),
                size: 123,
            }],
        };
        assert_eq!(choose_game_locres(&manifest).unwrap().source, "Game.locres");
    }
}
