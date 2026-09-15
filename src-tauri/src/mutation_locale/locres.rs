use std::collections::{BTreeSet, HashMap};
use std::io::{Cursor, Read};

const LOCRES_MAGIC: [u8; 16] = [
    0x0E, 0x14, 0x74, 0x75, 0x67, 0x4A, 0x03, 0xFC, 0x4A, 0x15, 0x90, 0x9D, 0xC3, 0x37,
    0x7F, 0x1B,
];

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

#[derive(Debug, Clone)]
pub struct PatchResult {
    pub bytes: Vec<u8>,
    pub replaced: usize,
    pub matched_sources: Vec<String>,
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
            .and_then(|n| i32::try_from(n).ok())
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
        .and_then(|n| i32::try_from(n).ok())
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
                    let _ref_count = read_i32(&mut cursor)?;
                }
            }
            cursor.set_position(return_pos);
            Some(values)
        } else {
            None
        };

        if version >= 2 {
            let _entry_count = read_i32(&mut cursor)?;
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
            let mut namespace_indexes = Vec::with_capacity(namespace.entries.len());
            for entry in &namespace.entries {
                let index = if let Some(index) = table.iter().position(|value| value == &entry.value) {
                    refs[index] += 1;
                    index
                } else {
                    table.push(entry.value.clone());
                    refs.push(1);
                    table.len() - 1
                };
                namespace_indexes.push(index);
            }
            entry_indexes.push(namespace_indexes);
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

pub fn patch_locres(
    input: &[u8],
    replacements: &HashMap<&str, &str>,
) -> Result<PatchResult, String> {
    let mut file = LocresFile::parse(input)?;
    let mut matched = BTreeSet::new();
    let mut replaced = 0usize;
    for namespace in &mut file.namespaces {
        for entry in &mut namespace.entries {
            if let Some(next) = replacements.get(entry.value.as_str()) {
                matched.insert(entry.value.clone());
                entry.value = (*next).to_string();
                replaced += 1;
            }
        }
    }
    Ok(PatchResult {
        bytes: file.write()?,
        replaced,
        matched_sources: matched.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(version: u8) -> Vec<u8> {
        LocresFile {
            version,
            namespaces: vec![LocresNamespace {
                name_hash: (version >= 2).then_some(0x1234_5678),
                name: "Mutation".to_string(),
                entries: vec![
                    LocresEntry {
                        key_hash: (version >= 2).then_some(0x1111_2222),
                        key: "Name".to_string(),
                        source_hash: 1,
                        value: "Cellular Regeneration".to_string(),
                    },
                    LocresEntry {
                        key_hash: (version >= 2).then_some(0x3333_4444),
                        key: "Description".to_string(),
                        source_hash: 2,
                        value: "Recovers health slightly faster".to_string(),
                    },
                ],
            }],
        }
        .write()
        .unwrap()
    }

    #[test]
    fn patches_description_but_keeps_english_mutation_name() {
        let input = fixture(3);
        let replacements = HashMap::from([(
            "Recovers health slightly faster",
            "Hồi máu nhanh hơn một chút.",
        )]);
        let result = patch_locres(&input, &replacements).unwrap();
        let roundtrip = LocresFile::parse(&result.bytes).unwrap();
        assert_eq!(roundtrip.values()[0], "Cellular Regeneration");
        assert_eq!(roundtrip.values()[1], "Hồi máu nhanh hơn một chút.");
        assert_eq!(result.replaced, 1);
        assert_eq!(
            result.matched_sources,
            vec!["Recovers health slightly faster".to_string()]
        );
    }

    #[test]
    fn supports_compact_and_optimized_locres() {
        for version in [1, 2, 3] {
            let input = fixture(version);
            let parsed = LocresFile::parse(&input).unwrap();
            assert_eq!(parsed.values().len(), 2);
            assert_eq!(parsed.write().unwrap(), input);
        }
    }

    #[test]
    fn rejects_unknown_or_truncated_locres() {
        let mut bad = LOCRES_MAGIC.to_vec();
        bad.push(9);
        assert!(LocresFile::parse(&bad).is_err());
        let truncated = &fixture(2)[..20];
        assert!(LocresFile::parse(truncated).is_err());
    }
}
