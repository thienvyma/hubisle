<script lang="ts">
  import { onMount } from "svelte";
  import {
    mutationLocaleInstall,
    mutationLocaleStatus,
    mutationLocaleUninstall,
    type MutationLocaleStatus,
  } from "$lib/mutation-locale-api";

  let status = $state<MutationLocaleStatus | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  const installedLike = $derived(
    status !== null && ["installed", "update-available", "incompatible", "corrupt"].includes(status.state),
  );

  const primaryLabel = $derived(
    status?.state === "update-available" || status?.state === "incompatible" || status?.state === "corrupt"
      ? "CẬP NHẬT GÓI DỊCH"
      : "CÀI VIỆT HOÁ",
  );

  const canInstall = $derived(
    status !== null && !["installed", "game-running", "game-not-found"].includes(status.state),
  );

  function stateLabel(value: MutationLocaleStatus | null): string {
    if (!value) return "Đang kiểm tra…";
    switch (value.state) {
      case "installed": return "ĐÃ CÀI ✓";
      case "not-installed": return "CHƯA CÀI";
      case "update-available": return "CÓ BẢN CẬP NHẬT";
      case "incompatible": return "CẦN CẬP NHẬT GÓI DỊCH";
      case "corrupt": return "CẦN SỬA GÓI DỊCH";
      case "game-running": return "HÃY ĐÓNG THE ISLE";
      case "game-not-found": return "KHÔNG TÌM THẤY THE ISLE";
    }
  }

  function stateClass(value: MutationLocaleStatus | null): string {
    if (!value) return "neutral";
    if (value.state === "installed") return "good";
    if (value.state === "not-installed") return "neutral";
    return "warning";
  }

  async function refresh() {
    error = null;
    try {
      status = await mutationLocaleStatus();
    } catch (reason) {
      error = String(reason);
    }
  }

  async function install() {
    busy = true;
    error = null;
    try {
      status = await mutationLocaleInstall();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function uninstall() {
    busy = true;
    error = null;
    try {
      status = await mutationLocaleUninstall();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    void refresh();
  });
</script>

<section class="locale-card" aria-label="VIỆT HOÁ THE ISLE MUTATIONS">
  <div class="heading">
    <div>
      <span class="eyebrow">GAME LOCALIZATION</span>
      <h3>VIỆT HOÁ THE ISLE MUTATIONS</h3>
    </div>
    <span class:good={stateClass(status) === "good"} class:warning={stateClass(status) === "warning"} class="status">
      {stateLabel(status)}
    </span>
  </div>

  <p class="description">
    Dịch phần mô tả Mutation trong The Isle sang tiếng Việt. Tên Mutation vẫn giữ nguyên tiếng Anh.
  </p>

  <div class="meta">
    <div>
      <span>THE ISLE</span>
      <strong>{status?.gamePath ?? "Chưa phát hiện thư mục game"}</strong>
    </div>
    <div>
      <span>GÓI DỊCH</span>
      <strong>v{status?.packVersion ?? "—"}</strong>
    </div>
    <div>
      <span>ĐỘ PHỦ</span>
      <strong>{status ? `${status.matched}/${status.total}` : "—"}</strong>
    </div>
  </div>

  {#if status?.state === "game-running"}
    <p class="notice">Đóng The Isle hoàn toàn rồi bấm kiểm tra lại trước khi cài hoặc gỡ.</p>
  {:else if status?.message}
    <p class="notice">{status.message}</p>
  {/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}

  <div class="actions">
    {#if status?.state === "installed"}
      <button class="installed" disabled>ĐÃ CÀI</button>
    {:else}
      <button class="primary" disabled={busy || !canInstall} onclick={() => void install()}>
        {busy ? "ĐANG XỬ LÝ…" : primaryLabel}
      </button>
    {/if}

    {#if installedLike}
      <button class="danger" disabled={busy} onclick={() => void uninstall()}>GỠ VIỆT HOÁ</button>
    {/if}

    <button class="secondary" disabled={busy} onclick={() => void refresh()}>KIỂM TRA LẠI</button>
  </div>

  <p class="safety">
    Gói chỉ thêm dữ liệu ngôn ngữ do hub quản lý; không sửa file thực thi, không inject DLL và không can thiệp EAC.
  </p>
</section>

<style>
  .locale-card {
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-panel);
    padding: 16px;
  }
  .heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 14px; }
  .eyebrow { color: var(--color-muted); font: 9px Consolas, monospace; letter-spacing: .18em; }
  h3 { margin: 4px 0 0; color: var(--color-accent); font-size: 14px; letter-spacing: .04em; }
  .status { border: 1px solid var(--color-border); padding: 5px 8px; color: var(--color-muted); font: 9px Consolas, monospace; white-space: nowrap; }
  .status.good { border-color: rgba(69,245,162,.42); color: #45f5a2; }
  .status.warning { border-color: rgba(255,200,87,.42); color: #ffc857; }
  .description, .notice, .error, .safety { font-size: 11px; line-height: 1.55; }
  .description { margin: 12px 0; color: var(--color-text); }
  .meta { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; gap: 10px; }
  .meta div { min-width: 0; border: 1px solid rgba(53,242,255,.14); background: rgba(3,12,23,.45); padding: 8px 9px; }
  .meta span { display: block; color: #617890; font: 8px Consolas, monospace; letter-spacing: .12em; }
  .meta strong { display: block; margin-top: 4px; overflow-wrap: anywhere; color: #b9ccdc; font: 10px Consolas, monospace; }
  .notice { margin: 11px 0 0; color: #ffd591; }
  .error { margin: 11px 0 0; color: #ff8a80; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }
  button { border: 1px solid #27425a; background: rgba(7,16,29,.7); color: #b2c8db; padding: 8px 10px; font: 10px Consolas, monospace; cursor: pointer; }
  button:disabled { cursor: not-allowed; opacity: .5; }
  button.primary { border-color: var(--color-accent); background: rgba(53,242,255,.1); color: var(--color-accent); }
  button.installed { border-color: rgba(69,245,162,.45); color: #45f5a2; opacity: 1; }
  button.danger { border-color: rgba(255,86,120,.45); color: #ff8a80; }
  button:hover:not(:disabled) { border-color: var(--color-accent); }
  .safety { margin: 12px 0 0; color: #617890; }
  @media (max-width: 640px) {
    .heading { flex-direction: column; }
    .meta { grid-template-columns: 1fr; }
  }
</style>
