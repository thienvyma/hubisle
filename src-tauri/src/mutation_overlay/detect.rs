#[cfg(test)]
mod tests {
    use super::detect_mutation;

    #[test]
    fn exact_detail_identifies_cellular_regeneration() {
        let found = detect_mutation("Cellular Regeneration\nRecovers health slightly faster")
            .expect("mutation detail should be recognized");
        assert_eq!(found.name_en, "Cellular Regeneration");
        assert!(found.confidence >= 0.95);
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
    fn names_without_detail_do_not_trigger_translation() {
        let found = detect_mutation(
            "Cellular Regeneration Advanced Gestation Efficient Digestion Featherweight Wader",
        );
        assert!(found.is_none());
    }

    #[test]
    fn unrelated_text_does_not_match() {
        assert!(detect_mutation("Resume Settings Logout Server Browser").is_none());
    }
}
