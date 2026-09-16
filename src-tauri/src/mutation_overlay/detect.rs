use std::cmp::{max, min};
use std::collections::HashSet;

use super::catalog;

#[derive(Clone, Debug, PartialEq)]
pub struct DetectedMutation {
    pub name_en: String,
    pub description_vi: String,
    pub confidence: f32,
}

fn normalize(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut previous_space = true;
    for ch in value.chars().flat_map(char::to_lowercase) {
        let normalized = match ch {
            '0' => 'o',
            '1' | '|' => 'l',
            c if c.is_ascii_alphanumeric() => c,
            _ => ' ',
        };
        if normalized == ' ' {
            if !previous_space {
                out.push(' ');
                previous_space = true;
            }
        } else {
            out.push(normalized);
            previous_space = false;
        }
    }
    out.trim().to_string()
}

fn compact(value: &str) -> String {
    normalize(value).replace(' ', "")
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != cb);
            current[j + 1] = min(min(current[j] + 1, previous[j + 1] + 1), substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

fn edit_similarity(a: &str, b: &str) -> f32 {
    let a = compact(a);
    let b = compact(b);
    let denominator = max(a.chars().count(), b.chars().count());
    if denominator == 0 {
        return 0.0;
    }
    1.0 - levenshtein(&a, &b) as f32 / denominator as f32
}

fn token_similarity(a: &str, b: &str) -> f32 {
    let a = normalize(a);
    let b = normalize(b);
    let left: HashSet<&str> = a.split_whitespace().filter(|token| token.len() > 1).collect();
    let right: HashSet<&str> = b.split_whitespace().filter(|token| token.len() > 1).collect();
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(&right).count() as f32;
    (2.0 * intersection) / (left.len() + right.len()) as f32
}

fn phrase_score(ocr_text: &str, phrase: &str) -> f32 {
    let text = normalize(ocr_text);
    let phrase_normalized = normalize(phrase);
    if phrase_normalized.len() >= 5 && text.contains(&phrase_normalized) {
        return 1.0;
    }
    if text.len() >= 5 && phrase_normalized.contains(&text) {
        return (text.len() as f32 / phrase_normalized.len() as f32).clamp(0.0, 1.0);
    }

    let token = token_similarity(&text, &phrase_normalized);
    let lengths_are_close = {
        let short = min(text.len(), phrase_normalized.len()) as f32;
        let long = max(text.len(), phrase_normalized.len()) as f32;
        long > 0.0 && short / long >= 0.55
    };
    let edit = if lengths_are_close {
        edit_similarity(&text, &phrase_normalized)
    } else {
        0.0
    };
    token.max(edit)
}

fn to_detected(entry: &catalog::CatalogEntry, confidence: f32) -> DetectedMutation {
    DetectedMutation {
        name_en: entry.name_en.clone(),
        description_vi: entry.description_vi.clone(),
        confidence,
    }
}

fn score_entry(ocr_text: &str, entry: &catalog::CatalogEntry) -> f32 {
    let name_score = phrase_score(ocr_text, &entry.name_en);
    let description_score = entry
        .match_texts
        .iter()
        .map(|description| phrase_score(ocr_text, description))
        .fold(0.0f32, f32::max);

    // A broad Mutation panel can contain several names at once, so this
    // general detector still requires detail evidence.
    if description_score >= 0.90 {
        (0.88 + 0.12 * name_score).min(1.0)
    } else if name_score >= 0.82 && description_score >= 0.42 {
        0.62 * name_score + 0.38 * description_score
    } else {
        0.50 * name_score + 0.30 * description_score
    }
}

pub fn detect_mutation_with_threshold(
    ocr_text: &str,
    threshold: f32,
) -> Option<DetectedMutation> {
    if normalize(ocr_text).len() < 8 {
        return None;
    }

    let mut scored: Vec<(&catalog::CatalogEntry, f32)> = catalog::catalog()
        .iter()
        .map(|entry| (entry, score_entry(ocr_text, entry)))
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));

    let (best, best_score) = scored.first().copied()?;
    let second_score = scored.get(1).map(|(_, score)| *score).unwrap_or(0.0);
    if best_score < threshold || best_score - second_score < 0.03 {
        return None;
    }

    Some(to_detected(best, best_score))
}

/// Detector for the narrow, selected-Mutation title region. Because that
/// region contains only one selected title (not the full list on the left),
/// a strong name match is sufficient and avoids OCR'ing the description.
pub fn detect_mutation_name_with_threshold(
    ocr_text: &str,
    threshold: f32,
) -> Option<DetectedMutation> {
    let normalized = normalize(ocr_text);
    if normalized.len() < 5 {
        return None;
    }

    let mut scored: Vec<(&catalog::CatalogEntry, f32)> = catalog::catalog()
        .iter()
        .map(|entry| (entry, phrase_score(&normalized, &entry.name_en)))
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));

    let (best, best_score) = scored.first().copied()?;
    let second_score = scored.get(1).map(|(_, score)| *score).unwrap_or(0.0);
    if best_score < threshold || best_score - second_score < 0.08 {
        return None;
    }

    Some(to_detected(best, best_score))
}

pub fn detect_mutation(ocr_text: &str) -> Option<DetectedMutation> {
    detect_mutation_with_threshold(ocr_text, 0.82)
}

#[cfg(test)]
mod tests {
    use super::{
        detect_mutation, detect_mutation_name_with_threshold, detect_mutation_with_threshold,
    };

    #[test]
    fn exact_detail_identifies_cellular_regeneration() {
        let found = detect_mutation("Cellular Regeneration\nRecovers health slightly faster")
            .expect("mutation detail should be recognized");
        assert_eq!(found.name_en, "Cellular Regeneration");
        assert!(found.confidence >= 0.95);
        assert_eq!(found.description_vi, "Hồi phục máu nhanh hơn 15%.");
    }

    #[test]
    fn punctuation_and_case_differences_still_match() {
        let found = detect_mutation("FASTER EGG GESTATION, INCUBATION & COOLDOWN RATE")
            .expect("normalized detail should be recognized");
        assert_eq!(found.name_en, "Advanced Gestation");
    }

    #[test]
    fn one_damaged_word_from_ocr_can_still_match() {
        let found = detect_mutation("Recover a small amount of water during the rainy weather or while swirnming in drinkable water")
            .expect("minor OCR damage should still match");
        assert_eq!(found.name_en, "Reabsorption");
    }

    #[test]
    fn common_zero_and_letter_confusion_is_tolerated() {
        let found = detect_mutation("Increased O2 capacity")
            .expect("known game description should be recognized");
        assert_eq!(found.name_en, "Increased Inspiratory Capacity");
        let found = detect_mutation("Increased 02 capacity")
            .expect("OCR zero/O confusion should be recognized");
        assert_eq!(found.name_en, "Increased Inspiratory Capacity");
    }

    #[test]
    fn names_without_detail_do_not_trigger_general_panel_detector() {
        let found = detect_mutation(
            "Cellular Regeneration Advanced Gestation Efficient Digestion Featherweight Wader",
        );
        assert!(found.is_none());
    }

    #[test]
    fn selected_title_detector_accepts_one_mutation_name() {
        let found = detect_mutation_name_with_threshold("Xerocole Adaptation", 0.78)
            .expect("selected title should identify the mutation");
        assert_eq!(found.name_en, "Xerocole Adaptation");
        assert!(found.confidence >= 0.99);
    }

    #[test]
    fn selected_title_detector_tolerates_small_ocr_damage() {
        let found = detect_mutation_name_with_threshold("Xerocole Adaptatlon", 0.78)
            .expect("minor title OCR damage should still match");
        assert_eq!(found.name_en, "Xerocole Adaptation");
    }

    #[test]
    fn selected_title_detector_rejects_multiple_exact_names() {
        assert!(detect_mutation_name_with_threshold(
            "Xerocole Adaptation Hypervigilance",
            0.78,
        )
        .is_none());
    }

    #[test]
    fn unrelated_text_does_not_match() {
        assert!(detect_mutation("Resume Settings Logout Server Browser").is_none());
        assert!(detect_mutation_name_with_threshold("MUTATIONS NEST GROUP", 0.78).is_none());
    }

    #[test]
    fn confidence_threshold_is_respected() {
        assert!(detect_mutation_with_threshold("Recovers health slightly faster", 0.99).is_none());
        assert!(detect_mutation_with_threshold("Recovers health slightly faster", 0.80).is_some());
    }
}
