export type GarageParkPhase =
  | "starting"
  | "countdown"
  | "finalizing"
  | "polling"
  | "cancelled"
  | "done";

export interface GarageParkStart {
  pending: boolean;
  delaySec: number;
  commandId: string | number | null;
}

export interface GarageParkProgress {
  phase: GarageParkPhase;
  remainingSec: number;
  totalSec: number;
}

export interface GarageParkDependencies {
  start: () => Promise<GarageParkStart>;
  finalize: () => Promise<string | number>;
  cancel: () => Promise<unknown>;
  wait: (commandId: string) => Promise<unknown>;
  sleep: (milliseconds: number) => Promise<void>;
  isCancelled: () => boolean;
  onProgress: (progress: GarageParkProgress) => void;
}

export interface GarageParkResult {
  cancelled: boolean;
}

function commandId(value: string | number | null | undefined): string | null {
  if (typeof value !== "string" && typeof value !== "number") return null;
  const normalized = String(value).trim();
  return normalized.length > 0 ? normalized : null;
}

function countdownSeconds(value: number): number {
  if (!Number.isFinite(value) || value <= 0) return 0;
  return Math.min(3600, Math.ceil(value));
}

export async function runGaragePark(
  dependencies: GarageParkDependencies,
): Promise<GarageParkResult> {
  const report = (phase: GarageParkPhase, remainingSec = 0, totalSec = 0) =>
    dependencies.onProgress({ phase, remainingSec, totalSec });

  report("starting");
  const start = await dependencies.start();
  const totalSec = countdownSeconds(start.delaySec);

  if ((start.pending || totalSec > 0) && totalSec > 0) {
    for (let remainingSec = totalSec; remainingSec > 0; remainingSec -= 1) {
      if (dependencies.isCancelled()) {
        await dependencies.cancel().catch(() => undefined);
        report("cancelled");
        return { cancelled: true };
      }
      report("countdown", remainingSec, totalSec);
      await dependencies.sleep(1000);
    }

    if (dependencies.isCancelled()) {
      await dependencies.cancel().catch(() => undefined);
      report("cancelled");
      return { cancelled: true };
    }

    report("finalizing");
    let finalizedId: string | null = null;
    let finalError: unknown;
    for (let attempt = 0; attempt < 4; attempt += 1) {
      try {
        finalizedId = commandId(await dependencies.finalize());
        break;
      } catch (error) {
        finalError = error;
        if (attempt === 3) throw error;
        await dependencies.sleep(1200);
      }
    }
    if (!finalizedId) {
      if (finalError) throw finalError;
      throw new Error("The server did not return a garage command id");
    }
    report("polling");
    await dependencies.wait(finalizedId);
  } else {
    const immediateId = commandId(start.commandId);
    if (immediateId) {
      report("polling");
      await dependencies.wait(immediateId);
    }
  }

  report("done");
  return { cancelled: false };
}
