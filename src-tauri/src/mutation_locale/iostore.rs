mod public_pack;

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct IoStoreLocres {
    pub virtual_path: String,
    pub bytes: Vec<u8>,
    pub utoc_path: PathBuf,
}

fn game_fingerprint_file(game_root: &Path) -> Option<PathBuf> {
    let paks = game_root.join("TheIsle").join("Content").join("Paks");
    ["pakchunk0-WindowsClient.utoc", "global.utoc"]
        .into_iter()
        .map(|name| paks.join(name))
        .find(|path| path.is_file())
        .or_else(|| {
            fs::read_dir(&paks)
                .ok()?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|path| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("utoc"))
                })
        })
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
        if !replaced
            && (component.eq_ignore_ascii_case("en") || component.eq_ignore_ascii_case("en-US"))
        {
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
    // Current EVRIMA pakchunk IoStore containers are encrypted. Do not recover
    // or use the game's encryption key. Instead, use the public,
    // checksum-verified Vietnamese Game.locres published by IsleLiveMap as a
    // key/namespace reference, reduce it to Mutation descriptions only, and
    // convert those values back to English. The existing installer then applies
    // this hub's Vietnamese description strings and writes a loose vi locres.
    let (minimal_english, _retained) = public_pack::build_english_mutation_locres(cache_root)?;
    let fingerprint = game_fingerprint_file(game_root)
        .ok_or_else(|| "Không tìm thấy file IoStore để theo dõi phiên bản The Isle.".to_string())?;

    Ok(vec![IoStoreLocres {
        virtual_path: "TheIsle/Content/Localization/Game/en/Game.locres".to_string(),
        bytes: minimal_english,
        utoc_path: fingerprint,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_maps_only_english_culture_to_vi() {
        let root = Path::new(r"C:\Steam\steamapps\common\The Isle");
        assert_eq!(
            destination_for_virtual_source(
                root,
                "../../../TheIsle/Content/Localization/Game/en/Game.locres"
            ),
            Some(
                root.join("TheIsle")
                    .join("Content")
                    .join("Localization")
                    .join("Game")
                    .join("vi")
                    .join("Game.locres")
            )
        );
    }
}
