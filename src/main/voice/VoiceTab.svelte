<script lang="ts">
  import { onMount } from "svelte";
  import {
    listenerBag,
    onProviderState,
    onVoiceState,
    voiceLogout,
    voiceStartLogin,
    voiceStatus,
    type VoiceState,
    type VoiceStatus,
  } from "$lib/api";
  import { t } from "$lib/i18n";

  let status = $state<VoiceStatus | null>(null);
  let loading = $state(true);
  let busy = $state(false);
  let error = $state<string | null>(null);

  function stateLabel(state: VoiceState): string {
    switch (state) {
      case "not-configured": return $t("voice.state_not_configured");
      case "login-required": return $t("voice.state_login_required");
      case "authorizing": return $t("voice.state_authorizing");
      case "ready": return $t("voice.state_ready");
      case "unavailable": return $t("voice.state_unavailable");
      case "blocked": return $t("voice.state_blocked");
      case "error": return $t("voice.state_error");
    }
  }

  function stateHint(state: VoiceState): string {
    switch (state) {
      case "not-configured": return $t("voice.not_configured_hint");
      case "login-required": return $t("voice.login_hint");
      case "authorizing": return $t("voice.authorizing_hint");
      case "ready": return $t("voice.ready_hint");
      case "unavailable": return $t("voice.unavailable_hint");
      case "blocked": return $t("voice.blocked_hint");
      case "error": return $t("voice.error_hint");
    }
  }

  async function refresh() {
    loading = true;
    error = null;
    try {
      status = await voiceStatus();
    } catch {
      error = $t("voice.status_error");
    } finally {
      loading = false;
    }
  }

  async function signIn() {
    busy = true;
    error = null;
    try {
      status = await voiceStartLogin();
    } catch {
      error = $t("voice.action_error");
    } finally {
      busy = false;
    }
  }

  async function logout() {
    busy = true;
    error = null;
    try {
      status = await voiceLogout();
    } catch {
      error = $t("voice.action_error");
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    const listeners = listenerBag();
    void listeners.add(onVoiceState((next) => {
      status = next;
      loading = false;
      error = null;
    }));
    void listeners.add(onProviderState(() => void refresh()));
    void refresh();
    return () => listeners.dispose();
  });
</script>

<div class="mx-auto max-w-3xl space-y-5 p-6">
  <section>
    <div class="eyebrow">SERVER // VOICE PROVIDER</div>
    <h2 class="mt-1 text-xl font-semibold" style="color: var(--color-accent)">{$t("voice.title")}</h2>
    <p class="mt-1 text-sm leading-6" style="color: var(--color-muted)">{$t("voice.subtitle")}</p>
  </section>

  <section class="panel p-5">
    {#if loading}
      <p class="text-sm" style="color: var(--color-muted)">{$t("voice.checking")}</p>
    {:else if status}
      <div class="status-grid">
        <div class="status-card">
          <small>{$t("voice.server")}</small>
          <strong>{status.serverName ?? status.serverOrigin ?? $t("voice.not_connected")}</strong>
        </div>
        <div class="status-card">
          <small>{$t("voice.provider")}</small>
          <strong>{status.provider?.displayName ?? $t("voice.provider_none")}</strong>
        </div>
        <div class:ok={status.state === "ready"} class:pending={status.state === "authorizing"} class="status-card">
          <small>{$t("voice.status")}</small>
          <strong>{stateLabel(status.state)}</strong>
        </div>
      </div>

      <p class="hint mt-5">{stateHint(status.state)}</p>

      {#if status.session}
        <div class="identity mt-4">
          <small>{$t("voice.identity")}</small>
          <strong>{status.session.playerName ?? status.session.steamId ?? $t("voice.identity_unknown")}</strong>
          {#if status.session.steamId && status.session.playerName}
            <span>{$t("voice.steam_id")}: {status.session.steamId}</span>
          {/if}
        </div>
      {/if}

      {#if status.state === "not-configured"}
        <p class="notice mt-5">{$t("voice.server_owner_hint")}</p>
      {/if}

      <div class="mt-5 flex flex-wrap gap-2 border-t pt-4" style="border-color: var(--color-border)">
        {#if status.state === "login-required"}
          <button class="accent-button" disabled={busy} onclick={() => void signIn()}>{$t("voice.sign_in")}</button>
        {:else if status.state === "ready"}
          <button class="control-button" disabled={busy} onclick={() => void logout()}>{$t("voice.logout")}</button>
        {/if}
        <button class="control-button" disabled={loading || busy} onclick={() => void refresh()}>{$t("voice.refresh")}</button>
      </div>
    {/if}

    {#if error}<p class="mt-4 text-sm" role="alert" style="color: #ff8a80">{error}</p>{/if}
  </section>
</div>

<style>
  .panel { border: 1px solid var(--color-border); background: var(--color-panel); }
  .eyebrow { color: var(--color-accent); font-size: 10px; letter-spacing: .24em; font-weight: 700; }
  .status-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }
  .status-card { min-height: 72px; border: 1px solid var(--color-border); padding: 12px; background: rgba(255,255,255,.018); }
  .status-card.ok { border-color: rgba(77,224,143,.45); background: rgba(77,224,143,.05); }
  .status-card.pending { border-color: rgba(231,174,69,.45); background: rgba(231,174,69,.05); }
  .status-card small, .identity small { display: block; margin-bottom: 5px; color: var(--color-muted); font-size: 9px; letter-spacing: .14em; }
  .status-card strong, .identity strong { display: block; overflow-wrap: anywhere; color: var(--color-text); font-size: 12px; }
  .hint { color: var(--color-muted); font-size: 13px; line-height: 1.65; }
  .identity { border: 1px solid var(--color-border); padding: 12px; background: rgba(53,242,255,.025); }
  .identity span { display: block; margin-top: 4px; color: var(--color-muted); font: 10px Consolas, monospace; }
  .notice { border-left: 2px solid #e7ae45; padding: 8px 11px; color: #e9c77d; font-size: 12px; line-height: 1.55; }
  button { cursor: pointer; }
  button:disabled { cursor: not-allowed; opacity: .45; }
  .accent-button, .control-button { border: 1px solid var(--color-border); padding: 8px 14px; font-size: 13px; }
  .accent-button { border-color: var(--color-accent); color: var(--color-accent); background: rgba(53,242,255,.08); }
  .control-button { color: var(--color-text); background: rgba(255,255,255,.025); }
  @media (max-width: 760px) { .status-grid { grid-template-columns: 1fr; } }
</style>
