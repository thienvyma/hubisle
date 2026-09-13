<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { dinovoiceLaunch, dinovoiceStatus, type DinoVoiceStatus } from "$lib/api";
  import { t } from "$lib/i18n";

  let status = $state<DinoVoiceStatus | null>(null);
  let loading = $state(true);
  let launching = $state(false);
  let error = $state<string | null>(null);

  async function refresh() {
    loading = true;
    error = null;
    try {
      status = await dinovoiceStatus();
    } catch {
      error = $t("voice.status_error");
    } finally {
      loading = false;
    }
  }

  async function launch() {
    launching = true;
    error = null;
    try {
      status = await dinovoiceLaunch();
      window.setTimeout(() => void refresh(), 1200);
    } catch {
      error = $t("voice.launch_error");
    } finally {
      launching = false;
    }
  }

  async function openOfficialRelease() {
    try {
      await openUrl(status?.officialReleaseUrl ?? "https://github.com/ricktanker/dinovietnam-overlay/releases/latest");
    } catch {
      error = $t("voice.link_error");
    }
  }

  onMount(() => void refresh());
</script>

<div class="mx-auto max-w-3xl space-y-5 p-6">
  <section>
    <div class="eyebrow">DINOVIETNAM // LIVE VOICE</div>
    <h2 class="mt-1 text-xl font-semibold" style="color: var(--color-accent)">{$t("voice.title")}</h2>
    <p class="mt-1 text-sm leading-6" style="color: var(--color-muted)">{$t("voice.subtitle")}</p>
  </section>

  <section class="panel p-5">
    {#if loading}
      <p class="text-sm" style="color: var(--color-muted)">{$t("voice.checking")}</p>
    {:else}
      <div class="status-grid">
        <div class:ok={status?.installed} class="status-card">
          <span>{status?.installed ? "●" : "○"}</span>
          <div><small>{$t("voice.client")}</small><strong>{status?.installed ? $t("voice.ready") : $t("voice.missing")}</strong></div>
        </div>
        <div class:ok={status?.steamAuthenticated} class="status-card">
          <span>{status?.steamAuthenticated ? "●" : "○"}</span>
          <div><small>STEAM</small><strong>{status?.steamAuthenticated ? $t("voice.authenticated") : $t("voice.login_needed")}</strong></div>
        </div>
        <div class:ok={status?.running} class="status-card">
          <span>{status?.running ? "●" : "○"}</span>
          <div><small>{$t("voice.process")}</small><strong>{status?.running ? $t("voice.running") : $t("voice.stopped")}</strong></div>
        </div>
      </div>

      {#if status?.installed}
        <p class="mt-5 text-sm leading-6" style="color: var(--color-muted)">
          {status.steamAuthenticated ? $t("voice.ready_hint") : $t("voice.login_hint")}
        </p>
        <button class="accent-button mt-4" disabled={launching} onclick={() => void launch()}>
          {launching ? $t("voice.launching") : status.steamAuthenticated ? $t("voice.open_voice") : $t("voice.open_login")}
        </button>
      {:else}
        <p class="mt-5 text-sm leading-6" style="color: var(--color-muted)">{$t("voice.install_hint")}</p>
        <button class="accent-button mt-4" onclick={() => void openOfficialRelease()}>{$t("voice.download")}</button>
      {/if}

      <p class="notice mt-5">{$t("voice.security_note")}</p>
    {/if}

    <div class="mt-5 flex gap-2 border-t pt-4" style="border-color: var(--color-border)">
      <button class="control-button" disabled={loading} onclick={() => void refresh()}>{$t("voice.refresh")}</button>
      <button class="control-button" onclick={() => void openOfficialRelease()}>{$t("voice.official_source")}</button>
    </div>
    {#if error}<p class="mt-4 text-sm" role="alert" style="color: #ff8a80">{error}</p>{/if}
  </section>
</div>

<style>
  .panel { border: 1px solid var(--color-border); background: var(--color-panel); }
  .eyebrow { color: var(--color-accent); font-size: 10px; letter-spacing: .24em; font-weight: 700; }
  .status-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }
  .status-card { display: flex; align-items: center; gap: 10px; min-height: 72px; border: 1px solid var(--color-border); padding: 12px; color: #708198; background: rgba(255,255,255,.018); }
  .status-card.ok { border-color: rgba(53,242,255,.35); color: var(--color-accent); background: rgba(53,242,255,.045); }
  .status-card small { display: block; margin-bottom: 3px; color: var(--color-muted); font-size: 9px; letter-spacing: .14em; }
  .status-card strong { display: block; color: var(--color-text); font-size: 12px; }
  button { cursor: pointer; }
  button:disabled { cursor: not-allowed; opacity: .45; }
  .accent-button, .control-button { border: 1px solid var(--color-border); padding: 8px 14px; font-size: 13px; }
  .accent-button { border-color: var(--color-accent); color: var(--color-accent); background: rgba(53,242,255,.08); }
  .control-button { color: var(--color-text); background: rgba(255,255,255,.025); }
  .notice { border-left: 2px solid #e7ae45; padding: 8px 11px; color: #e9c77d; font-size: 12px; line-height: 1.55; }
  @media (max-width: 760px) { .status-grid { grid-template-columns: 1fr; } }
</style>
