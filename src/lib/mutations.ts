import type { Locale } from "./i18n";

export interface MutationEntry {
  nameEn: string;
  descriptionEn: string;
  descriptionVi: string;
}

export interface MutationDisplay {
  known: boolean;
  name: string;
  description: string;
}

export const MUTATION_CATALOG: readonly MutationEntry[] = [
  {
    nameEn: "Accelerated Prey Drive",
    descriptionEn: "Deal 10% more damage to animals below 35% health.",
    descriptionVi: "Gây thêm 10% sát thương lên con mồi còn dưới 35% máu.",
  },
  {
    nameEn: "Advanced Gestation",
    descriptionEn: "Female only. Egg gestation, incubation, and the breeding cooldown are 50% faster.",
    descriptionVi: "Chỉ dành cho con cái. Quá trình mang trứng, ấp trứng và hồi chiêu sinh sản nhanh hơn 50%.",
  },
  {
    nameEn: "Barometric Sensitivity",
    descriptionEn: "Herbivores receive an indication before storms or droughts.",
    descriptionVi: "Động vật ăn cỏ nhận được dấu hiệu cảnh báo trước khi có bão hoặc hạn hán.",
  },
  {
    nameEn: "Cannibalistic",
    descriptionEn: "Carnivore mutation for slots 2 and 4. Species that are not natural cannibals can gain nutrients by eating their own species.",
    descriptionVi: "Mutation dành cho động vật ăn thịt ở ô 2 và 4. Loài vốn không ăn đồng loại có thể nhận dinh dưỡng khi ăn chính loài của mình.",
  },
  {
    nameEn: "Cellular Regeneration",
    descriptionEn: "Recover health 15% faster.",
    descriptionVi: "Hồi phục máu nhanh hơn 15%.",
  },
  {
    nameEn: "Congenital Hypoalgesia",
    descriptionEn: "Take 15% less incoming damage when fighting a larger species.",
    descriptionVi: "Giảm 15% sát thương nhận vào khi chiến đấu với loài lớn hơn.",
  },
  {
    nameEn: "Efficient Digestion",
    descriptionEn: "Food drains 20% more slowly.",
    descriptionVi: "Thanh thức ăn giảm chậm hơn 20%.",
  },
  {
    nameEn: "Enlarged Meniscus",
    descriptionEn: "Fall damage consumes stamina before it starts draining health.",
    descriptionVi: "Sát thương do ngã sẽ trừ thể lực trước khi bắt đầu làm mất máu.",
  },
  {
    nameEn: "Epidermal Fibrosis",
    descriptionEn: "Increase bleed resistance by 15%.",
    descriptionVi: "Tăng 15% khả năng kháng chảy máu.",
  },
  {
    nameEn: "Featherweight",
    descriptionEn: "Footprints fade 50% faster.",
    descriptionVi: "Dấu chân biến mất nhanh hơn 50%.",
  },
  {
    nameEn: "Hematophagy",
    descriptionEn: "Carnivores restore some thirst while eating corpses.",
    descriptionVi: "Động vật ăn thịt hồi lại một phần nước khi ăn xác.",
  },
  {
    nameEn: "Hemomania",
    descriptionEn: "Carnivores deal 5% more damage to a bleeding target.",
    descriptionVi: "Động vật ăn thịt gây thêm 5% sát thương lên mục tiêu đang chảy máu.",
  },
  {
    nameEn: "Hydrodynamic",
    descriptionEn: "Increase swimming speed by 15%.",
    descriptionVi: "Tăng 15% tốc độ bơi.",
  },
  {
    nameEn: "Hydro-regenerative",
    descriptionEn: "Recover health 25% faster during rainy weather.",
    descriptionVi: "Hồi phục máu nhanh hơn 25% khi trời mưa.",
  },
  {
    nameEn: "Hypervigilance",
    descriptionEn: "Herbivores get wider camera angles while eating or drinking and hear other animals' footsteps more clearly.",
    descriptionVi: "Động vật ăn cỏ có góc quan sát rộng hơn khi ăn hoặc uống và nghe tiếng chân của sinh vật khác rõ hơn.",
  },
  {
    nameEn: "Increased Inspiratory Capacity",
    descriptionEn: "Increase oxygen capacity by 15%.",
    descriptionVi: "Tăng 15% dung tích oxy.",
  },
  {
    nameEn: "Infrasound Communication",
    descriptionEn: "Make 50% less noise when communicating through chat.",
    descriptionVi: "Giảm 50% tiếng động tạo ra khi giao tiếp qua chat.",
  },
  {
    nameEn: "Nocturnal",
    descriptionEn: "Recover health and locked health 5% faster at night.",
    descriptionVi: "Hồi phục máu và máu bị khóa nhanh hơn 5% vào ban đêm.",
  },
  {
    nameEn: "Osteosclerosis",
    descriptionEn: "Increase fracture resistance by 20%.",
    descriptionVi: "Tăng 20% khả năng kháng gãy xương.",
  },
  {
    nameEn: "Photosynthetic Regeneration",
    descriptionEn: "Herbivores regenerate stamina 10% faster during the day.",
    descriptionVi: "Động vật ăn cỏ hồi thể lực nhanh hơn 10% vào ban ngày.",
  },
  {
    nameEn: "Photosynthetic Tissue",
    descriptionEn: "Recover health and locked health 5% faster during the day.",
    descriptionVi: "Hồi phục máu và máu bị khóa nhanh hơn 5% vào ban ngày.",
  },
  {
    nameEn: "Reabsorption",
    descriptionEn: "Recover a small amount of water during rain or while swimming in drinkable water.",
    descriptionVi: "Hồi một lượng nước nhỏ khi trời mưa hoặc khi bơi trong nguồn nước có thể uống.",
  },
  {
    nameEn: "Sequential Hermaphroditism",
    descriptionEn: "Change sex. This mutation is not passed on to offspring.",
    descriptionVi: "Đổi giới tính của khủng long. Mutation này không truyền lại cho con non.",
  },
  {
    nameEn: "Social Behavior",
    descriptionEn: "Herbivore or omnivore group leaders can increase their maximum group size.",
    descriptionVi: "Trưởng nhóm của loài ăn cỏ hoặc ăn tạp có thể tăng số thành viên tối đa trong nhóm.",
  },
  {
    nameEn: "Submerged Optical Retention",
    descriptionEn: "Increase underwater vision range by 5%.",
    descriptionVi: "Tăng 5% tầm nhìn dưới nước.",
  },
  {
    nameEn: "Sustained Hydration",
    descriptionEn: "Water drains 20% more slowly.",
    descriptionVi: "Thanh nước giảm chậm hơn 20%.",
  },
  {
    nameEn: "Truculency",
    descriptionEn: "Herbivores have a 5% higher chance to dismount latched attackers when bucking.",
    descriptionVi: "Động vật ăn cỏ tăng 5% cơ hội hất văng kẻ địch đang bám khi giãy.",
  },
  {
    nameEn: "Wader",
    descriptionEn: "Reduce the movement penalty from wading through shallow water by 25%.",
    descriptionVi: "Giảm 25% mức cản trở di chuyển khi lội qua vùng nước nông.",
  },
  {
    nameEn: "Xerocole Adaptation",
    descriptionEn: "Herbivores gain some water when eating plants.",
    descriptionVi: "Động vật ăn cỏ nhận thêm một phần nước khi ăn thực vật.",
  },
  {
    nameEn: "Tactile Endurance",
    descriptionEn: "Herbivores convert incoming damage into stamina loss.",
    descriptionVi: "Động vật ăn cỏ chuyển sát thương nhận vào thành lượng thể lực bị mất.",
  },
  {
    nameEn: "Traumatic Thrombosis",
    descriptionEn: "Prevent death from blood loss while resting.",
    descriptionVi: "Ngăn tử vong do mất máu trong khi đang nằm nghỉ.",
  },
  {
    nameEn: "Gastronomic Regeneration",
    descriptionEn: "Eating restores a small amount of health.",
    descriptionVi: "Hồi một lượng máu nhỏ trong khi ăn.",
  },
  {
    nameEn: "Hypermetabolic Inanition",
    descriptionEn: "Carnivores deal more damage as their hunger gets lower.",
    descriptionVi: "Động vật ăn thịt gây nhiều sát thương hơn khi thanh thức ăn càng thấp.",
  },
  {
    nameEn: "Augmented Tapetum",
    descriptionEn: "Carnivores gain increased night vision after killing five players at night. Unlocks for slot 2.",
    descriptionVi: "Động vật ăn thịt tăng tầm nhìn ban đêm sau khi giết năm người chơi vào ban đêm. Mở khóa cho ô 2.",
  },
  {
    nameEn: "Enhanced Digestion",
    descriptionEn: "Reduce nutrient decay after maintaining nutrients for 60 minutes. Unlocks for slots 2 and 3.",
    descriptionVi: "Giảm tốc độ hao hụt dinh dưỡng sau khi duy trì chất dinh dưỡng trong 60 phút. Mở khóa cho ô 2 và 3.",
  },
  {
    nameEn: "Heightened Ghrelin",
    descriptionEn: "Greatly increase overeating capacity after keeping hunger above 80% for 30 minutes. Unlocks for slot 2.",
    descriptionVi: "Tăng mạnh sức chứa khi ăn quá no sau khi giữ thanh thức ăn trên 80% trong 30 phút. Mở khóa cho ô 2.",
  },
  {
    nameEn: "Multichambered Lungs",
    descriptionEn: "Improve the stamina regeneration threshold after spending 4,500 stamina by sprinting or fast swimming. Unlocks for slots 2 and 3.",
    descriptionVi: "Cải thiện ngưỡng hồi thể lực sau khi tiêu hao 4.500 thể lực bằng chạy nước rút hoặc bơi nhanh. Mở khóa cho ô 2 và 3.",
  },
  {
    nameEn: "Osteophagic",
    descriptionEn: "Carnivores can eat bones to regenerate fractures faster. Unlock by eating bones while fractured.",
    descriptionVi: "Động vật ăn thịt có thể ăn xương để hồi phục gãy xương nhanh hơn. Mở khóa bằng cách ăn xương khi đang bị gãy xương.",
  },
  {
    nameEn: "Parthenogenesis",
    descriptionEn: "Females can nest without a mate. This mutation unlocks for slot 2 and is not passed on to offspring.",
    descriptionVi: "Con cái có thể làm tổ mà không cần bạn đời. Mutation mở khóa cho ô 2 và không truyền lại cho con non.",
  },
  {
    nameEn: "Prolific Reproduction",
    descriptionEn: "Offspring regenerate health and stamina faster, need less food, and grow faster. Unlocks for slot 2.",
    descriptionVi: "Con non hồi máu và thể lực nhanh hơn, cần ít thức ăn hơn và trưởng thành nhanh hơn. Mở khóa cho ô 2.",
  },
  {
    nameEn: "Reinforced Tendons",
    descriptionEn: "Jumping costs less stamina and Pteranodon takeoff costs less stamina. Unlock by jumping 50 times.",
    descriptionVi: "Nhảy tiêu tốn ít thể lực hơn và Pteranodon cất cánh cũng tốn ít thể lực hơn. Mở khóa bằng cách nhảy 50 lần.",
  },
  {
    nameEn: "Reniculate Kidneys",
    descriptionEn: "Drink saltwater without the normal penalty. Unlock by losing 1,250 thirst from drinking saltwater; available for slots 2 and 3.",
    descriptionVi: "Có thể uống nước mặn mà không chịu hình phạt thông thường. Mở khóa sau khi mất 1.250 điểm nước do uống nước mặn; dùng cho ô 2 và 3.",
  },
];

function normalize(value: string): string {
  return value
    .normalize("NFD")
    .replace(/[đĐ]/g, "d")
    .replace(/\p{Diacritic}/gu, "")
    .toLocaleLowerCase("en-US")
    .replace(/[^a-z0-9]/g, "");
}

const MUTATIONS_BY_NAME = new Map(
  MUTATION_CATALOG.map((entry) => [normalize(entry.nameEn), entry]),
);

export function mutationDisplay(rawName: string, locale: Locale): MutationDisplay {
  const trimmedName = rawName.trim();
  const entry = MUTATIONS_BY_NAME.get(normalize(trimmedName));
  if (!entry) {
    return {
      known: false,
      name: trimmedName,
      description:
        locale === "vi"
          ? "Chưa có mô tả tiếng Việt cho mutation này."
          : "No description is available for this mutation yet.",
    };
  }
  return {
    known: true,
    name: entry.nameEn,
    description: locale === "vi" ? entry.descriptionVi : entry.descriptionEn,
  };
}

export function searchMutations(query: string, locale: Locale): readonly MutationEntry[] {
  const needle = normalize(query);
  if (!needle) return MUTATION_CATALOG;
  return MUTATION_CATALOG.filter((entry) =>
    normalize(`${entry.nameEn} ${locale === "vi" ? entry.descriptionVi : entry.descriptionEn}`).includes(
      needle,
    ),
  );
}
