use std::sync::LazyLock;

use regex::Regex;

#[derive(Clone, Debug, PartialEq)]
pub struct CatalogEntry {
    pub name_en: String,
    pub description_en: String,
    pub description_vi: String,
    pub match_texts: Vec<String>,
}

const MUTATIONS_TS: &str = include_str!("../../../src/lib/mutations.ts");

fn decode_js_string(literal: &str) -> String {
    serde_json::from_str::<String>(literal).expect("mutation catalog string must be valid JSON syntax")
}

fn observed_game_aliases(name: &str) -> &'static [&'static str] {
    match name {
        "Accelerated Prey Drive" => &["Deal more damage to animals with low health"],
        "Advanced Gestation" => &["Faster Egg Gestation/Incubation/Cooldown Rate"],
        "Cellular Regeneration" => &["Recovers health slightly faster"],
        "Congenital Hypoalgesia" => &["Reduce incoming damage when fighting larger species"],
        "Efficient Digestion" => &["Your food drains more slowly"],
        "Enlarged Meniscus" => &["Fall damage hits stamina before draining health, no value"],
        "Epidermal Fibrosis" => &["Increase bleed resistance"],
        "Featherweight" => &["Your footprints fade much faster"],
        "Hematophagy" => &["Restore some thirst when eating"],
        "Hemomania" => &["Do extra damage on a bleeding target"],
        "Hydrodynamic" => &["Increased swimming speed"],
        "Hydro-regenerative" => &["Recover health faster during rain"],
        "Hypervigilance" => &[
            "Increases camera angles when eating and drinking. Increases footsteps audio from others",
        ],
        "Increased Inspiratory Capacity" => &["Increased O2 capacity"],
        "Infrasound Communication" => &["Make significantly less noise when talking in chat"],
        "Nocturnal" => &["Faster health / locked health recovery during the night"],
        "Osteophagic" => &["Able to consume bones to regenerate fractures faster"],
        "Osteosclerosis" => &["Resist or Reduce Fracture damage"],
        "Photosynthetic Regeneration" => &["Regenerates stamina faster during the day"],
        "Photosynthetic Tissue" => &["Faster health / locked health recovery during the day"],
        "Reabsorption" => &[
            "Recover a small amount of water during the rainy weather or while swimming in drinkable water",
        ],
        "Reinforced Tendons" => &["Jumping costs less stamina"],
        "Reniculate Kidneys" => &["Can drink saltwater, no value"],
        "Multichambered Lungs" => &["Reduce stamina regeneration threshold"],
        "Submerged Optical Retention" => &["Increased underwater vision range"],
        "Sustained Hydration" => &["Your water drains more slowly"],
        "Truculency" => &["Bucking has a higher chance to dismount latched animals"],
        "Wader" => &["Less hindered when wading through shallow water"],
        "Xerocole Adaptation" => &["Gain some water when eating plants"],
        "Enhanced Digestion" => &["Decrease nutrition decay rate"],
        _ => &[],
    }
}

fn parse_catalog(source: &str) -> Vec<CatalogEntry> {
    let quoted = r#"("(?:\\.|[^"\\])*")"#;
    let pattern = format!(
        r#"(?s)\{{\s*nameEn:\s*{quoted},\s*descriptionEn:\s*{quoted},\s*descriptionVi:\s*{quoted},\s*\}}"#
    );
    let regex = Regex::new(&pattern).expect("mutation TypeScript parser regex must compile");

    let mut entries = Vec::new();
    for captures in regex.captures_iter(source) {
        let name_en = decode_js_string(captures.get(1).unwrap().as_str());
        let description_en = decode_js_string(captures.get(2).unwrap().as_str());
        let description_vi = decode_js_string(captures.get(3).unwrap().as_str());
        let mut match_texts = vec![description_en.clone()];
        for alias in observed_game_aliases(&name_en) {
            if !match_texts.iter().any(|item| item == alias) {
                match_texts.push((*alias).to_string());
            }
        }
        entries.push(CatalogEntry {
            name_en,
            description_en,
            description_vi,
            match_texts,
        });
    }

    entries.sort_by(|a, b| a.name_en.cmp(&b.name_en));
    entries
}

static CATALOG: LazyLock<Vec<CatalogEntry>> = LazyLock::new(|| {
    let entries = parse_catalog(MUTATIONS_TS);
    assert_eq!(
        entries.len(),
        42,
        "src/lib/mutations.ts must contain exactly 42 Mutation entries"
    );
    entries
});

pub fn catalog() -> &'static [CatalogEntry] {
    &CATALOG
}

pub fn find_by_name(name_en: &str) -> Option<&'static CatalogEntry> {
    CATALOG
        .iter()
        .find(|entry| entry.name_en.eq_ignore_ascii_case(name_en.trim()))
}

#[cfg(test)]
mod tests {
    use super::{catalog, find_by_name, parse_catalog, MUTATIONS_TS};

    #[test]
    fn typescript_catalog_is_the_single_translation_source() {
        let entries = parse_catalog(MUTATIONS_TS);
        assert_eq!(entries.len(), 42);
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.name_en == "Cellular Regeneration")
                .count(),
            1
        );
        assert_eq!(
            find_by_name("Cellular Regeneration")
                .expect("known mutation")
                .description_vi,
            "Hồi phục máu nhanh hơn 15%."
        );
        assert_eq!(catalog().len(), 42);
    }

    #[test]
    fn current_game_aliases_are_detection_only() {
        let cellular = find_by_name("Cellular Regeneration").expect("known mutation");
        assert!(cellular
            .match_texts
            .iter()
            .any(|text| text == "Recovers health slightly faster"));
        assert_eq!(cellular.description_en, "Recover health 15% faster.");
    }
}
