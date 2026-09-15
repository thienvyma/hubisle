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
    description: "Hồi phục máu nhanh hơn 15%.",
  });
});

test("English display uses the English detail without changing the name", () => {
  const display = mutationModule.mutationDisplay?.("Hydro regenerative", "en");
  assert.deepEqual(display, {
    known: true,
    name: "Hydro-regenerative",
    description: "Recover health 25% faster during rainy weather.",
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
