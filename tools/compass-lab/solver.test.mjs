import { test } from "node:test";
import assert from "node:assert/strict";
import { solveCompassFrame, validCalibration, MAX_FRAME_AGE_MS } from "./solver.mjs";

// SYNTHETIC calibration. Not measured from The Isle and not a shipping default.
const cal = {
  width: 1920, height: 1080,
  roi: { x: 600, y: 90, width: 720, height: 40 }, centreU: 0.5,
  samples: [{ u: 0, offsetDeg: -90 }, { u: 0.5, offsetDeg: 0 }, { u: 1, offsetDeg: 90 }],
};
const mark = (kind, u, matchScore = 1) => ({ kind, x: cal.roi.x + u * cal.roi.width, y: 110, matchScore });
const frame = (...markers) => ({ width: 1920, height: 1080, capturedAtMs: 1000, markers });
const solve = (...markers) => solveCompassFrame(frame(...markers), cal, 1010);
const close = (actual, expected) => assert.ok(Math.abs(actual - expected) < 1e-8, `${actual} != ${expected}`);

test("centred north/south are absolute compass bearings, with no provider yaw offset", () => {
  close(solve(mark("north", 0.5)).bearingDeg, 0);
  close(solve(mark("south", 0.5)).bearingDeg, 180);
});

test("both cardinals agree when looking east or west in the synthetic projection", () => {
  close(solve(mark("north", 0), mark("south", 1)).bearingDeg, 90);
  close(solve(mark("south", 0), mark("north", 1)).bearingDeg, 270);
});

test("intermediate bearings and 359/0 crossing do not jump to the opposite direction", () => {
  close(solve(mark("north", 0.25)).bearingDeg, 45);
  close(solve(mark("north", 0.75)).bearingDeg, 315);
  close(solve(mark("north", 0.5 + 1 / 180)).bearingDeg, 359);
  close(solve(mark("north", 0.5 - 1 / 180)).bearingDeg, 1);
  close(solve(mark("north", 0.5 - 1 / 180), mark("north", 0.5 + 1 / 180)).bearingDeg, 0);
});

test("a measured nonlinear table is used rather than assuming a 360-degree strip", () => {
  const nonlinear = { ...cal, samples: [
    { u: 0, offsetDeg: -80 }, { u: 0.25, offsetDeg: -25 },
    { u: 0.5, offsetDeg: 0 }, { u: 0.75, offsetDeg: 25 }, { u: 1, offsetDeg: 80 },
  ] };
  close(solveCompassFrame(frame(mark("north", 0.25)), nonlinear, 1010).bearingDeg, 25);
  close(solveCompassFrame(frame(mark("south", 0.625)), nonlinear, 1010).bearingDeg, 167.5);
});

test("missing Q, ambiguous east/west waves, poor matches and scent icons do not invent bearings", () => {
  for (const marks of [[], [mark("wave", 0.5)], [mark("water", 0.5)],
    [mark("east-west", 0.5)], [mark("north", 0.5, 0.6)], [mark("north", 0.5, NaN)]]) {
    assert.equal(solve(...marks).reason, "no-readable-cardinal");
  }
});

test("contradictory north and south are rejected instead of averaged", () => {
  assert.equal(solve(mark("north", 0.5), mark("south", 0.5)).reason, "conflicting-cardinals");
  assert.equal(solve(mark("north", 0), mark("south", 0.9)).reason, "conflicting-cardinals");
});

test("frozen capture expires using capture time, even if the consumer keeps running", () => {
  const frozen = frame(mark("north", 0.5));
  assert.equal(solveCompassFrame(frozen, cal, 1000 + MAX_FRAME_AGE_MS).status, "valid");
  assert.equal(solveCompassFrame(frozen, cal, 1001 + MAX_FRAME_AGE_MS).reason, "stale-frame");
  assert.equal(solveCompassFrame(frozen, cal, 999).reason, "stale-frame");
  assert.equal(solveCompassFrame(frozen, cal, NaN).reason, "stale-frame");
});

test("fresh frames of an unchanged heading are valid; identical pixels are not a stall signal", () => {
  const fresh = { ...frame(mark("north", 0.5)), capturedAtMs: 5000 };
  assert.equal(solveCompassFrame(fresh, cal, 5010).status, "valid");
});

test("resize requires calibration, and markers outside the ROI are ignored", () => {
  assert.equal(solveCompassFrame({ ...frame(), width: 2560 }, cal, 1010).reason, "viewport-changed");
  assert.equal(solve({ ...mark("north", 0.5), y: 500 }).reason, "no-readable-cardinal");
  assert.equal(solve(mark("north", -0.01)).reason, "no-readable-cardinal");
  assert.equal(solve({ ...mark("north", 0.5), x: Infinity }).reason, "no-readable-cardinal");
});

test("no extrapolation outside measured coverage", () => {
  const partial = { ...cal, samples: [
    { u: 0.25, offsetDeg: -40 }, { u: 0.5, offsetDeg: 0 }, { u: 0.75, offsetDeg: 40 },
  ] };
  assert.equal(solveCompassFrame(frame(mark("north", 0)), partial, 1010).reason, "no-readable-cardinal");
});

test("missing, degenerate or backwards calibration is rejected", () => {
  assert.equal(validCalibration(cal), true);
  const invalid = [null, {}, { ...cal, centreU: NaN }, { ...cal, roi: { ...cal.roi, width: 0 } },
    { ...cal, roi: { ...cal.roi, x: 1900 } }, { ...cal, samples: cal.samples.toReversed() },
    { ...cal, samples: [cal.samples[0], cal.samples[1], cal.samples[1]] },
    { ...cal, samples: [{ u: 0, offsetDeg: -90 }, { u: 0.5, offsetDeg: 5 }, { u: 1, offsetDeg: 90 }] },
  ];
  for (const item of invalid) {
    assert.equal(validCalibration(item), false);
    assert.equal(solveCompassFrame(frame(), item, 1010).reason, "uncalibrated");
  }
});
