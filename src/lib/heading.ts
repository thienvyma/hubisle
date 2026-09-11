export interface HeadingUpdate {
  headingDeg: number | null;
  headingSource: "local-camera" | "provider-camera" | "movement" | null;
  compassKey: string | null;
  headingObservedAtMs: number;
}

export const emptyHeading = (): HeadingUpdate => ({
  headingDeg: null, headingSource: null, compassKey: null, headingObservedAtMs: -1,
});

/** Both position responses and heading events carry an observation time.
 * An older HTTP/IPC completion must not restore an already-expired bearing. */
export function acceptHeading(current: HeadingUpdate, next: HeadingUpdate): HeadingUpdate {
  if (!Number.isFinite(next.headingObservedAtMs)
      || next.headingObservedAtMs < 0
      || next.headingObservedAtMs < current.headingObservedAtMs) return current;
  if (next.headingDeg !== null && !Number.isFinite(next.headingDeg)) return current;
  return {
    headingDeg: next.headingDeg,
    headingSource: next.headingSource,
    compassKey: next.compassKey,
    headingObservedAtMs: next.headingObservedAtMs,
  };
}

export function headingSourceLabel(source: HeadingUpdate["headingSource"], lang: "vi" | "en"): string {
  if (source === "local-camera") return lang === "vi" ? "Nguồn cục bộ" : "Local source";
  // Some provider fallbacks contain body yaw, not camera yaw.
  if (source === "provider-camera") return lang === "vi" ? "Góc server" : "Server bearing";
  if (source === "movement") return lang === "vi" ? "Hướng di chuyển ≈" : "Travel direction ≈";
  return lang === "vi" ? "Chưa có góc mới" : "No fresh bearing";
}
