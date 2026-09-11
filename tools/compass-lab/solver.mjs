/**
 * OFFLINE RESEARCH ONLY. Converts already-identified Q-compass marks to a
 * bearing. This does not capture a window or detect pixels, and is not wired
 * into the app. No shipping calibration has been established for Evrima.
 *
 * A measured lookup table is mandatory: neither a 360-degree strip nor a
 * linear pixel-to-angle projection is assumed. Offsets increase to the right.
 */
export const MAX_FRAME_AGE_MS = 250;
const MIN_MATCH_SCORE = 0.85;
const MAX_DISAGREEMENT_DEG = 5;

const finite = Number.isFinite;
const wrap = (angle) => ((angle % 360) + 360) % 360;
const difference = (a, b) => wrap(a - b + 180) - 180;
const unknown = (reason) => ({ status: "unknown", reason });

export function validCalibration(cal) {
  if (!cal || !Number.isInteger(cal.width) || !Number.isInteger(cal.height)
      || cal.width <= 0 || cal.height <= 0) return false;
  const roi = cal.roi;
  if (!roi || ![roi.x, roi.y, roi.width, roi.height].every(finite)
      || roi.x < 0 || roi.y < 0 || roi.width <= 0 || roi.height <= 0
      || roi.x + roi.width > cal.width || roi.y + roi.height > cal.height) return false;
  if (!finite(cal.centreU) || cal.centreU <= 0 || cal.centreU >= 1) return false;
  const samples = cal.samples;
  if (!Array.isArray(samples) || samples.length < 3 || samples.length > 1000) return false;
  if (!samples.some((s) => s?.u === cal.centreU && s?.offsetDeg === 0)) return false;
  return samples.every((s, index) => {
    if (!s || !finite(s.u) || !finite(s.offsetDeg)
        || s.u < 0 || s.u > 1 || s.offsetDeg < -180 || s.offsetDeg > 180) return false;
    return index === 0 || (s.u > samples[index - 1].u
      && s.offsetDeg > samples[index - 1].offsetDeg);
  });
}

// Interpolate only inside measured coverage. Do not extrapolate at the ends:
// an unseen marker must never be clamped into a plausible-looking direction.
function offsetAt(u, samples) {
  if (!finite(u) || u < samples[0].u || u > samples.at(-1).u) return null;
  for (let i = 1; i < samples.length; i++) {
    const left = samples[i - 1];
    const right = samples[i];
    if (u <= right.u) {
      const t = (u - left.u) / (right.u - left.u);
      return left.offsetDeg + t * (right.offsetDeg - left.offsetDeg);
    }
  }
  return null;
}

/**
 * frame.markers: [{ kind: "north" | "south", x, y, matchScore }].
 * x/y are in window-frame pixels; the upstream detector must distinguish an
 * actual pointed compass mark from waves/icons. matchScore is a detector
 * score, NOT a calibrated probability. capturedAtMs and nowMs share a clock.
 * Output is compass bearing (N=0, E=90), never provider yaw (+90 conversion).
 */
export function solveCompassFrame(frame, cal, nowMs) {
  if (!validCalibration(cal)) return unknown("uncalibrated");
  if (!frame || frame.width !== cal.width || frame.height !== cal.height) {
    return unknown("viewport-changed");
  }
  if (!finite(nowMs) || !finite(frame.capturedAtMs)
      || nowMs < frame.capturedAtMs || nowMs - frame.capturedAtMs > MAX_FRAME_AGE_MS) {
    return unknown("stale-frame");
  }
  if (!Array.isArray(frame.markers) || frame.markers.length > 32) return unknown("invalid-markers");
  const headings = [];
  for (const mark of frame.markers) {
    // E/W waveform junctions are visually ambiguous without additional
    // evidence; this prototype deliberately accepts only identified N/S.
    if (!mark || !["north", "south"].includes(mark.kind)) continue;
    if (!finite(mark.matchScore) || mark.matchScore < MIN_MATCH_SCORE || mark.matchScore > 1) continue;
    if (!finite(mark.x) || !finite(mark.y)) continue;
    const roi = cal.roi;
    if (mark.x < roi.x || mark.x > roi.x + roi.width
        || mark.y < roi.y || mark.y > roi.y + roi.height) continue;
    const u = (mark.x - roi.x) / roi.width;
    const offset = offsetAt(u, cal.samples);
    if (offset === null) continue;
    const cardinal = mark.kind === "north" ? 0 : 180;
    headings.push(wrap(cardinal - offset));
  }
  if (!headings.length) return unknown("no-readable-cardinal");
  // Reject contradictory detections instead of averaging a false mark into
  // the result. Circular differences also handle the 359 -> 0 wrap correctly.
  for (let i = 0; i < headings.length; i++) {
    for (let j = i + 1; j < headings.length; j++) {
      if (Math.abs(difference(headings[i], headings[j])) > MAX_DISAGREEMENT_DEG) {
        return unknown("conflicting-cardinals");
      }
    }
  }
  const anchor = headings[0];
  const bearingDeg = wrap(anchor + headings.reduce((sum, h) => sum + difference(h, anchor), 0) / headings.length);
  return {
    status: "valid",
    bearingDeg,
    markerCount: headings.length,
    maxResidualDeg: Math.max(...headings.map((h) => Math.abs(difference(h, bearingDeg)))),
  };
}
