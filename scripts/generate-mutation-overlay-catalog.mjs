import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = fileURLToPath(new URL("../", import.meta.url));
const SOURCE = path.join(ROOT, "src", "lib", "mutations.ts");
const OUTPUT = path.join(
  ROOT,
  "src-tauri",
  "src",
  "mutation_overlay",
  "catalog.generated.json",
);

const GAME_ALIASES = Object.freeze({
  "Accelerated Prey Drive": ["Deal more damage to animals with low health"],
  "Advanced Gestation": ["Faster Egg Gestation/Incubation/Cooldown Rate"],
  "Cellular Regeneration": ["Recovers health slightly faster"],
  "Congenital Hypoalgesia": ["Reduce incoming damage when fighting larger species"],
  "Efficient Digestion": ["Your food drains more slowly"],
  "Enlarged Meniscus": ["Fall damage hits stamina before draining health, no value"],
  "Epidermal Fibrosis": ["Increase bleed resistance"],
  Featherweight: ["Your footprints fade much faster"],
  Hematophagy: ["Restore some thirst when eating"],
  Hemomania: ["Do extra damage on a bleeding target"],
  Hydrodynamic: ["Increased swimming speed"],
  "Hydro-regenerative": ["Recover health faster during rain"],
  Hypervigilance: [
    "Increases camera angles when eating and drinking. Increases footsteps audio from others",
  ],
  "Increased Inspiratory Capacity": ["Increased O2 capacity"],
  "Infrasound Communication": ["Make significantly less noise when talking in chat"],
  Nocturnal: ["Faster health / locked health recovery during the night"],
  Osteophagic: ["Able to consume bones to regenerate fractures faster"],
  Osteosclerosis: ["Resist or Reduce Fracture damage"],
  "Photosynthetic Regeneration": ["Regenerates stamina faster during the day"],
  "Photosynthetic Tissue": ["Faster health / locked health recovery during the day"],
  Reabsorption: [
    "Recover a small amount of water during the rainy weather or while swimming in drinkable water",
  ],
  "Reinforced Tendons": ["Jumping costs less stamina"],
  "Reniculate Kidneys": ["Can drink saltwater, no value"],
  "Multichambered Lungs": ["Reduce stamina regeneration threshold"],
  "Submerged Optical Retention": ["Increased underwater vision range"],
  "Sustained Hydration": ["Your water drains more slowly"],
  Truculency: ["Bucking has a higher chance to dismount latched animals"],
  Wader: ["Less hindered when wading through shallow water"],
  "Xerocole Adaptation": ["Gain some water when eating plants"],
  "Enhanced Digestion": ["Decrease nutrition decay rate"],
});

function decodeString(literal) {
  return JSON.parse(literal);
}

export function parseMutationCatalog(source) {
  const string = String.raw`"(?:\\.|[^"\\])*"`;
  const pattern = new RegExp(
    String.raw`\{\s*nameEn:\s*(${string}),\s*descriptionEn:\s*(${string}),\s*descriptionVi:\s*(${string}),\s*\}`,
    "g",
  );
  const entries = [];
  for (const match of source.matchAll(pattern)) {
    entries.push({
      nameEn: decodeString(match[1]),
      descriptionEn: decodeString(match[2]),
      descriptionVi: decodeString(match[3]),
    });
  }
  return entries;
}

export async function buildCatalog() {
  const source = await readFile(SOURCE, "utf8");
  const parsed = parseMutationCatalog(source);
  if (parsed.length !== 42) {
    throw new Error(`expected 42 mutations in ${SOURCE}, found ${parsed.length}`);
  }
  const seen = new Set();
  return parsed
    .map((entry) => {
      if (seen.has(entry.nameEn)) throw new Error(`duplicate mutation: ${entry.nameEn}`);
      seen.add(entry.nameEn);
      return {
        nameEn: entry.nameEn,
        descriptionVi: entry.descriptionVi,
        matchTexts: [...new Set([entry.descriptionEn, ...(GAME_ALIASES[entry.nameEn] ?? [])])],
      };
    })
    .sort((a, b) => a.nameEn.localeCompare(b.nameEn, "en"));
}

export async function buildCatalogText() {
  return `${JSON.stringify(await buildCatalog(), null, 2)}\n`;
}

async function main() {
  await writeFile(OUTPUT, await buildCatalogText(), "utf8");
  console.log(`wrote ${OUTPUT}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  await main();
}
