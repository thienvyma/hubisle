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
const settings = readFileSync(
  new URL("../src/main/settings/Settings.svelte", import.meta.url),
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
const iostore = readFileSync(
  new URL("../src-tauri/src/mutation_locale/iostore.rs", import.meta.url),
  "utf8",
);
const publicPack = readFileSync(
  new URL("../src-tauri/src/mutation_locale/iostore/public_pack.rs", import.meta.url),
  "utf8",
);
const nativeLocale = readFileSync(
  new URL("../src-tauri/src/mutation_locale/mod.rs", import.meta.url),
  "utf8",
);

test("the approved localization installer lives in Settings while Dino keeps its mutation UI", () => {
  assert.match(card, /VIỆT HOÁ THE ISLE MUTATIONS/);
  assert.match(dinoTab, /<MutationLibrary\s*\/>/);
  assert.match(dinoTab, /<MutationDetails\s+names=\{player\.mutations\}\s*\/>/);
  assert.doesNotMatch(dinoTab, /MutationGameLocalization/);
  assert.match(settings, /MutationGameLocalization/);
  assert.match(settings, /<MutationGameLocalization\s*\/>/);
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

test("encrypted EVRIMA localization uses a checksum-verified public locres reference instead of container key recovery", () => {
  assert.match(iostore, /public_pack::build_english_mutation_locres/);
  assert.match(iostore, /pakchunk0-WindowsClient\.utoc/);
  assert.doesNotMatch(iostore, /retoc_cli|RETOC_ZIP_URL|AesKey/);

  assert.match(publicPack, /https:\/\/isle\.klong\.dev\/v1\/releases\/translation\/latest/);
  assert.match(publicPack, /PUBLIC_ARTIFACT_PREFIX/);
  assert.match(publicPack, /TheIsle\/Content\/Localization\/Game\/vi\/Game\.locres/);
  assert.match(publicPack, /sha256/i);
  assert.match(publicPack, /filter_vietnamese_locres/);
  assert.match(publicPack, /entry\.vi/);
  assert.match(publicPack, /entry\.source/);
  assert.match(publicPack, /retain_mut/);

  assert.match(nativeLocale, /collect_iostore_locres/);
});
