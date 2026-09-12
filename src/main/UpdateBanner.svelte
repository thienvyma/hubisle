<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { t } from "$lib/i18n";
  import { RELEASES_URL, updates, updateState } from "$lib/updates";

  const busy = $derived(["downloading", "installing"].includes($updateState.status));
  const available = $derived(["available", "downloading", "installing", "install-error"].includes($updateState.status));
  let linkFailed = $state(false);

  onMount(() => {
    void updates.checkNow();
    const timer = window.setInterval(() => void updates.checkNow(), 5 * 60 * 1000);
    const online = () => void updates.checkNow();
    window.addEventListener("online", online);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("online", online);
      void updates.dispose();
    };
  });

  async function openReleases() {
    linkFailed = false;
    try { await openUrl(RELEASES_URL); } catch { linkFailed = true; }
  }
</script>

{#if $updateState.visible}
  <section class="update-card" aria-label={$t("update.title")}>
    <div class="copy" role="status" aria-live="polite">
      <small>ISLE PULSE · v{__APP_VERSION__}</small>
      <strong>
        {#if $updateState.status === "checking"}{$t("update.checking")}
        {:else if $updateState.status === "check-error"}{$t("update.check_failed")}
        {:else if $updateState.status === "current"}{$t("update.current")}
        {:else}{$t("update.available", { version: $updateState.version ?? "" })}{/if}
      </strong>
      <span>
        {#if $updateState.status === "check-error"}{$t("update.check_failed_body")}
        {:else if $updateState.status === "install-error"}{$t("update.failed")}
        {:else}{$t("update.body")}{/if}
      </span>
    </div>
    {#if available && $updateState.notes}
      <details><summary>{$t("update.notes")}</summary><p>{$updateState.notes}</p></details>
    {/if}
    <div class="actions">
      {#if available}
        <button class="install" disabled={busy} onclick={() => void updates.install()}>
          {#if $updateState.status === "downloading"}
            {$updateState.progress === null ? $t("update.downloading_unknown") : $t("update.downloading", { percent: $updateState.progress })}
          {:else if $updateState.status === "installing"}{$t("update.installing")}
          {:else}{$t("update.install")}{/if}
        </button>
      {:else if $updateState.status === "check-error"}
        <button class="install" onclick={() => void updates.checkNow(true)}>{$t("btn.retry")}</button>
      {/if}
      <button disabled={busy} onclick={() => void openReleases()}>{$t("update.releases")}</button>
      <button disabled={busy} onclick={() => updates.dismiss()}>{available ? $t("update.later") : $t("btn.close")}</button>
    </div>
    {#if linkFailed}<p role="alert">{$t("update.link_failed")}</p>{/if}
    {#if busy}
      <progress max="100" value={$updateState.progress ?? undefined} aria-label={$t("update.install")}></progress>
    {/if}
  </section>
{/if}

<style>
  .update-card {
    position: fixed; z-index: 10000; top: 12px; right: 14px;
    width: min(540px, calc(100vw - 28px)); max-height: calc(100vh - 24px);
    overflow-y: auto; padding: 16px; border: 1px solid rgba(53, 242, 255, .52);
    background: rgba(4, 14, 27, .98); color: var(--color-text);
    box-shadow: 0 12px 40px rgba(0, 0, 0, .5);
  }
  .copy { display: grid; gap: 6px; }
  small { color: var(--color-accent); font: 10px Consolas, monospace; }
  strong { color: #d8f9ff; font-size: 14px; }
  .copy span, p { color: #9bb3c8; font-size: 12px; }
  details { margin-top: 12px; font-size: 12px; }
  summary { cursor: pointer; color: var(--color-accent); }
  details p { margin-top: 8px; max-height: 180px; overflow: auto; white-space: pre-wrap; overflow-wrap: anywhere; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }
  button { border: 1px solid #27425a; padding: 8px 10px; color: #b2c8db; font-size: 12px; cursor: pointer; }
  button:disabled { cursor: wait; opacity: .65; }
  button:focus-visible, summary:focus-visible { outline: 2px solid var(--color-accent); outline-offset: 3px; }
  .install { border-color: var(--color-accent); background: rgba(53, 242, 255, .12); color: var(--color-accent); }
  progress { display: block; width: 100%; height: 5px; margin-top: 12px; accent-color: var(--color-accent); }
</style>
