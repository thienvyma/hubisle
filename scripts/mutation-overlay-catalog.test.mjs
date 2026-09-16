import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { buildCatalogText } from "./generate-mutation-overlay-catalog.mjs";

const generatedPath = new URL(
  "../src-tauri/src/mutation_overlay/catalog.generated.json",
  import.meta.url,
);

const rustSources = [
  new URL("../src-tauri/src/mutation_overlay/mod.rs", import.meta.url),
  new URL("../src-tauri/src/mutation_overlay/detect.rs", import.meta.url),
]
  .map((url) => readFileSync(url, "utf8"))
  .join("\n");

test("generated mutation overlay catalog stays byte-for-byte in sync with mutations.ts", async () => {
  const expected = await buildCatalogText();
  const actual = readFileSync(generatedPath, "utf8");
  assert.equal(actual, expected);

  const parsed = JSON.parse(actual);
  assert.equal(parsed.length, 42);
  assert.equal(new Set(parsed.map((entry) => entry.nameEn)).size, 42);
  assert.ok(parsed.every((entry) => entry.descriptionVi && entry.matchTexts?.length >= 1));
});

test("Vietnamese translation strings are not duplicated by hand in Rust detector code", () => {
  assert.doesNotMatch(rustSources, /Hồi phục|Giảm |Tăng |Động vật|Thanh nước|Thanh thức ăn/);
});
