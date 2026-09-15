use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct MutationDescription {
    pub name_en: &'static str,
    pub source: &'static str,
    pub vi: &'static str,
}

// The English source strings are the exact visible mutation-description text
// used by the current public EVRIMA mutation catalog. Canonical mutation names
// are kept only for diagnostics and are NEVER replacement keys.
pub const DESCRIPTIONS: &[MutationDescription] = &[
    MutationDescription { name_en: "Hemomania", source: "Do extra damage on a bleeding target", vi: "Gây thêm sát thương lên mục tiêu đang chảy máu." },
    MutationDescription { name_en: "Hematophagy", source: "Restore some thirst when eating", vi: "Hồi một phần nước khi ăn." },
    MutationDescription { name_en: "Accelerated Prey Drive", source: "Deal more damage to animals with low health", vi: "Gây thêm sát thương lên động vật còn ít máu." },
    MutationDescription { name_en: "Osteophagic", source: "Able to consume bones to regenerate fractures faster", vi: "Có thể ăn xương để hồi phục gãy xương nhanh hơn." },
    MutationDescription { name_en: "Xerocole Adaptation", source: "Gain some water when eating plants", vi: "Nhận một lượng nước khi ăn thực vật." },
    MutationDescription { name_en: "Hypervigilance", source: "Increases camera angles when eating and drinking. Increases footsteps audio from others", vi: "Mở rộng góc nhìn khi ăn và uống, đồng thời nghe tiếng bước chân của sinh vật khác rõ hơn." },
    MutationDescription { name_en: "Truculency", source: "Bucking has a higher chance to dismount latched animals", vi: "Tăng khả năng hất văng sinh vật đang bám vào cơ thể." },
    MutationDescription { name_en: "Photosynthetic Regeneration", source: "Regenerates stamina faster during the day", vi: "Hồi thể lực nhanh hơn vào ban ngày." },
    MutationDescription { name_en: "Cellular Regeneration", source: "Recovers health slightly faster", vi: "Hồi máu nhanh hơn một chút." },
    MutationDescription { name_en: "Advanced Gestation", source: "Faster Egg Gestation/Incubation/Cooldown Rate", vi: "Tăng tốc độ hình thành trứng, ấp trứng và hồi chiêu sinh sản." },
    MutationDescription { name_en: "Sustained Hydration", source: "Your water drains more slowly", vi: "Nước giảm chậm hơn." },
    MutationDescription { name_en: "Enlarged Meniscus", source: "Fall damage hits stamina before draining health, no value", vi: "Sát thương do ngã sẽ trừ thể lực trước khi trừ máu." },
    MutationDescription { name_en: "Efficient Digestion", source: "Your food drains more slowly", vi: "Thức ăn giảm chậm hơn." },
    MutationDescription { name_en: "Featherweight", source: "Your footprints fade much faster", vi: "Dấu chân biến mất nhanh hơn nhiều." },
    MutationDescription { name_en: "Osteosclerosis", source: "Resist or Reduce Fracture damage", vi: "Kháng hoặc giảm sát thương gây gãy xương." },
    MutationDescription { name_en: "Wader", source: "Less hindered when wading through shallow water", vi: "Ít bị cản trở hơn khi lội qua vùng nước nông." },
    MutationDescription { name_en: "Epidermal Fibrosis", source: "Increase bleed resistance", vi: "Tăng khả năng kháng chảy máu." },
    MutationDescription { name_en: "Congenital Hypoalgesia", source: "Reduce incoming damage when fighting larger species", vi: "Giảm sát thương nhận vào khi chiến đấu với loài lớn hơn." },
    MutationDescription { name_en: "Photosynthetic Tissue", source: "Faster health / locked health recovery during the day", vi: "Hồi máu và phần máu bị khóa nhanh hơn vào ban ngày." },
    MutationDescription { name_en: "Nocturnal", source: "Faster health / locked health recovery during the night", vi: "Hồi máu và phần máu bị khóa nhanh hơn vào ban đêm." },
    MutationDescription { name_en: "Hydro-regenerative", source: "Recover health faster during rain", vi: "Hồi máu nhanh hơn khi trời mưa." },
    MutationDescription { name_en: "Increased Inspiratory Capacity", source: "Increased O2 capacity", vi: "Tăng dung tích ôxy." },
    MutationDescription { name_en: "Hydrodynamic", source: "Increased swimming speed", vi: "Tăng tốc độ bơi." },
    MutationDescription { name_en: "Submerged Optical Retention", source: "Increased underwater vision range", vi: "Tăng tầm nhìn dưới nước." },
    MutationDescription { name_en: "Reabsorption", source: "Recover a small amount of water during the rainy weather or while swimming in drinkable water", vi: "Hồi một ít nước khi trời mưa hoặc khi bơi trong nguồn nước có thể uống." },
    MutationDescription { name_en: "Enhanced Digestion", source: "Decrease nutrition decay rate", vi: "Giảm tốc độ hao hụt dinh dưỡng." },
    MutationDescription { name_en: "Reinforced Tendons", source: "Jumping costs less stamina", vi: "Nhảy tiêu hao ít thể lực hơn." },
    MutationDescription { name_en: "Reniculate Kidneys", source: "Can drink saltwater, no value", vi: "Có thể uống nước mặn." },
    MutationDescription { name_en: "Multichambered Lungs", source: "Reduce stamina regeneration threshold", vi: "Giảm ngưỡng cần thiết để bắt đầu hồi thể lực." },
    MutationDescription { name_en: "Infrasound Communication", source: "Make significantly less noise when talking in chat", vi: "Giảm đáng kể tiếng động phát ra khi trò chuyện." },
    MutationDescription { name_en: "Sequential Hermaphroditism", source: "Taking this mutation will change your asset sex. Does not inherit by offspring, no value", vi: "Chọn đột biến này sẽ đổi giới tính nhân vật; không di truyền cho con non." },
    MutationDescription { name_en: "Augmented Tapetum", source: "Have increased vision at night", vi: "Tăng khả năng quan sát vào ban đêm." },
    MutationDescription { name_en: "Cannibalistic", source: "For species that are not, by default, cannibals. Adds their own species as a preferred prey for nutrients, no value", vi: "Cho phép loài vốn không ăn đồng loại xem chính loài của mình là con mồi ưu tiên để nhận dinh dưỡng." },
    MutationDescription { name_en: "Hypermetabolic Inanition", source: "The less hunger you have, the more damage you deal", vi: "Càng đói, bạn càng gây nhiều sát thương." },
    MutationDescription { name_en: "Barometric Sensitivity", source: "Receive an indication prior to storms or droughts, no value", vi: "Nhận được dấu hiệu cảnh báo trước bão hoặc hạn hán." },
    MutationDescription { name_en: "Social Behavior", source: "Increased group size. Only applies to group leader", vi: "Tăng giới hạn số thành viên nhóm; chỉ áp dụng cho trưởng nhóm." },
    MutationDescription { name_en: "Tactile Endurance", source: "Convert incoming damage to stamina", vi: "Chuyển sát thương nhận vào thành hao hụt thể lực." },
    MutationDescription { name_en: "Gastronomic Regeneration", source: "Eating restores a small amount of health", vi: "Ăn sẽ hồi một lượng máu nhỏ." },
    MutationDescription { name_en: "Heightened Ghrelin", source: "Increase overeating capacity by a large amount", vi: "Tăng mạnh khả năng ăn vượt mức no." },
    MutationDescription { name_en: "Prolific Reproduction", source: "Your babies have increased health and stamina regen. Your babies require less food. Your babies grow faster", vi: "Con non hồi máu và thể lực nhanh hơn, cần ít thức ăn hơn và lớn nhanh hơn." },
    MutationDescription { name_en: "Parthenogenesis", source: "Allows the player to nest without a mate, no value", vi: "Cho phép làm tổ mà không cần bạn đời." },
    // This mutation was added after the public catalog above. The entry is
    // intentionally exact-match only: if the shipped game wording differs,
    // it remains untranslated and the coverage counter exposes that fact.
    MutationDescription { name_en: "Traumatic Thrombosis", source: "Prevent death from blood loss while resting", vi: "Ngăn tử vong do mất máu trong khi đang nằm nghỉ." },
];

pub fn description_replacements() -> HashMap<&'static str, &'static str> {
    DESCRIPTIONS
        .iter()
        .map(|entry| (entry.source, entry.vi))
        .collect()
}

pub fn total_descriptions() -> usize {
    DESCRIPTIONS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_no_vietnamese_mutation_name_field_or_name_replacement() {
        let replacements = description_replacements();
        assert_eq!(replacements.len(), DESCRIPTIONS.len());
        for entry in DESCRIPTIONS {
            assert!(!replacements.contains_key(entry.name_en));
            assert!(!entry.source.trim().is_empty());
            assert!(!entry.vi.trim().is_empty());
        }
    }
}
