<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { viethoaLaunch, viethoaStatus, type VietHoaStatus } from "$lib/api";
  import { t } from "$lib/i18n";

  let status = $state<VietHoaStatus | null>(null);
  let loading = $state(true);
  let launching = $state(false);
  let error = $state<string | null>(null);

  async function refresh() {
    loading = true;
    error = null;
    try {
      status = await viethoaStatus();
    } catch {
      error = $t("viethoa.status_error");
    } finally {
      loading = false;
    }
  }

  async function launch() {
    launching = true;
    error = null;
    try {
      status = await viethoaLaunch();
    } catch {
      error = $t("viethoa.launch_error");
    } finally {
      launching = false;
    }
  }

  async function openOfficialRelease() {
    try {
      await openUrl(status?.officialReleaseUrl ?? "https://github.com/ricktanker/dinovietnam-overlay/releases/tag/viethoa");
    } catch {
      error = $t("viethoa.link_error");
    }
  }

  onMount(() => void refresh());
</script>

<div class="mx-auto max-w-3xl space-y-5 p-6">
  <section>
    <div class="eyebrow">THE ISLE // LANGUAGE PACK</div>
    <h2 class="mt-1 text-xl font-semibold" style="color: var(--color-accent)">{$t("viethoa.title")}</h2>
    <p class="mt-1 text-sm" style="color: var(--color-muted)">{$t("viethoa.subtitle")}</p>
  </section>

  <section class="panel p-5">
    {#if loading}
      <p class="text-sm" style="color: var(--color-muted)">{$t("viethoa.checking")}</p>
    {:else if status?.installed}
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <strong>{$t("viethoa.ready")}</strong>
          <p class="mt-1 text-sm" style="color: var(--color-muted)">
            {$t("viethoa.version")}: {status.version ?? "—"}
          </p>
        </div>
        <button class="accent-button" disabled={launching} onclick={() => void launch()}>
          {launching ? $t("viethoa.launching") : $t("viethoa.launch")}
        </button>
      </div>
      <p class="notice mt-5">{$t("viethoa.close_game")}</p>
    {:else}
      <div class="space-y-4">
        <strong>{$t("viethoa.not_installed")}</strong>
        <p class="text-sm leading-6" style="color: var(--color-muted)">{$t("viethoa.official_required")}</p>
        <button class="accent-button" onclick={() => void openOfficialRelease()}>{$t("viethoa.open_official")}</button>
      </div>
    {/if}

    <div class="mt-5 flex gap-2 border-t pt-4" style="border-color: var(--color-border)">
      <button class="control-button" disabled={loading} onclick={() => void refresh()}>{$t("viethoa.refresh")}</button>
      <button class="control-button" onclick={() => void openOfficialRelease()}>{$t("viethoa.source")}</button>
    </div>
    {#if error}<p class="mt-4 text-sm" role="alert" style="color: #ff8a80">{error}</p>{/if}
  </section>
</div>

<style>
  .panel { border: 1px solid var(--color-border); background: var(--color-panel); }
  .eyebrow { color: var(--color-accent); font-size: 10px; letter-spacing: .24em; font-weight: 700; }
  button { cursor: pointer; }
  button:disabled { cursor: not-allowed; opacity: .45; }
  .accent-button, .control-button { border: 1px solid var(--color-border); padding: 8px 14px; font-size: 13px; }
  .accent-button { border-color: var(--color-accent); color: var(--color-accent); background: rgba(53,242,255,.08); }
  .control-button { color: var(--color-text); background: rgba(255,255,255,.025); }
  .notice { border-left: 2px solid #e7ae45; padding: 8px 11px; color: #e9c77d; font-size: 12px; line-height: 1.55; }
</style>
