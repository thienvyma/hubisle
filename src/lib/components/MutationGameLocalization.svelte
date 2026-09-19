<script lang="ts">
  import { onMount } from "svelte";
  import {
    getSettings,
    listenerBag,
    onSettingsChanged,
    patchSettings,
    type Settings,
  } from "$lib/api";
  import {
    DEFAULT_MUTATION_OVERLAY_SETTINGS,
    mutationOverlayBeginCalibration,
    mutationOverlayCancelCalibration,
    mutationOverlayClearManual,
    mutationOverlayPreview,
    mutationOverlaySaveCalibration,
    mutationOverlaySetManual,
    mutationOverlayStatus,
    onMutationOverlayState,
    type NormalizedRect,
    type MutationOverlaySettings,
    type MutationOverlayStatus,
  } from "$lib/mutation-overlay-api";
  import { MUTATION_CATALOG, searchMutations } from "$lib/mutations";

  let overlay = $state<MutationOverlaySettings>({
    ...DEFAULT_MUTATION_OVERLAY_SETTINGS,
    scan_rect: { ...DEFAULT_MUTATION_OVERLAY_SETTINGS.scan_rect },
    description_rect: { ...DEFAULT_MUTATION_OVERLAY_SETTINGS.description_rect },
  });
  let status = $state<MutationOverlayStatus | null>(null);
  let query = $state("");
  let selectedName = $state("Cellular Regeneration");
  let busy = $state(false);
  let error = $state<string | null>(null);

  const matches = $derived(searchMutations(query, "vi").slice(0, 8));
  const calibrating = $derived(status?.state === "calibrating");

  function readRect(
    raw: Partial<NormalizedRect> | undefined,
    fallback: NormalizedRect,
  ): NormalizedRect {
    return {
      x: typeof raw?.x === "number" ? raw.x : fallback.x,
      y: typeof raw?.y === "number" ? raw.y : fallback.y,
      w: typeof raw?.w === "number" ? raw.w : fallback.w,
      h: typeof raw?.h === "number" ? raw.h : fallback.h,
    };
  }

  function readOverlaySettings(settings: Settings): MutationOverlaySettings {
    const raw = (settings as Record<string, unknown>).mutation_overlay as
      | Partial<MutationOverlaySettings>
      | undefined;
    return {
      enabled: raw?.enabled ?? DEFAULT_MUTATION_OVERLAY_SETTINGS.enabled,
      auto_detect: raw?.auto_detect ?? DEFAULT_MUTATION_OVERLAY_SETTINGS.auto_detect,
      confidence_threshold:
        raw?.confidence_threshold ?? DEFAULT_MUTATION_OVERLAY_SETTINGS.confidence_threshold,
      scan_rect: readRect(raw?.scan_rect, DEFAULT_MUTATION_OVERLAY_SETTINGS.scan_rect),
      description_rect: readRect(
        raw?.description_rect,
        DEFAULT_MUTATION_OVERLAY_SETTINGS.description_rect,
      ),
    };
  }

  function statusLabel(value: MutationOverlayStatus | null): string {
    if (!value) return "Đang kiểm tra…";
    switch (value.state) {
      case "disabled": return "ĐÃ TẮT";
      case "waiting-game": return "Đang chờ The Isle";
      case "recognizing": return "Đang nhận diện";
      case "recognized": return "Đã nhận diện";
      case "manual": return "Chọn thủ công";
      case "calibrating": return "Đang căn chỉnh";
      case "needs-calibration": return "Cần căn chỉnh vị trí";
      case "capture-unavailable": return "Không chụp được vùng Mutation";
      case "ocr-unavailable": return "OCR không khả dụng";
      default: return "Đang hoạt động";
    }
  }

  async function refreshStatus() {
    try {
      status = await mutationOverlayStatus();
    } catch {
      status = overlay.enabled
        ? { state: "waiting-game", nameEn: null, confidence: null, message: null }
        : { state: "disabled", nameEn: null, confidence: null, message: null };
    }
  }

  async function patchOverlay(patch: Partial<MutationOverlaySettings>) {
    error = null;
    overlay = {
      ...overlay,
      ...patch,
      scan_rect: patch.scan_rect
        ? { ...overlay.scan_rect, ...patch.scan_rect }
        : overlay.scan_rect,
      description_rect: patch.description_rect
        ? { ...overlay.description_rect, ...patch.description_rect }
        : overlay.description_rect,
    };
    try {
      await patchSettings({ mutation_overlay: patch });
      await refreshStatus();
    } catch (reason) {
      error = String(reason);
    }
  }

  async function manual(nameEn: string) {
    selectedName = nameEn;
    busy = true;
    error = null;
    try {
      status = await mutationOverlaySetManual(nameEn);
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function clearManual() {
    busy = true;
    error = null;
    try {
      status = await mutationOverlayClearManual();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function preview() {
    busy = true;
    error = null;
    try {
      status = await mutationOverlayPreview(selectedName || MUTATION_CATALOG[0].nameEn);
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function calibrate() {
    busy = true;
    error = null;
    try {
      status = await mutationOverlayBeginCalibration();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function saveCalibration() {
    busy = true;
    error = null;
    try {
      status = await mutationOverlaySaveCalibration();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function cancelCalibration() {
    busy = true;
    error = null;
    try {
      status = await mutationOverlayCancelCalibration();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    const bag = listenerBag();
    void getSettings().then((settings) => {
      overlay = readOverlaySettings(settings);
      void refreshStatus();
    });
    void bag.add(onSettingsChanged((settings) => (overlay = readOverlaySettings(settings))));
    void bag.add(onMutationOverlayState((next) => (status = next)));
    return () => bag.dispose();
  });
</script>

<section class="locale-card" aria-label="VIỆT HOÁ THE ISLE MUTATIONS">
  <div class="heading">
    <div>
      <span class="eyebrow">SAFE SCREEN OVERLAY</span>
      <h3>VIỆT HOÁ THE ISLE MUTATIONS</h3>
    </div>
    <span class="status">{statusLabel(status)}</span>
  </div>

  <p class="description">
    Giữ nguyên tên Mutation tiếng Anh và phủ mô tả tiếng Việt trực tiếp lên vùng mô tả trong game.
    Hub chỉ đọc một vùng nhỏ trên màn hình, xử lý trong RAM và không sửa file The Isle.
  </p>

  <div class="switches">
    <label>
      <input
        type="checkbox"
        checked={overlay.enabled}
        onchange={(e) => void patchOverlay({ enabled: e.currentTarget.checked })}
      />
      <strong>BẬT VIỆT HOÁ MUTATIONS</strong>
    </label>
    <label>
      <input
        type="checkbox"
        checked={overlay.auto_detect}
        disabled={!overlay.enabled}
        onchange={(e) => void patchOverlay({ auto_detect: e.currentTarget.checked })}
      />
      TỰ ĐỘNG NHẬN DIỆN
    </label>
  </div>

  {#if status?.nameEn}
    <div class="current">
      <span>ĐANG HIỂN THỊ</span>
      <strong>{status.nameEn}</strong>
      {#if status.confidence !== null}
        <small>{Math.round(status.confidence * 100)}%</small>
      {/if}
    </div>
  {/if}

  {#if status?.message}<p class="notice">{status.message}</p>{/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}

  <div class="actions">
    {#if calibrating}
      <button class="primary" disabled={busy} onclick={() => void saveCalibration()}>LƯU VỊ TRÍ</button>
      <button disabled={busy} onclick={() => void cancelCalibration()}>HUỶ CĂN CHỈNH</button>
    {:else}
      <button class="primary" disabled={busy || !overlay.enabled} onclick={() => void calibrate()}>
        CĂN CHỈNH VỊ TRÍ
      </button>
      <button disabled={busy} onclick={() => void preview()}>HIỆN THỬ</button>
    {/if}
  </div>

  <div class="manual">
    <div class="manual-heading">
      <span>CHỌN THỦ CÔNG</span>
      <button class="clear" disabled={busy} onclick={() => void clearManual()}>XÓA CHỌN</button>
    </div>
    <input class="search" type="search" placeholder="Tìm Mutation…" bind:value={query} />
    <div class="results">
      {#each matches as mutation (mutation.nameEn)}
        <button
          class:selected={selectedName === mutation.nameEn}
          disabled={busy}
          onclick={() => void manual(mutation.nameEn)}
        >
          <strong>{mutation.nameEn}</strong>
          <span>{mutation.descriptionVi}</span>
        </button>
      {/each}
    </div>
  </div>

  <p class="safety">
    Không patch EXE, không inject DLL, không đọc bộ nhớ game, không ghi screenshot ra ổ cứng và không can thiệp EAC.
  </p>
</section>

<style>
  .locale-card { border: 1px solid var(--color-border); border-radius: 4px; background: var(--color-panel); padding: 16px; }
  .heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 14px; }
  .eyebrow { color: var(--color-muted); font: 9px Consolas, monospace; letter-spacing: .18em; }
  h3 { margin: 4px 0 0; color: var(--color-accent); font-size: 14px; letter-spacing: .04em; }
  .status { border: 1px solid var(--color-border); padding: 5px 8px; color: #9fb5c8; font: 9px Consolas, monospace; white-space: nowrap; }
  .description, .notice, .error, .safety { font-size: 11px; line-height: 1.55; }
  .description { margin: 12px 0; color: var(--color-text); }
  .switches { display: grid; gap: 8px; }
  .switches label { display: flex; align-items: center; gap: 8px; font-size: 11px; }
  .current { display: flex; align-items: center; gap: 8px; margin-top: 12px; border: 1px solid rgba(53,242,255,.16); background: rgba(3,12,23,.42); padding: 8px 9px; }
  .current span { color: #617890; font: 8px Consolas, monospace; letter-spacing: .12em; }
  .current strong { color: #c9d9e6; font: 10px Consolas, monospace; }
  .current small { margin-left: auto; color: #7f97aa; font: 9px Consolas, monospace; }
  .notice { margin: 11px 0 0; color: #ffd591; }
  .error { margin: 11px 0 0; color: #ff8a80; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }
  button { border: 1px solid #27425a; background: rgba(7,16,29,.7); color: #b2c8db; padding: 8px 10px; font: 10px Consolas, monospace; cursor: pointer; }
  button:disabled { cursor: not-allowed; opacity: .5; }
  button.primary { border-color: var(--color-accent); background: rgba(53,242,255,.1); color: var(--color-accent); }
  button:hover:not(:disabled) { border-color: var(--color-accent); }
  .manual { margin-top: 14px; }
  .manual-heading { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; color: #7890a5; font: 9px Consolas, monospace; letter-spacing: .11em; }
  .clear { padding: 4px 7px; font-size: 8px; }
  .search { width: 100%; border: 1px solid #27425a; background: rgba(3,12,23,.55); color: #d5e2ec; padding: 8px; font: 10px Consolas, monospace; outline: none; }
  .search:focus { border-color: var(--color-accent); }
  .results { display: grid; max-height: 220px; overflow: auto; margin-top: 7px; gap: 5px; }
  .results button { display: grid; gap: 3px; text-align: left; }
  .results button.selected { border-color: var(--color-accent); background: rgba(53,242,255,.08); }
  .results strong { font-size: 10px; }
  .results span { color: #8299ab; font: 9px/1.4 system-ui, sans-serif; }
  .safety { margin: 12px 0 0; color: #617890; }
  @media (max-width: 640px) { .heading { flex-direction: column; } }
</style>
