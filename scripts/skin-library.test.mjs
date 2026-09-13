import { test } from "node:test";
import assert from "node:assert/strict";
import {
  parseSkinLibrary,
  removeSkinPreset,
  upsertSkinPreset,
} from "../src/lib/skin-library.ts";

test("stored skin presets are normalized and malformed records are discarded", () => {
  const stored = JSON.stringify([
    { name: " Forest ", colors: ["#aabbcc", "#112233"], variation: 1.8 },
    { name: "forest", colors: ["#FFFFFF", "#000000"], variation: 0.2 },
    { name: "Broken", colors: ["not-a-color"], variation: 0 },
    { name: "", colors: ["#123456"], variation: 0 },
  ]);

  assert.deepEqual(parseSkinLibrary(stored), [
    { name: "Forest", colors: ["#AABBCC", "#112233"], variation: 1 },
  ]);
});

test("saving an existing preset name replaces it without changing list order", () => {
  const existing = [
    { name: "Forest", colors: ["#111111"], variation: 0 },
    { name: "Snow", colors: ["#EEEEEE"], variation: 0.2 },
  ];

  assert.deepEqual(upsertSkinPreset(existing, " forest ", ["#abcdef"], 0.75), [
    { name: "forest", colors: ["#ABCDEF"], variation: 0.75 },
    { name: "Snow", colors: ["#EEEEEE"], variation: 0.2 },
  ]);
});

test("deleting a preset is case-insensitive and does not mutate the input", () => {
  const existing = [{ name: "Night", colors: ["#101010"], variation: 0 }];
  const result = removeSkinPreset(existing, "NIGHT");
  assert.deepEqual(result, []);
  assert.equal(existing.length, 1);
});

test("a preset rejects an empty name or invalid palette", () => {
  assert.throws(() => upsertSkinPreset([], " ", ["#123456"], 0), /name/);
  assert.throws(() => upsertSkinPreset([], "Valid", ["bad"], 0), /colors/);
});
