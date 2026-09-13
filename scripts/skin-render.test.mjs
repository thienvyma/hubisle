import assert from "node:assert/strict";
import test from "node:test";

import { hexToRgb } from "../src/lib/dino3d/color.ts";

test("skin palette colours reach the compositor without brightness loss", () => {
  assert.deepEqual(hexToRgb("#A1B2C3"), [161, 178, 195]);
  assert.deepEqual(hexToRgb("#000000"), [0, 0, 0]);
  assert.deepEqual(hexToRgb("ffffff"), [255, 255, 255]);
});

test("malformed skin colours are rejected before compositing", () => {
  assert.throws(() => hexToRgb("#12345"), /Invalid skin colour/);
  assert.throws(() => hexToRgb("red"), /Invalid skin colour/);
});
