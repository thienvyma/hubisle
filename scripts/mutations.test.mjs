import assert from "node:assert/strict";
import test from "node:test";

const mutationModule = await import("../src/lib/mutations.ts").catch(() => ({}));

test("the offline catalog exposes every current public Evrima mutation", () => {
  const catalog = mutationModule.MUTATION_CATALOG;
  assert.ok(Array.isArray(catalog), "the mutation catalog must be exported");
  assert.equal(catalog.length, 42);

  const normalizedNames = catalog.map((entry) =>
    entry.nameEn.toLocaleLowerCase("en-US").replace(/[^a-z0-9]/g, ""),
  );
  assert.equal(new Set(normalizedNames).size, catalog.length, "English names must be unique");
  assert.ok(catalog.every((entry) => entry.descriptionEn.trim().length > 0));
  assert.ok(catalog.every((entry) => entry.descriptionVi.trim().length > 0));
  assert.ok(catalog.every((entry) => !("nameVi" in entry)), "mutation names must never be translated");
});

test("Vietnamese display keeps the canonical English mutation name", () => {
  const display = mutationModule.mutationDisplay?.("  cellular_regeneration  ", "vi");
  assert.deepEqual(display, {
    known: true,
    name: "Cellular Regeneration",
    description: "Hồi phục máu nhanh hơn một chút. Giá trị: 10%.",
  });
});

function normalizedNumbers(value) {
  return [...value.matchAll(/\d+(?:[.,]\d+)?%?/g)].map((match) => {
    const raw = match[0];
    const suffix = raw.endsWith("%") ? "%" : "";
    const numeric = suffix ? raw.slice(0, -1) : raw;
    const separator = numeric.match(/[.,]/)?.[0];
    if (!separator) return `${numeric}${suffix}`;
    const [left, right] = numeric.split(separator);
    return right.length === 3
      ? `${left}${right}${suffix}`
      : `${left}.${right}${suffix}`;
  });
}

test("Vietnamese descriptions preserve every numeric value from English", () => {
  for (const mutation of mutationModule.MUTATION_CATALOG) {
    assert.deepEqual(
      normalizedNumbers(mutation.descriptionVi),
      normalizedNumbers(mutation.descriptionEn),
      mutation.nameEn,
    );
  }
});

test("current Evrima fallback effect values stay pinned", () => {
  const expected = new Map([
    ["Accelerated Prey Drive", "10%"],
    ["Advanced Gestation", "50%"],
    ["Cellular Regeneration", "10%"],
    ["Congenital Hypoalgesia", "15%"],
    ["Efficient Digestion", "20%"],
    ["Epidermal Fibrosis", "15%"],
    ["Featherweight", "15%"],
    ["Hematophagy", "15%"],
    ["Hemomania", "5%"],
    ["Hydrodynamic", "15%"],
    ["Hydro-regenerative", "25%"],
    ["Hypervigilance", "50%"],
    ["Increased Inspiratory Capacity", "15%"],
    ["Infrasound Communication", "50%"],
    ["Nocturnal", "5%"],
    ["Osteosclerosis", "20%"],
    ["Photosynthetic Regeneration", "10%"],
    ["Photosynthetic Tissue", "5%"],
    ["Reabsorption", "1"],
    ["Social Behavior", "1.5"],
    ["Submerged Optical Retention", "5%"],
    ["Sustained Hydration", "20%"],
    ["Truculency", "5%"],
    ["Wader", "25%"],
    ["Xerocole Adaptation", "15%"],
    ["Tactile Endurance", "25%"],
    ["Gastronomic Regeneration", "5%"],
    ["Hypermetabolic Inanition", "15%"],
    ["Augmented Tapetum", "50%"],
    ["Enhanced Digestion", "10%"],
    ["Heightened Ghrelin", "25%"],
    ["Multichambered Lungs", "5%"],
    ["Osteophagic", "15%"],
    ["Prolific Reproduction", "10%"],
    ["Reinforced Tendons", "50%"],
  ]);

  const catalog = new Map(
    mutationModule.MUTATION_CATALOG.map((mutation) => [mutation.nameEn, mutation]),
  );
  for (const [name, value] of expected) {
    const mutation = catalog.get(name);
    assert.ok(mutation, name);
    const escapedValue = value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    assert.match(mutation.descriptionEn, new RegExp(`Value: ${escapedValue}`));
    const viValue = value === "1.5" ? "1,5" : value;
    const escapedViValue = viValue.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    assert.match(mutation.descriptionVi, new RegExp(`Giá trị: ${escapedViValue}`));
  }
});

test("English display uses the English detail without changing the name", () => {
  const display = mutationModule.mutationDisplay?.("Hydro regenerative", "en");
  assert.deepEqual(display, {
    known: true,
    name: "Hydro-regenerative",
    description: "Recover health faster during rainy weather. Value: 25%.",
  });
});

test("unknown server mutations preserve their raw English name", () => {
  assert.deepEqual(mutationModule.mutationDisplay?.("Future Mutation", "vi"), {
    known: false,
    name: "Future Mutation",
    description: "Chưa có mô tả tiếng Việt cho mutation này.",
  });
  assert.deepEqual(mutationModule.mutationDisplay?.("Future Mutation", "en"), {
    known: false,
    name: "Future Mutation",
    description: "No description is available for this mutation yet.",
  });
});

test("the library can search English names and Vietnamese details", () => {
  assert.deepEqual(
    mutationModule.searchMutations?.("saltwater", "en").map((entry) => entry.nameEn),
    ["Reniculate Kidneys"],
  );
  assert.deepEqual(
    mutationModule.searchMutations?.("nước mặn", "vi").map((entry) => entry.nameEn),
    ["Reniculate Kidneys"],
  );
});
