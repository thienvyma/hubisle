import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const card = readFileSync(
  new URL("../src/lib/components/MutationGameLocalization.svelte", import.meta.url),
  "utf8",
);
const dinoTab = readFileSync(
  new URL("../src/main/dino/DinoTab.svelte", import.meta.url),
  "utf8",
);
const api = readFileSync(
  new URL("../src/lib/mutation-locale-api.ts", import.meta.url),
  "utf8",
);
const catalog = readFileSync(
  new URL("../src-tauri/src/mutation_locale/catalog.rs", import.meta.url),
  "utf8",
);

test("the in-hub installer uses the approved title and keeps current mutation UI", () => {
  assert.match(card, /VIỆT HOÁ THE ISLE MUTATIONS/);
  assert.match(dinoTab, /<MutationLibrary\s*\/>/);
  assert.match(dinoTab, /<MutationGameLocalization\s*\/>/);
  assert.match(dinoTab, /<MutationDetails\s+names=\{player\.mutations\}\s*\/>/);
});

test("the frontend exposes only narrow localization commands", () => {
  assert.match(api, /mutation_locale_status/);
  assert.match(api, /mutation_locale_install/);
  assert.match(api, /mutation_locale_uninstall/);
  assert.doesNotMatch(api, /OpenProcess|ReadProcessMemory|WriteProcessMemory|inject/i);
});

test("game pack replacements never translate canonical mutation names", () => {
  assert.match(catalog, /name_en/);
  assert.match(catalog, /description_replacements/);
  assert.doesNotMatch(catalog, /vi_name|name_vi|viName|nameVi/);
  assert.match(catalog, /!replacements\.contains_key\(entry\.name_en\)/);
});
