import { test } from "node:test";
import assert from "node:assert/strict";
import { acceptHeading, emptyHeading, headingSourceLabel } from "../src/lib/heading.ts";

const sample = (at, deg = 90, source = "provider-camera") => ({
  headingDeg: deg, headingSource: deg === null ? null : source,
  compassKey: deg === null ? null : "dir.E", headingObservedAtMs: at,
});

test("expiry event clears the bearing even when no position packet follows", () => {
  const current = acceptHeading(emptyHeading(), sample(1000));
  assert.equal(acceptHeading(current, sample(16001, null)).headingDeg, null);
});

test("delayed position response cannot revive a bearing after expiry", () => {
  const expired = acceptHeading(emptyHeading(), sample(16001, null));
  const oldPosition = { ...sample(1000), xCm: 300, yCm: 500 };
  assert.equal(acceptHeading(expired, oldPosition), expired);
});

test("an actual fresh server or local update recovers from expiry", () => {
  const expired = sample(16001, null);
  assert.equal(acceptHeading(expired, sample(17000)).headingDeg, 90);
  assert.equal(acceptHeading(expired, sample(17000, 0, "local-camera")).headingDeg, 0);
});

test("heading events have no position/trail properties", () => {
  const position = { ...sample(1000), xCm: 10, yCm: 20, segments: [[1, 2]] };
  assert.deepEqual(Object.keys(acceptHeading(emptyHeading(), position)).sort(),
    ["headingDeg", "headingObservedAtMs", "headingSource", "compassKey"].sort());
});

test("invalid observation times and non-finite bearings do not corrupt state", () => {
  const current = sample(1000);
  for (const next of [sample(NaN), sample(Infinity), sample(-5), sample(2000, NaN)]) {
    assert.equal(acceptHeading(current, next), current);
  }
});

test("labels do not present movement/body fallbacks as exact camera data", () => {
  assert.equal(headingSourceLabel("provider-camera", "vi"), "Góc server");
  assert.match(headingSourceLabel("movement", "vi"), /≈/);
  assert.equal(headingSourceLabel(null, "en"), "No fresh bearing");
});
