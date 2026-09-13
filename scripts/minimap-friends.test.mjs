import assert from "node:assert/strict";
import test from "node:test";

import { friendLabelLayout } from "../src/minimap/friend-label.ts";

test("friend names are placed toward the minimap centre", () => {
  const leftMarker = friendLabelLayout(20, 100, 100, 85, 50);
  const rightMarker = friendLabelLayout(180, 100, 100, 85, 50);
  assert.ok(leftMarker.left > 20);
  assert.ok(rightMarker.left + rightMarker.width < 180);
});

test("friend names remain vertically inside the minimap bounds", () => {
  const top = friendLabelLayout(100, 15, 100, 85, 50);
  const bottom = friendLabelLayout(100, 185, 100, 85, 50);
  assert.equal(top.top, 18);
  assert.equal(bottom.top + bottom.height, 182);
});
