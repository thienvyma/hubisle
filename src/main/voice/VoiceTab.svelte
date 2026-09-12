<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import {
    getSettings,
    islevoipLaunch,
    islevoipStatus,
    patchSettings,
    type IsleVoipStatus,
  } from "$lib/api";
  import { t } from "$lib/i18n";

  const ISLEVOIP_URL = "https://isle-voip.com/";
  let status = $state<IsleVoipStatus | null>(null);
  let loading = $state(true);
  let starting = $state(false);
  let saving = $state(false);
  let autoStart = $state(false);
  let error = $state<string | null>(null);

  onMount(() => {
    void refresh();
  });

  async function refresh() {
    loading = true;
    error = null;
    try {
      const [settings, nextStatus] = await Promise.all([getSettings(), islevoipStatus()]);
      status = nextStatus;
      autoStart = settings.voice?.auto_start ?? false;
    } catch {
      error = $t("voice.status_error");
    } finally {
      loading = false;
    }
  }

  async function start() {
    starting = true;
    error = null;
    try {
      status = await islevoipLaunch();
    } catch {
      error = $t("voice.launch_error");
    } finally {
      starting = false;
    }
  }

  async function setAutoStart(checked: boolean) {
    const previous = autoStart;
    autoStart = checked;
    saving = true;
    error = null;
    try {
      const settings = await patchSettings({ voice: { auto_start: checked } });
      autoStart = settings.voice?.auto_start ?? checked;
    } catch {
      autoStart = previous;
      error = $t("voice.setting_error");
    } finally {
      saving = false;
    }
  }

  async function openOfficialSite() {
    error = null;
    try {
      await openUrl(ISLEVOIP_URL);
    } catch {
      error = $t("voice.link_error");
    }
  }
</script>

<div class="mx-auto max-w-3xl space-y-5 p-6">
  <section>
    <h2 class="text-lg font-semibold" style="color: var(--color-accent)">{$t("voice.title")}</h2>
    <p class="mt-1 text-sm" style="color: var(--color-muted)">{$t("voice.subtitle")}</p>
  </section>

  <section class="rounded border p-4" style="border-color: var(--color-border); background: var(--color-panel)">
    {#if loading}
      <p class="text-sm" style="color: var(--color-muted)">{$t("voice.checking")}</p>
    {:else if status?.installed}
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <strong class="text-sm">{$t("voice.installed")}</strong>
          <p class="mt-1 text-sm" style="color: var(--color-muted)">
            {status.running ? $t("voice.running") : $t("voice.not_running")}
          </p>
        </div>
        <button class="rounded border px-3 py-1.5 text-sm disabled:opacity-50" style="border-color: var(--color-accent); color: var(--color-accent)" disabled={starting || status.running} onclick={() => void start()}>
          {starting ? $t("voice.starting") : $t("voice.start")}
        </button>
      </div>
      <label class="mt-5 flex cursor-pointer items-center gap-2 text-sm">
        <input type="checkbox" checked={autoStart} disabled={saving} onchange={(event) => void setAutoStart(event.currentTarget.checked)} />
        {$t("voice.auto_start")}
      </label>
      <p class="mt-1 pl-6 text-xs" style="color: var(--color-muted)">{$t("voice.auto_start_hint")}</p>
    {:else}
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <strong class="text-sm">{$t("voice.not_installed")}</strong>
          <p class="mt-1 text-sm" style="color: var(--color-muted)">{$t("voice.install_hint")}</p>
        </div>
        <button class="rounded border px-3 py-1.5 text-sm" style="border-color: var(--color-accent); color: var(--color-accent)" onclick={() => void openOfficialSite()}>{$t("voice.install")}</button>
      </div>
    {/if}

    <div class="mt-5 flex flex-wrap gap-2 border-t pt-4" style="border-color: var(--color-border)">
      <button class="rounded border px-3 py-1.5 text-sm" style="border-color: var(--color-border)" disabled={loading} onclick={() => void refresh()}>{$t("voice.refresh")}</button>
      <button class="rounded border px-3 py-1.5 text-sm" style="border-color: var(--color-border)" onclick={() => void openOfficialSite()}>{$t("voice.help")}</button>
    </div>
    {#if error}<p class="mt-3 text-sm" role="alert" style="color: #ff8a80">{error}</p>{/if}
  </section>

  <section class="rounded border p-4 text-sm" style="border-color: var(--color-border); background: var(--color-panel); color: var(--color-muted)">
    <strong class="block" style="color: var(--color-text)">{$t("voice.pro_title")}</strong>
    <p class="mt-2">{$t("voice.pro_body")}</p>
  </section>
</div>
