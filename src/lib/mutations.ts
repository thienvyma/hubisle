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
    descriptionEn: "Deal more damage to animals below 35% health. Value: 10%.",
    descriptionVi: "Gây thêm sát thương lên con mồi còn dưới 35% máu. Giá trị: 10%.",
  },
  {
    nameEn: "Advanced Gestation",
    descriptionEn: "Female only. Faster egg gestation, incubation, and breeding cooldown. Value: 50%.",
    descriptionVi: "Chỉ dành cho con cái. Tăng tốc độ mang trứng, ấp trứng và hồi chiêu sinh sản. Giá trị: 50%.",
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
    descriptionEn: "Recover health slightly faster. Value: 10%.",
    descriptionVi: "Hồi phục máu nhanh hơn một chút. Giá trị: 10%.",
  },
  {
    nameEn: "Congenital Hypoalgesia",
    descriptionEn: "Take less incoming damage when fighting a larger species. Value: 15%.",
    descriptionVi: "Giảm sát thương nhận vào khi chiến đấu với loài lớn hơn. Giá trị: 15%.",
  },
  {
    nameEn: "Efficient Digestion",
    descriptionEn: "Food drains more slowly. Value: 20%.",
    descriptionVi: "Thanh thức ăn giảm chậm hơn. Giá trị: 20%.",
  },
  {
    nameEn: "Enlarged Meniscus",
    descriptionEn: "Fall damage consumes stamina before it starts draining health.",
    descriptionVi: "Sát thương do ngã sẽ trừ thể lực trước khi bắt đầu làm mất máu.",
  },
  {
    nameEn: "Epidermal Fibrosis",
    descriptionEn: "Increase bleed resistance. Value: 15%.",
    descriptionVi: "Tăng khả năng kháng chảy máu. Giá trị: 15%.",
  },
  {
    nameEn: "Featherweight",
    descriptionEn: "Footprints fade faster. Value: 15%.",
    descriptionVi: "Dấu chân biến mất nhanh hơn. Giá trị: 15%.",
  },
  {
    nameEn: "Hematophagy",
    descriptionEn: "Carnivores restore some thirst while eating corpses. Value: 15%.",
    descriptionVi: "Động vật ăn thịt hồi lại một phần nước khi ăn xác. Giá trị: 15%.",
  },
  {
    nameEn: "Hemomania",
    descriptionEn: "Carnivores deal more damage to a bleeding target. Value: 5%.",
    descriptionVi: "Động vật ăn thịt gây thêm sát thương lên mục tiêu đang chảy máu. Giá trị: 5%.",
  },
  {
    nameEn: "Hydrodynamic",
    descriptionEn: "Increase swimming speed. Value: 15%.",
    descriptionVi: "Tăng tốc độ bơi. Giá trị: 15%.",
  },
  {
    nameEn: "Hydro-regenerative",
    descriptionEn: "Recover health faster during rainy weather. Value: 25%.",
    descriptionVi: "Hồi phục máu nhanh hơn khi trời mưa. Giá trị: 25%.",
  },
  {
    nameEn: "Hypervigilance",
    descriptionEn: "Herbivores get wider camera angles while eating or drinking and hear other animals' footsteps more clearly. Value: 50%.",
    descriptionVi: "Động vật ăn cỏ có góc quan sát rộng hơn khi ăn hoặc uống và nghe tiếng chân của sinh vật khác rõ hơn. Giá trị: 50%.",
  },
  {
    nameEn: "Increased Inspiratory Capacity",
    descriptionEn: "Increase oxygen capacity. Value: 15%.",
    descriptionVi: "Tăng dung tích oxy. Giá trị: 15%.",
  },
  {
    nameEn: "Infrasound Communication",
    descriptionEn: "Make less noise when communicating through chat. Value: 50%.",
    descriptionVi: "Giảm tiếng động tạo ra khi giao tiếp qua chat. Giá trị: 50%.",
  },
  {
    nameEn: "Nocturnal",
    descriptionEn: "Recover health and locked health faster at night. Value: 5%.",
    descriptionVi: "Hồi phục máu và máu bị khóa nhanh hơn vào ban đêm. Giá trị: 5%.",
  },
  {
    nameEn: "Osteosclerosis",
    descriptionEn: "Increase fracture resistance. Value: 20%.",
    descriptionVi: "Tăng khả năng kháng gãy xương. Giá trị: 20%.",
  },
  {
    nameEn: "Photosynthetic Regeneration",
    descriptionEn: "Herbivores regenerate stamina faster during the day. Value: 10%.",
    descriptionVi: "Động vật ăn cỏ hồi thể lực nhanh hơn vào ban ngày. Giá trị: 10%.",
  },
  {
    nameEn: "Photosynthetic Tissue",
    descriptionEn: "Recover health and locked health faster during the day. Value: 5%.",
    descriptionVi: "Hồi phục máu và máu bị khóa nhanh hơn vào ban ngày. Giá trị: 5%.",
  },
  {
    nameEn: "Reabsorption",
    descriptionEn: "Recover a small amount of water during rain or while swimming in drinkable water. Value: 1.",
    descriptionVi: "Hồi một lượng nước nhỏ khi trời mưa hoặc khi bơi trong nguồn nước có thể uống. Giá trị: 1.",
  },
  {
    nameEn: "Sequential Hermaphroditism",
    descriptionEn: "Change sex. This mutation is not passed on to offspring.",
    descriptionVi: "Đổi giới tính của khủng long. Mutation này không truyền lại cho con non.",
  },
  {
    nameEn: "Social Behavior",
    descriptionEn: "Herbivore or omnivore group leaders can increase their maximum group size. Value: 1.5.",
    descriptionVi: "Trưởng nhóm của loài ăn cỏ hoặc ăn tạp có thể tăng quy mô tối đa của nhóm. Giá trị: 1,5.",
  },
  {
    nameEn: "Submerged Optical Retention",
    descriptionEn: "Increase underwater vision range. Value: 5%.",
    descriptionVi: "Tăng tầm nhìn dưới nước. Giá trị: 5%.",
  },
  {
    nameEn: "Sustained Hydration",
    descriptionEn: "Water drains more slowly. Value: 20%.",
    descriptionVi: "Thanh nước giảm chậm hơn. Giá trị: 20%.",
  },
  {
    nameEn: "Truculency",
    descriptionEn: "Herbivores have a higher chance to dismount latched attackers when bucking. Value: 5%.",
    descriptionVi: "Động vật ăn cỏ tăng cơ hội hất văng kẻ địch đang bám khi giãy. Giá trị: 5%.",
  },
  {
    nameEn: "Wader",
    descriptionEn: "Reduce the movement penalty from wading through shallow water. Value: 25%.",
    descriptionVi: "Giảm mức cản trở di chuyển khi lội qua vùng nước nông. Giá trị: 25%.",
  },
  {
    nameEn: "Xerocole Adaptation",
    descriptionEn: "Herbivores gain some water when eating plants. Value: 15%.",
    descriptionVi: "Động vật ăn cỏ nhận thêm một phần nước khi ăn thực vật. Giá trị: 15%.",
  },
  {
    nameEn: "Tactile Endurance",
    descriptionEn: "Herbivores restore stamina from a portion of incoming damage. Value: 25%.",
    descriptionVi: "Động vật ăn cỏ hồi thể lực dựa trên một phần sát thương nhận vào. Giá trị: 25%.",
  },
  {
    nameEn: "Traumatic Thrombosis",
    descriptionEn: "Prevent death from blood loss while resting.",
    descriptionVi: "Ngăn tử vong do mất máu trong khi đang nằm nghỉ.",
  },
  {
    nameEn: "Gastronomic Regeneration",
    descriptionEn: "Eating restores a small amount of health. Value: 5%.",
    descriptionVi: "Hồi một lượng máu nhỏ trong khi ăn. Giá trị: 5%.",
  },
  {
    nameEn: "Hypermetabolic Inanition",
    descriptionEn: "Carnivores deal more damage as their hunger gets lower. Value: 15%.",
    descriptionVi: "Động vật ăn thịt gây nhiều sát thương hơn khi thanh thức ăn càng thấp. Giá trị: 15%.",
  },
  {
    nameEn: "Augmented Tapetum",
    descriptionEn: "Carnivores gain increased night vision after killing five players at night. Value: 50%. Unlocks for slot 2.",
    descriptionVi: "Động vật ăn thịt tăng tầm nhìn ban đêm sau khi giết năm người chơi vào ban đêm. Giá trị: 50%. Mở khóa cho ô 2.",
  },
  {
    nameEn: "Enhanced Digestion",
    descriptionEn: "Reduce nutrient decay after maintaining a Good diet from 60% to 80% growth. Value: 10%. Unlocks for slots 2 and 3.",
    descriptionVi: "Giảm tốc độ hao hụt dinh dưỡng sau khi duy trì chế độ ăn Tốt từ 60% đến 80% tăng trưởng. Giá trị: 10%. Mở khóa cho ô 2 và 3.",
  },
  {
    nameEn: "Heightened Ghrelin",
    descriptionEn: "Greatly increase overeating capacity after repeatedly eating past full hunger. Value: 25%. Unlocks for slot 2.",
    descriptionVi: "Tăng mạnh lượng thức ăn có thể ăn vượt mức no sau khi lặp lại việc ăn quá no. Giá trị: 25%. Mở khóa cho ô 2.",
  },
  {
    nameEn: "Multichambered Lungs",
    descriptionEn: "Improve the stamina regeneration threshold after travelling about 2 km. Value: 5%. Unlocks for slots 2 and 3.",
    descriptionVi: "Cải thiện ngưỡng hồi thể lực sau khi di chuyển khoảng 2 km. Giá trị: 5%. Mở khóa cho ô 2 và 3.",
  },
  {
    nameEn: "Osteophagic",
    descriptionEn: "Carnivores can eat bones to regenerate fractures faster. Value: 15%. Unlock by eating bones while fractured.",
    descriptionVi: "Động vật ăn thịt có thể ăn xương để hồi phục gãy xương nhanh hơn. Giá trị: 15%. Mở khóa bằng cách ăn xương khi đang bị gãy xương.",
  },
  {
    nameEn: "Parthenogenesis",
    descriptionEn: "Females can nest without a mate. This mutation unlocks for slot 2 and is not passed on to offspring.",
    descriptionVi: "Con cái có thể làm tổ mà không cần bạn đời. Mutation mở khóa cho ô 2 và không truyền lại cho con non.",
  },
  {
    nameEn: "Prolific Reproduction",
    descriptionEn: "Offspring regenerate health and stamina faster, need less food, and grow faster. Value: 10%. Unlocks for slot 2.",
    descriptionVi: "Con non hồi máu và thể lực nhanh hơn, cần ít thức ăn hơn và trưởng thành nhanh hơn. Giá trị: 10%. Mở khóa cho ô 2.",
  },
  {
    nameEn: "Reinforced Tendons",
    descriptionEn: "Jumping costs less stamina and Pteranodon takeoff costs less stamina. Value: 50%. Unlock by jumping 50 times.",
    descriptionVi: "Nhảy tiêu tốn ít thể lực hơn và Pteranodon cất cánh cũng tốn ít thể lực hơn. Giá trị: 50%. Mở khóa bằng cách nhảy 50 lần.",
  },
  {
    nameEn: "Reniculate Kidneys",
    descriptionEn: "Drink saltwater without the normal penalty. Unlock by drinking saltwater through the Fluid Deficient debuff; available for slots 2 and 3.",
    descriptionVi: "Có thể uống nước mặn mà không chịu hình phạt thông thường. Mở khóa bằng cách tiếp tục uống nước mặn qua trạng thái Fluid Deficient; dùng cho ô 2 và 3.",
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
