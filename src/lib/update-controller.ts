import { writable, readonly } from "svelte/store";
import type { DownloadEvent, Update } from "@tauri-apps/plugin-updater";

type UpdateHandle = Pick<Update, "version" | "body" | "close" | "downloadAndInstall">;
type Status = "idle" | "checking" | "current" | "available" | "downloading" | "installing" | "check-error" | "install-error";
export interface UpdateState {
  status: Status;
  visible: boolean;
  version: string | null;
  notes: string;
  progress: number | null;
}

// Own the native resource in one place; inject check for offline lifecycle tests.
export function createUpdateController(check: () => Promise<UpdateHandle | null>) {
  let state: UpdateState = { status: "idle", visible: false, version: null, notes: "", progress: null };
  const store = writable(state);
  let handle: UpdateHandle | null = null;
  let disposed = false;
  let revealResult = false;
  let dismissedVersion: string | null = null;
  const set = (patch: Partial<UpdateState>) => {
    if (disposed) return;
    state = { ...state, ...patch };
    store.set(state);
  };
  const close = async (resource: UpdateHandle | null) => {
    try { await resource?.close(); } catch { /* cleanup must not hide the result */ }
  };

  async function checkNow(manual = false) {
    if (disposed || state.status === "downloading" || state.status === "installing") return;
    if (state.status === "checking") {
      if (manual) { revealResult = true; set({ visible: true }); }
      return;
    }
    revealResult = manual;
    set({ status: "checking", visible: manual, progress: null });
    try {
      const next = await check();
      if (disposed) { await close(next); return; }
      const previous = handle;
      handle = next;
      set({
        status: next ? "available" : "current",
        version: next?.version ?? null,
        notes: next?.body ?? "",
        visible: revealResult || (!!next && next.version !== dismissedVersion),
      });
      await close(previous);
    } catch {
      set({ status: "check-error", visible: revealResult });
    }
  }

  async function install() {
    if (disposed || !handle || !["available", "install-error"].includes(state.status)) return;
    const selected = handle;
    let downloaded = 0;
    let total = 0;
    set({ status: "downloading", visible: true, progress: null });
    const onProgress = (event: DownloadEvent) => {
      if (event.event === "Started") {
        total = event.data.contentLength ?? 0;
        downloaded = 0;
        set({ progress: total > 0 ? 0 : null });
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        set({ progress: total > 0 ? Math.min(99, Math.round(downloaded / total * 100)) : null });
      } else {
        set({ status: "installing", progress: 100 });
      }
    };
    try {
      await selected.downloadAndInstall(onProgress);
      set({ status: "installing", progress: 100 });
    } catch {
      set({ status: "install-error", progress: null });
    }
  }

  return {
    state: readonly(store), checkNow, install,
    dismiss() {
      if (["downloading", "installing"].includes(state.status)) return;
      dismissedVersion = state.version;
      revealResult = false;
      set({ visible: false });
    },
    async dispose() {
      disposed = true;
      await close(handle);
      handle = null;
    },
  };
}
