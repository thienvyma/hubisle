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
  new URL("../src/lib/mutation-overlay-api.ts", import.meta.url),
  "utf8",
);
const mutationWindow = readFileSync(
  new URL("../src-tauri/src/mutation_overlay/window.rs", import.meta.url),
  "utf8",
);

test("mutation translation stays in Settings while Dino keeps its mutation library", () => {
  assert.match(card, /VIỆT HOÁ THE ISLE MUTATIONS/);
  assert.match(dinoTab, /<MutationLibrary\s*\/>/);
  assert.match(dinoTab, /<MutationDetails\s+names=\{player\.mutations\}\s*\/>/);
  assert.doesNotMatch(dinoTab, /MutationGameLocalization/);
  assert.match(settings, /MutationGameLocalization/);
  assert.match(settings, /<MutationGameLocalization\s*\/>/);
});

test("settings card exposes overlay controls instead of localization installer actions", () => {
  assert.match(card, /BẬT VIỆT HOÁ MUTATIONS/);
  assert.match(card, /TỰ ĐỘNG NHẬN DIỆN/);
  assert.match(card, /CĂN CHỈNH VỊ TRÍ/);
  assert.match(card, /HIỆN THỬ/);
  assert.doesNotMatch(card, /CÀI VIỆT HOÁ|CẬP NHẬT GÓI DỊCH|GỠ VIỆT HOÁ|GÓI DỊCH|packVersion/);
});

test("frontend overlay API stays narrow and outside the game process", () => {
  assert.match(api, /mutation_overlay_status/);
  assert.match(api, /mutation_overlay_set_manual/);
  assert.match(api, /mutation_overlay_clear_manual/);
  assert.match(api, /mutation_overlay_begin_calibration/);
  assert.match(api, /mutation_overlay_save_calibration/);
  assert.match(api, /mutation_overlay_cancel_calibration/);
  assert.match(api, /mutation_overlay_preview/);
  assert.doesNotMatch(api, /OpenProcess|ReadProcessMemory|WriteProcessMemory|inject/i);
});

test("mutation overlay keeps WebView2 rendering while covered by The Isle", () => {
  assert.match(mutationWindow, /additional_browser_args\(WEBVIEW_ARGS\)/);
  assert.match(mutationWindow, /--disable-background-timer-throttling/);
  assert.match(mutationWindow, /--disable-backgrounding-occluded-windows/);
  assert.match(mutationWindow, /--disable-renderer-backgrounding/);
  assert.match(mutationWindow, /CalculateNativeWinOcclusion/);
});
