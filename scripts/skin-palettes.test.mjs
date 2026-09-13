import assert from "node:assert/strict";
import test from "node:test";

import {
  presetPalette,
  randomSkinPalette,
  SKIN_PRESET_KEYS,
} from "../src/lib/skin-palettes.ts";

const HEX = /^#[0-9A-F]{6}$/;

test("every quick palette adapts to all provider colour-zone counts", () => {
  for (const key of SKIN_PRESET_KEYS) {
    for (const [provider, count] of [["era", 7], ["titan", 7], ["isle-pilot", 10]]) {
      const palette = presetPalette(key, provider, Array(count).fill("#123456"));
      assert.equal(palette.length, count, `${key}/${provider}`);
      assert.ok(palette.every((color) => HEX.test(color)), `${key}/${provider}`);
    }
  }
});

test("quick and random palettes retain server-locked colour zones", () => {
  const current = Array(10).fill("#123456");
  const locked = new Set([1, 8]);
  const preset = presetPalette("green_black", "isle-pilot", current, locked);
  const random = randomSkinPalette(current, locked, () => 0.42);

  assert.equal(preset[1], "#123456");
  assert.equal(preset[8], "#123456");
  assert.equal(random[1], "#123456");
  assert.equal(random[8], "#123456");
  assert.notEqual(preset[0], "#123456");
  assert.notEqual(random[0], "#123456");
});

test("random palettes return valid uppercase colours without changing size", () => {
  const values = [0.01, 0.17, 0.33, 0.51, 0.72, 0.94];
  let index = 0;
  const palette = randomSkinPalette(Array(13).fill("#808080"), new Set(), () => values[index++ % values.length]);
  assert.equal(palette.length, 13);
  assert.ok(palette.every((color) => HEX.test(color)));
  assert.ok(new Set(palette).size >= 7);
});
