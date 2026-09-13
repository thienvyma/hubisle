//! Offline English -> Vietnamese translation for known IslePilot quest strings.
//!
//! The panel serves free-form English prose with no IDs, so translation is
//! keyed on the exact text, two layers deep:
//!
//!   1. compile-time dictionary — the known Prime quest pool, hand-translated
//!      so game terms stay right ("Get nested in" is not machine-translatable);
//!   2. template rules — numeric variants ("Visit 3 Patrol zones").
//!
//! A miss simply stays English — translation must never break the dino panel.

use std::sync::LazyLock;

use regex::Regex;

use crate::islepilot::parser::QuestStatus;

/// Hand-translated Prime quest pool (exact match, trimmed). Keep game terms
/// recognisable — players see the English names in-game and on the panel.
const DICT: &[(&str, &str)] = &[
    (
        "Visit a Sanctuary as a juvenile",
        "Ghé Khu bảo tồn (Sanctuary) khi còn non",
    ),
    ("Get nested in", "Được sinh ra từ tổ (nest)"),
    (
        "Get perfect diet (1% of each)",
        "Đạt chế độ ăn hoàn hảo (mỗi loại 1%)",
    ),
    (
        "Visit Mass Migration zone",
        "Ghé khu Đại di cư (Mass Migration)",
    ),
    ("Never be Infertile", "Không bao giờ bị Vô sinh (Infertile)"),
    (
        "Never get Muscle spasms",
        "Không bao giờ bị Co thắt cơ (Muscle spasms)",
    ),
    ("Raise children to Subadult", "Nuôi con đến Subadult"),
    (
        "Be a Hypsi, Troodon, Beipi, Dryo or Deino",
        "Chơi Hypsi, Troodon, Beipi, Dryo hoặc Deino",
    ),
];

/// Numeric variants: `{n}` is replaced by the captured count.
static TEMPLATES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    [
        (
            r"^Visit (\d+) Migration zones?$",
            "Ghé {n} khu Di cư (Migration)",
        ),
        (
            r"^Visit (\d+) Patrol zones?$",
            "Ghé {n} khu Tuần tra (Patrol)",
        ),
        (
            r"^Visit (\d+) Sanctuar(?:y|ies)$",
            "Ghé {n} Khu bảo tồn (Sanctuary)",
        ),
        (
            r"^Raise (\d+) child(?:ren)? to Subadult$",
            "Nuôi {n} con đến Subadult",
        ),
    ]
    .into_iter()
    .map(|(re, vi)| (Regex::new(re).unwrap(), vi))
    .collect()
});

fn dict_lookup(text: &str) -> Option<&'static str> {
    DICT.iter().find(|(en, _)| *en == text).map(|(_, vi)| *vi)
}

fn template_lookup(text: &str) -> Option<String> {
    for (re, out) in TEMPLATES.iter() {
        if let Some(caps) = re.captures(text) {
            let n = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            return Some(out.replace("{n}", n));
        }
    }
    None
}

// --------------------------------------------------------------- public API ---

/// Dictionary + templates only. This function performs no I/O or networking.
pub fn translate_offline(text: &str) -> Option<String> {
    let key = text.trim();
    if let Some(vi) = dict_lookup(key) {
        return Some(vi.to_string());
    }
    if let Some(vi) = template_lookup(key) {
        return Some(vi);
    }
    None
}

/// Fill `text_vi` only for known strings. Unknown quests remain in English and
/// are never sent to a translation service.
pub fn translate_quests(quests: &mut [QuestStatus], _client: &reqwest::blocking::Client) {
    for quest in quests.iter_mut() {
        if quest.text_vi.is_some() {
            continue;
        }
        if let Some(vi) = translate_offline(&quest.text) {
            quest.text_vi = Some(vi);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ME: &str = include_str!("../fixtures/islepilot/me.html");

    /// Pins the seed dictionary to the real quest pool: every string the
    /// fixture page carries must resolve WITHOUT touching cache or network.
    #[test]
    fn dict_and_templates_cover_the_fixture_pool() {
        let stats = crate::islepilot::parser::parse_me(ME);
        assert_eq!(stats.prime_quests.len(), 10);
        for quest in &stats.prime_quests {
            let hit = dict_lookup(&quest.text).is_some() || template_lookup(&quest.text).is_some();
            assert!(hit, "no offline translation for {:?}", quest.text);
        }
    }

    #[test]
    fn templates_fill_in_the_number() {
        assert_eq!(
            template_lookup("Visit 3 Patrol zones").as_deref(),
            Some("Ghé 3 khu Tuần tra (Patrol)")
        );
        assert_eq!(
            template_lookup("Visit 1 Migration zone").as_deref(),
            Some("Ghé 1 khu Di cư (Migration)")
        );
        assert_eq!(
            template_lookup("Visit 2 Sanctuaries").as_deref(),
            Some("Ghé 2 Khu bảo tồn (Sanctuary)")
        );
        assert_eq!(
            template_lookup("Raise 2 children to Subadult").as_deref(),
            Some("Nuôi 2 con đến Subadult")
        );
        assert_eq!(template_lookup("Visit zones"), None);
        assert_eq!(
            template_lookup("visit 3 patrol zones"),
            None,
            "case matters"
        );
    }

    #[test]
    fn unknown_quests_stay_english_without_network_fallback() {
        assert_eq!(translate_offline("Visit the lake twice"), None);
    }
}
