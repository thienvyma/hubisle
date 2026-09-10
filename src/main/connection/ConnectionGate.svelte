<script lang="ts">
  import {
    providerCancelLogin,
    providerDetect,
    providerSelectManual,
    providerStartLogin,
    type DetectedProvider,
    type ProviderState,
  } from "$lib/api";
  import { t } from "$lib/i18n";
  import { providerLabel } from "$lib/provider-ui";
  import PulseLogo from "../PulseLogo.svelte";

  let { connection }: { connection: ProviderState } = $props();
  let website = $state("");
  let detected = $state<DetectedProvider | null>(null);
  let busy = $state(false);
  let waitingLogin = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (!website) website = connection.website ?? "https://eragamingvn.net";
  });

  async function detect() {
    busy = true;
    waitingLogin = false;
    error = null;
    try {
      detected = await providerDetect(website.trim());
      website = detected.origin;
    } catch (reason) {
      detected = null;
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function login() {
    if (!detected) return;
    busy = true;
    error = null;
    try {
      await providerStartLogin(detected.origin);
      waitingLogin = true;
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function cancel() {
    await providerCancelLogin();
    waitingLogin = false;
  }
</script>

<div class="link-station flex h-full min-h-[520px] items-center justify-center p-6">
  <section class="connection-console w-full max-w-2xl">
    <div class="console-head">
      <PulseLogo size={62} />
      <div class="scanner"><i></i><span></span></div>
    </div>
    <div class="console-title">
      <span>CONNECTION PROTOCOL / 01</span>
      <h1>{$t("provider.title")}</h1>
      <p>{$t("provider.subtitle")}</p>
    </div>

    {#if connection.message}
      <p class="system-message"><b>STATUS</b>{connection.message}</p>
    {/if}

    <div class="input-module">
      <label for="provider-website"><span>01</span>{$t("provider.website")}</label>
      <div class="flex gap-2">
        <input
          id="provider-website"
          class="min-w-0 flex-1"
          bind:value={website}
          placeholder="https://eragamingvn.net"
          oninput={() => {
            detected = null;
            error = null;
          }}
          onkeydown={(event) => {
            if (event.key === "Enter") void detect();
          }}
        />
        <button class="scan-button" disabled={busy || !website.trim()} onclick={() => void detect()}>
          {busy ? "SCANNING…" : $t("provider.detect")}
        </button>
      </div>
      <p>{$t("provider.supported")}</p>
    </div>

    {#if error}
      <p class="error-module"><b>LINK ERROR</b>{error}<br />{$t("provider.adapter_required")}</p>
    {/if}

    {#if detected}
      <div class="detected-module">
        <div class="flex items-center justify-between gap-3">
          <div>
            <div class="detected-label"><i></i>{$t("provider.detected")}</div>
            <strong>{providerLabel(detected.id)}</strong>
            <small>{detected.origin}</small>
          </div>
          <button class="login-button" disabled={busy || waitingLogin} onclick={() => void login()}>
            {$t("provider.login")}
          </button>
        </div>
        {#if waitingLogin}
          <div class="login-wait flex items-center gap-3">
            <span>{$t("provider.login_wait")}</span>
            <button class="cursor-pointer underline" onclick={() => void cancel()}>
              {$t("btn.cancel")}
            </button>
          </div>
        {/if}
      </div>
    {/if}

    <div class="console-foot">
      <span>SECURE STEAM OPENID CHANNEL</span>
      <button onclick={() => void providerSelectManual()}>
        {$t("provider.manual")}
      </button>
    </div>
  </section>
</div>

<style>
  .link-station {
    position: relative;
    overflow: hidden;
    background:
      radial-gradient(circle at 50% 48%, rgba(27, 102, 148, .19), transparent 32%),
      linear-gradient(rgba(53,242,255,.025) 1px, transparent 1px),
      linear-gradient(90deg, rgba(53,242,255,.025) 1px, transparent 1px);
    background-size: auto, 38px 38px, 38px 38px;
  }
  .link-station::before, .link-station::after {
    content: "";
    position: absolute;
    width: 420px;
    height: 420px;
    border: 1px solid rgba(53,242,255,.07);
    border-radius: 50%;
    pointer-events: none;
  }
  .link-station::before { left: -230px; top: -190px; }
  .link-station::after { right: -250px; bottom: -220px; }
  .connection-console {
    position: relative;
    z-index: 1;
    border: 1px solid #1b4964;
    padding: 27px 30px 22px;
    background: linear-gradient(145deg, rgba(8,20,36,.97), rgba(3,9,19,.97));
    box-shadow: 0 30px 90px rgba(0,0,0,.45), inset 0 0 60px rgba(53,242,255,.025);
    clip-path: polygon(0 0, calc(100% - 26px) 0, 100% 26px, 100% 100%, 26px 100%, 0 calc(100% - 26px));
  }
  .connection-console::before {
    content: "";
    position: absolute;
    left: 0;
    top: 0;
    width: 110px;
    height: 2px;
    background: var(--color-accent);
    box-shadow: 0 0 16px var(--color-accent);
  }
  .console-head { display: flex; align-items: center; justify-content: space-between; }
  .scanner { position: relative; width: 72px; height: 72px; border: 1px solid #17364e; border-radius: 50%; }
  .scanner::before, .scanner::after { content: ""; position: absolute; inset: 10px; border: 1px solid rgba(53,242,255,.14); border-radius: 50%; }
  .scanner::after { inset: 25px; }
  .scanner i { position: absolute; left: 50%; top: 50%; width: 30px; height: 1px; transform-origin: left; transform: rotate(-35deg); background: linear-gradient(90deg, var(--color-accent), transparent); }
  .scanner span { position: absolute; right: 15px; top: 18px; width: 4px; height: 4px; border-radius: 50%; background: var(--color-success); box-shadow: 0 0 8px var(--color-success); }
  .console-title { margin: 22px 0 24px; }
  .console-title > span { color: var(--color-accent); font: 9px Consolas, monospace; letter-spacing: .22em; }
  .console-title h1 { margin: 5px 0 4px; color: var(--color-text); font-size: 25px; font-weight: 500; letter-spacing: .035em; }
  .console-title p { margin: 0; color: var(--color-muted); font-size: 13px; }
  .system-message, .error-module { display: grid; grid-template-columns: 76px 1fr; gap: 10px; margin: 0 0 15px; border-left: 2px solid var(--color-warning); padding: 10px 12px; background: rgba(255,200,87,.07); color: #d7bd79; font-size: 12px; }
  .system-message b, .error-module b { color: var(--color-warning); font: 9px Consolas, monospace; letter-spacing: .13em; }
  .input-module { border: 1px solid #153650; padding: 16px; background: rgba(3,10,20,.62); }
  .input-module label { display: flex; align-items: center; gap: 9px; margin-bottom: 8px; color: #b9cadd; font-size: 12px; }
  .input-module label span { color: var(--color-accent); font: 9px Consolas, monospace; }
  .input-module input { min-height: 42px; border: 1px solid #1d405a; padding: 0 13px; background: #030914; color: var(--color-text); font: 12px Consolas, monospace; }
  .input-module p { margin: 8px 0 0; color: #58718b; font-size: 10px; line-height: 1.5; }
  .scan-button, .login-button { min-width: 124px; border: 1px solid var(--color-accent); padding: 0 17px; color: var(--color-accent); font-size: 11px; font-weight: 650; letter-spacing: .05em; cursor: pointer; clip-path: polygon(0 0, calc(100% - 9px) 0, 100% 9px, 100% 100%, 0 100%); }
  .scan-button:disabled, .login-button:disabled { cursor: default; opacity: .48; }
  .error-module { margin-top: 13px; border-color: var(--color-danger); background: rgba(255,86,120,.07); color: #ff9daf; }
  .error-module b { color: var(--color-danger); }
  .detected-module { margin-top: 14px; border: 1px solid rgba(69,245,162,.35); padding: 14px 16px; background: rgba(69,245,162,.055); }
  .detected-label { display: flex; align-items: center; gap: 8px; color: var(--color-success); font: 9px Consolas, monospace; letter-spacing: .12em; }
  .detected-label i { width: 6px; height: 6px; border-radius: 50%; background: var(--color-success); box-shadow: 0 0 8px var(--color-success); }
  .detected-module strong { display: block; margin-top: 5px; color: var(--color-text); font-size: 14px; }
  .detected-module small { display: block; margin-top: 2px; color: #607c94; font: 10px Consolas, monospace; }
  .login-button { min-height: 40px; background: var(--color-accent); color: #031019; box-shadow: 0 0 24px rgba(53,242,255,.14); }
  .login-wait { margin-top: 12px; border-top: 1px solid rgba(69,245,162,.18); padding-top: 10px; color: var(--color-warning); font-size: 11px; }
  .console-foot { display: flex; justify-content: space-between; margin-top: 18px; border-top: 1px solid #132a3e; padding-top: 13px; }
  .console-foot span { color: #3d566d; font: 8px Consolas, monospace; letter-spacing: .13em; }
  .console-foot button { color: #647f97; font-size: 10px; text-decoration: underline; cursor: pointer; }
</style>
