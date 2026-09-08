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

<div class="flex h-full min-h-[520px] items-center justify-center p-6">
  <section
    class="w-full max-w-xl rounded-xl border p-6 shadow-2xl"
    style="border-color: var(--color-border); background: var(--color-panel)"
  >
    <div class="mb-5 flex items-center gap-3">
      <div
        class="flex size-11 items-center justify-center rounded-full text-xl"
        style="background: color-mix(in srgb, var(--color-accent) 18%, transparent); color: var(--color-accent)"
      >
        ◉
      </div>
      <div>
        <h1 class="text-xl font-semibold" style="color: var(--color-accent)">
          {$t("provider.title")}
        </h1>
        <p class="text-sm" style="color: var(--color-muted)">{$t("provider.subtitle")}</p>
      </div>
    </div>

    {#if connection.message}
      <p class="mb-3 rounded px-3 py-2 text-sm" style="background: #332819; color: #ffd591">
        {connection.message}
      </p>
    {/if}

    <label class="mb-1 block text-sm font-medium" for="provider-website">
      {$t("provider.website")}
    </label>
    <div class="flex gap-2">
      <input
        id="provider-website"
        class="min-w-0 flex-1 rounded border px-3 py-2 font-mono text-sm"
        style="border-color: var(--color-border); background: var(--color-bg); color: var(--color-text)"
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
      <button
        class="cursor-pointer rounded border px-4 py-2 text-sm font-medium disabled:opacity-50"
        style="border-color: var(--color-accent); color: var(--color-accent)"
        disabled={busy || !website.trim()}
        onclick={() => void detect()}
      >
        {$t("provider.detect")}
      </button>
    </div>
    <p class="mt-2 text-xs leading-relaxed" style="color: var(--color-muted)">
      {$t("provider.supported")}
    </p>

    {#if error}
      <p class="mt-3 rounded px-3 py-2 text-sm" style="background: #3a2222; color: #ff8a80">
        {error}<br />{$t("provider.adapter_required")}
      </p>
    {/if}

    {#if detected}
      <div
        class="mt-4 rounded border p-4"
        style="border-color: #315c43; background: #17291f"
      >
        <div class="flex items-center justify-between gap-3">
          <div>
            <div class="text-sm" style="color: #72d653">
              ✓ {$t("provider.detected")}: <strong>{providerLabel(detected.id)}</strong>
            </div>
            <div class="mt-1 font-mono text-xs" style="color: var(--color-muted)">
              {detected.origin}
            </div>
          </div>
          <button
            class="cursor-pointer rounded px-4 py-2 text-sm font-semibold disabled:opacity-50"
            style="background: var(--color-accent); color: var(--color-bg)"
            disabled={busy || waitingLogin}
            onclick={() => void login()}
          >
            {$t("provider.login")}
          </button>
        </div>
        {#if waitingLogin}
          <div class="mt-3 flex items-center gap-3 text-sm" style="color: #ffd591">
            <span>{$t("provider.login_wait")}</span>
            <button class="cursor-pointer underline" onclick={() => void cancel()}>
              {$t("btn.cancel")}
            </button>
          </div>
        {/if}
      </div>
    {/if}

    <div class="mt-5 border-t pt-4" style="border-color: var(--color-border)">
      <button
        class="cursor-pointer text-xs underline underline-offset-2"
        style="color: var(--color-muted)"
        onclick={() => void providerSelectManual()}
      >
        {$t("provider.manual")}
      </button>
    </div>
  </section>
</div>
