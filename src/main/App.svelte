<script lang="ts">
  // Main window shell: tab navigation, the
  // exclusive-fullscreen warning banner, and locale bootstrapping.
  import { onMount } from "svelte";
  import {
    getDataStatus,
    getFullscreenMode,
    getSettings,
    listenerBag,
    onFetchFinished,
    onFullmapShow,
    onHotkeyFailed,
    onProviderState,
    onSettingsChanged,
    providerState,
    simulatePosition,
    trackFeature,
    type DataStatus,
    type FailedHotkey,
    type Feature,
    type ProviderState,
  } from "$lib/api";
  import { locale, t, type Locale } from "$lib/i18n";
  import { providerAllowsMain, providerLabel } from "$lib/provider-ui";
  import FullMap from "./fullmap/FullMap.svelte";
  import Footer from "./Footer.svelte";
  import DinoTab from "./dino/DinoTab.svelte";
  import GarageTab from "./garage/GarageTab.svelte";
  import ServerGarage from "./garage/ServerGarage.svelte";
  import SkinTab from "./skin/SkinTab.svelte";
  import CombatHistory from "./history/CombatHistory.svelte";
  import FriendsTab from "./friends/FriendsTab.svelte";
  import Settings from "./settings/Settings.svelte";
  import FirstRun from "./firstrun/FirstRun.svelte";
  import ConnectionGate from "./connection/ConnectionGate.svelte";
  import IslemapLogo from "./IslemapLogo.svelte";
  import UpdateBanner from "./UpdateBanner.svelte";
  import VoiceTab from "./voice/VoiceTab.svelte";

  type Tab = "map" | "dino" | "friends" | "voice" | "garage" | "skin" | "history" | "settings";
  const TAB_ITEMS: readonly [Tab, string][] = [
    ["map", "tab.map"],
    ["dino", "tab.dino"],
    ["friends", "tab.friends"],
    ["voice", "tab.voice"],
    ["garage", "tab.garage"],
    ["skin", "tab.skin"],
    ["history", "tab.history"],
    ["settings", "tab.settings"],
  ];
  const PROVIDER_TABS: readonly Tab[] = ["map", "dino", "garage", "skin", "history"];
  const requestedTab = location.hash.slice(1);
  const initialTab = TAB_ITEMS.some(([key]) => key === requestedTab)
    ? (requestedTab as Tab)
    : "map";

  // Lucide-style tab icons (24x24, stroke = currentColor) as inline path
  // markup — no icon library, and the color follows the button state.
  const TAB_ICONS: Record<Tab, string> = {
    map: '<path d="M14.106 5.553a2 2 0 0 0 1.788 0l3.659-1.83A1 1 0 0 1 21 4.619v12.764a1 1 0 0 1-.553.894l-4.553 2.277a2 2 0 0 1-1.788 0l-4.212-2.106a2 2 0 0 0-1.788 0l-3.659 1.83A1 1 0 0 1 3 19.381V6.618a1 1 0 0 1 .553-.894l4.553-2.277a2 2 0 0 1 1.788 0z"/><path d="M15 5.764v15"/><path d="M9 3.236v15"/>',
    dino: '<circle cx="11" cy="4" r="2"/><circle cx="18" cy="8" r="2"/><circle cx="20" cy="16" r="2"/><path d="M9 10a5 5 0 0 1 5 5v3.5a3.5 3.5 0 0 1-6.84 1.045Q6.52 17.48 4.46 16.84A3.5 3.5 0 0 1 5.5 10Z"/>',
    garage:
      '<path d="M22 8.35V20a2 2 0 0 1-2 2h-4v-9H8v9H4a2 2 0 0 1-2-2V8.35A2 2 0 0 1 3.26 6.5l8-3.2a2 2 0 0 1 1.48 0l8 3.2A2 2 0 0 1 22 8.35Z"/><path d="M6 18h12"/><path d="M6 14h12"/>',
    skin: '<path d="M12 22a10 10 0 1 0 0-20 7 7 0 0 0-7 7c0 1.8 1.2 3 3 3h1.2c1.1 0 1.8.9 1.4 1.9l-.5 1.3A5 5 0 0 0 12 22Z"/><circle cx="7.5" cy="7.5" r=".7" fill="currentColor"/><circle cx="12" cy="5.5" r=".7" fill="currentColor"/><circle cx="16.5" cy="8" r=".7" fill="currentColor"/>',
    history:
      '<path d="M3 3v5h5"/><path d="M3.6 15a9 9 0 1 0 .6-7.1L3 8"/><path d="M12 7v5l3 2"/><path d="m8 17 8-10"/>',
    friends:
      '<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>',
    voice:
      '<path d="M12 18.5a3.5 3.5 0 0 0 3.5-3.5V6a3.5 3.5 0 0 0-7 0v9a3.5 3.5 0 0 0 3.5 3.5Z"/><path d="M19 13v2a7 7 0 0 1-14 0v-2"/><path d="M12 22v-3.5"/>',
    settings:
      '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
  };
  let tab = $state<Tab>(initialTab);
  // Write-back so F5 restores the tab the user was on (the hash was already
  // read above; nothing ever wrote it). replaceState: no history spam.
  $effect(() => {
    history.replaceState(null, "", `#${tab}`);
  });
  // Which tabs people actually open. Everything else is counted in Rust, so
  // the hotkey and UI paths to the same action share one counter.
  // Deliberately a total Record, not Partial: adding a tab without deciding
  // how it is counted should be a compile error, not a silent zero.
  const TAB_FEATURE: Record<Tab, Feature | null> = {
    map: "fullmap_open",
    dino: "dino_tab_open",
    friends: null,
    voice: null,
    garage: "islepilot_garage",
    skin: null,
    history: null,
    settings: "settings_open",
  };
  // The first run of this effect is where the app OPENED — the default tab,
  // or whatever hash a reload restored — not somewhere the user went. It is
  // skipped: counting it inflated fullmap_open by one per launch, and
  // launches are already counted on the Rust side.
  let tabEffectPrimed = false;
  $effect(() => {
    const feature = TAB_FEATURE[tab];
    if (!tabEffectPrimed) {
      tabEffectPrimed = true;
      return;
    }
    if (feature) trackFeature(feature);
  });
  // Map, Dino and Garage tabs are KEPT ALIVE after their first visit (hidden
  // with display:none, not unmounted). Dino/Garage host a 3D viewer whose
  // teardown/rebuild made tab switching visibly laggy; the map is a Leaflet
  // instance over ~630 POI objects behind a 16-call IPC chain, and telemetry
  // shows people come back to it about twice a session. First visit still
  // lazy-mounts so an untouched tab costs nothing.
  let visitedMap = $state(false);
  let visitedDino = $state(false);
  let visitedGarage = $state(false);
  let visitedSkin = $state(false);
  let visitedFriends = $state(false);
  let visitedVoice = $state(false);
  $effect(() => {
    if (tab === "map") visitedMap = true;
    if (tab === "dino") visitedDino = true;
    if (tab === "garage") visitedGarage = true;
    if (tab === "skin") visitedSkin = true;
    if (tab === "friends") visitedFriends = true;
    if (tab === "voice") visitedVoice = true;
  });
  let dataStatus = $state<DataStatus | null>(null);
  let exclusiveFullscreen = $state(false);
  let failedHotkeys = $state<FailedHotkey[]>([]);
  let ready = $state(false);
  let connection = $state<ProviderState | null>(null);
  // Remount FullMap when the basemap changes ({#key} below): the imageOverlay
  // bounds and every layer's px change together, so a rebuild IS the correct
  // "in-place" update. Seeded before ready=true — no spurious first remount.
  let basemapSource = $state("vulnona");
  const activeTabLabel = $derived(TAB_ITEMS.find(([key]) => key === tab)?.[1] ?? "tab.map");

  // POIs are optional (fail-soft: the map works without dots); the basemap
  // images are the hard requirement.
  const dataOk = $derived(
    dataStatus !== null && dataStatus.basemapMinimap && dataStatus.basemapFullmap,
  );

  onMount(() => {
    const bag = listenerBag();
    (async () => {
      const settings = await getSettings();
      connection = await providerState();
      locale.set((settings.language as Locale) ?? "vi");
      basemapSource = settings.map?.basemap ?? "vulnona";
      dataStatus = await getDataStatus();
      exclusiveFullscreen = (await getFullscreenMode()) === 0;
      await bag.add(
        onSettingsChanged((s) => {
          locale.set((s.language as Locale) ?? "vi");
          basemapSource = s.map?.basemap ?? "vulnona";
        }),
      );
      await bag.add(onHotkeyFailed((failed) => (failedHotkeys = failed)));
      await bag.add(onProviderState((state) => (connection = state)));
      // Full-map hotkey mid-game: land on the map, not the last-open tab.
      await bag.add(onFullmapShow(() => (tab = "map")));
      // The download can finish while the user is on another tab (FirstRun
      // unmounted) — the App itself must notice and unlock the map tab.
      await bag.add(onFetchFinished(() => void getDataStatus().then((d) => (dataStatus = d))));
      ready = true;
    })();
    return () => bag.dispose();
  });

  // Dev-only: walk south-east to exercise the pipeline without the game.
  let simX = -231654;
  function simulateStep() {
    simX += 30_000;
    void simulatePosition(simX, 52099.673, 0);
  }

</script>

<UpdateBanner />

{#if !ready || !connection}
  <div class="flex h-screen items-center justify-center" style="color: var(--color-muted)">…</div>
{:else}
<div class="app-shell flex h-screen">
  <aside class="sidebar flex w-[218px] shrink-0 flex-col">
    <div class="brand-block"><IslemapLogo size={48} /></div>
    <div class="nav-label">NAVIGATION / 01</div>
    <nav class="grid gap-1.5 px-3">
      {#each TAB_ITEMS as [key, labelKey], index (key)}
        <button class:active={tab === key} class="nav-item" onclick={() => (tab = key as Tab)}>
          <span class="nav-index">0{index + 1}</span>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            {@html TAB_ICONS[key as Tab]}
          </svg>
          <span>{$t(labelKey as never)}</span>
          <i></i>
        </button>
      {/each}
    </nav>

    <div class="sidebar-spacer"></div>
    {#if import.meta.env.DEV}
      <button class="dev-step" onclick={simulateStep}>+300 M / DEV</button>
    {/if}
    <div class="signal-card">
      <div class="signal-heading"><span class="live-dot"></span> LIVE LINK</div>
      <strong>{providerLabel(connection.provider)}</strong>
      <small>{connection.website ?? "LOCAL PROVIDER"}</small>
      <div class="signal-bars"><span></span><span></span><span></span><span></span></div>
    </div>
    <div class="sidebar-version">SYS // {__APP_VERSION__}</div>
  </aside>

  <section class="workspace flex min-w-0 flex-1 flex-col">
    <header class="workspace-header">
      <div>
        <span class="eyebrow">islemap-thienvyma</span>
        <h1>{$t(activeTabLabel as never)}</h1>
      </div>
      <div class="telemetry">
        <div><small>NETWORK</small><strong>LINKED</strong></div>
        <div><small>PROVIDER</small><strong>{providerLabel(connection.provider)}</strong></div>
        <span class="radar-orbit"><i></i></span>
      </div>
    </header>

    {#if failedHotkeys.length > 0}
      <div class="alert-strip danger">
        <span>SYS-ERR</span> {$t("warn.hotkey_failed")}
        {failedHotkeys.map((f) => `${f.spec} (${$t(`hotkey.${f.action}` as never)})`).join(", ")}
        <button onclick={() => (failedHotkeys = [])}>{$t("btn.close")}</button>
      </div>
    {/if}

    {#if exclusiveFullscreen}
      <div class="alert-strip warning">
        <span>DISPLAY</span> {$t("warn.exclusive_fullscreen")}
        <button onclick={() => (exclusiveFullscreen = false)}>{$t("btn.close")}</button>
      </div>
    {/if}

    <main class="content-stage min-h-0 flex-1">
    {#if PROVIDER_TABS.includes(tab) && !providerAllowsMain(connection.status)}
      <div class="h-full overflow-y-auto"><ConnectionGate {connection} /></div>
    {:else if !ready}
      <div class="p-6" style="color: var(--color-muted)">…</div>
    {:else if tab === "map" && !dataOk}
      <!-- Only the map needs the downloaded data; the other tabs must stay
           usable during (and before) the first-run download. The map itself
           lives in the kept-alive block below. -->
      <FirstRun oncomplete={() => void getDataStatus().then((d) => (dataStatus = d))} />
    {:else if tab === "settings"}
      <div class="h-full overflow-y-auto"><Settings /></div>
    {:else if tab === "history"}
      <div class="h-full overflow-y-auto"><CombatHistory /></div>
    {/if}
    <!-- Kept-alive tabs (see visitedMap/visitedDino/visitedGarage above).
         All are error-isolated: a Leaflet throw, a failure in the IslePilot
         integration or the 3D viewer must never take down the shell (and
         its tab bar) or any other feature. -->
    {#if ready && providerAllowsMain(connection.status) && dataOk && visitedMap}
      <div class="h-full min-h-0" style:display={tab === "map" ? null : "none"}>
        {#key basemapSource}
          <svelte:boundary>
            <FullMap visible={tab === "map"} />
            {#snippet failed(_error, reset)}
              <div class="mx-auto max-w-lg p-8">
                <p class="mb-3 text-sm" style="color: #ff8a80">{$t("map.crashed")}</p>
                <button
                  class="cursor-pointer rounded border px-3 py-1 text-sm"
                  style="border-color: var(--color-border)"
                  onclick={reset}
                >
                  {$t("btn.retry")}
                </button>
              </div>
            {/snippet}
          </svelte:boundary>
        {/key}
      </div>
    {/if}
    {#if ready && providerAllowsMain(connection.status) && visitedDino}
      <div class="h-full overflow-y-auto" style:display={tab === "dino" ? null : "none"}>
        <svelte:boundary>
          <DinoTab />
          {#snippet failed(_error, reset)}
            <div class="mx-auto max-w-lg p-8">
              <p class="mb-3 text-sm" style="color: #ff8a80">{$t("dino.crashed")}</p>
              <button
                class="cursor-pointer rounded border px-3 py-1 text-sm"
                style="border-color: var(--color-border)"
                onclick={reset}
              >
                {$t("btn.retry")}
              </button>
            </div>
          {/snippet}
        </svelte:boundary>
      </div>
    {/if}
    {#if ready && providerAllowsMain(connection.status) && visitedGarage}
      <div class="h-full overflow-y-auto" style:display={tab === "garage" ? null : "none"}>
        <svelte:boundary>
          {#if connection.provider === "isle-pilot"}
            <GarageTab />
          {:else if connection.provider}
            <ServerGarage provider={connection.provider} />
          {/if}
          {#snippet failed(_error, reset)}
            <div class="mx-auto max-w-lg p-8">
              <p class="mb-3 text-sm" style="color: #ff8a80">{$t("dino.crashed")}</p>
              <button
                class="cursor-pointer rounded border px-3 py-1 text-sm"
                style="border-color: var(--color-border)"
                onclick={reset}
              >
                {$t("btn.retry")}
              </button>
            </div>
          {/snippet}
        </svelte:boundary>
      </div>
    {/if}
    {#if ready && providerAllowsMain(connection.status) && visitedSkin}
      <div class="h-full overflow-y-auto" style:display={tab === "skin" ? null : "none"}>
        <svelte:boundary>
          {#if connection.provider}<SkinTab provider={connection.provider} />{/if}
          {#snippet failed(_error, reset)}
            <div class="mx-auto max-w-lg p-8">
              <p class="mb-3 text-sm" style="color: #ff8a80">{$t("skin.failed")}</p>
              <button class="cursor-pointer rounded border px-3 py-1 text-sm" style="border-color: var(--color-border)" onclick={reset}>{$t("btn.retry")}</button>
            </div>
          {/snippet}
        </svelte:boundary>
      </div>
    {/if}
    {#if ready && visitedFriends}
      <div class="h-full overflow-y-auto" style:display={tab === "friends" ? null : "none"}>
        <svelte:boundary>
          <FriendsTab />
          {#snippet failed(_error, reset)}
            <div class="mx-auto max-w-lg p-8">
              <p class="mb-3 text-sm" style="color: #ff8a80">{$t("friends.error")}</p>
              <button class="cursor-pointer rounded border px-3 py-1 text-sm" style="border-color: var(--color-border)" onclick={reset}>{$t("btn.retry")}</button>
            </div>
          {/snippet}
        </svelte:boundary>
      </div>
    {/if}
    {#if ready && visitedVoice}
      <div class="h-full overflow-y-auto" style:display={tab === "voice" ? null : "none"}>
        <svelte:boundary>
          <VoiceTab />
          {#snippet failed(_error, reset)}
            <div class="mx-auto max-w-lg p-8">
              <p class="mb-3 text-sm" style="color: #ff8a80">{$t("voice.status_error")}</p>
              <button class="cursor-pointer rounded border px-3 py-1 text-sm" style="border-color: var(--color-border)" onclick={reset}>{$t("btn.retry")}</button>
            </div>
          {/snippet}
        </svelte:boundary>
      </div>
    {/if}
    </main>
    <Footer />
  </section>
</div>
{/if}

<style>
  .app-shell {
    background: transparent;
  }
  .sidebar {
    position: relative;
    z-index: 20;
    border-right: 1px solid var(--color-border);
    background: rgba(4, 10, 20, 0.96);
    box-shadow: 18px 0 50px rgba(0, 0, 0, 0.28);
  }
  .sidebar::after {
    content: "";
    position: absolute;
    inset: 0 0 0 auto;
    width: 1px;
    background: linear-gradient(transparent, var(--color-accent), transparent);
    opacity: 0.45;
    pointer-events: none;
  }
  .brand-block {
    padding: 22px 17px 21px;
    border-bottom: 1px solid rgba(53, 242, 255, 0.13);
  }
  .nav-label {
    padding: 22px 17px 10px;
    color: #40566f;
    font-family: Consolas, monospace;
    font-size: 9px;
    letter-spacing: 0.19em;
  }
  .nav-item {
    position: relative;
    display: grid;
    grid-template-columns: 21px 20px 1fr 3px;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    padding: 0 10px;
    overflow: hidden;
    border: 1px solid transparent;
    color: var(--color-muted);
    text-align: left;
    cursor: pointer;
    clip-path: polygon(0 0, calc(100% - 9px) 0, 100% 9px, 100% 100%, 0 100%);
  }
  .nav-item::before {
    content: "";
    position: absolute;
    inset: 0;
    z-index: -1;
    background: linear-gradient(90deg, rgba(53, 242, 255, 0.12), transparent 76%);
    opacity: 0;
  }
  .nav-item:hover,
  .nav-item.active {
    border-color: rgba(53, 242, 255, 0.3) !important;
    color: var(--color-text);
  }
  .nav-item.active::before { opacity: 1; }
  .nav-item.active svg { color: var(--color-accent); filter: drop-shadow(0 0 5px rgba(53,242,255,.45)); }
  .nav-item svg { width: 19px; height: 19px; }
  .nav-item i { width: 3px; height: 18px; background: transparent; }
  .nav-item.active i { background: var(--color-accent); box-shadow: 0 0 12px var(--color-accent); }
  .nav-index { color: #3e536a; font: 9px Consolas, monospace; }
  .sidebar-spacer { flex: 1; }
  .dev-step {
    margin: 0 13px 10px;
    border: 1px solid var(--color-border);
    padding: 7px;
    color: var(--color-muted);
    font: 10px Consolas, monospace;
  }
  .signal-card {
    position: relative;
    margin: 0 13px 14px;
    padding: 13px;
    overflow: hidden;
    border: 1px solid #16364d;
    background: linear-gradient(145deg, rgba(10, 29, 46, 0.9), rgba(7, 15, 28, 0.9));
    clip-path: polygon(0 0, calc(100% - 12px) 0, 100% 12px, 100% 100%, 0 100%);
  }
  .signal-card::after {
    content: "";
    position: absolute;
    right: -20px;
    bottom: -28px;
    width: 80px;
    height: 80px;
    border: 1px solid rgba(53, 242, 255, 0.13);
    border-radius: 50%;
  }
  .signal-heading { display: flex; align-items: center; gap: 7px; margin-bottom: 10px; color: var(--color-success); font: 9px Consolas, monospace; letter-spacing: .14em; }
  .live-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--color-success); box-shadow: 0 0 9px var(--color-success); }
  .signal-card strong { display: block; overflow: hidden; color: var(--color-text); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
  .signal-card small { display: block; margin-top: 3px; overflow: hidden; color: #59728b; font: 8px Consolas, monospace; text-overflow: ellipsis; white-space: nowrap; }
  .signal-bars { display: flex; align-items: end; gap: 2px; height: 11px; margin-top: 10px; }
  .signal-bars span { width: 3px; background: var(--color-accent); box-shadow: 0 0 5px rgba(53,242,255,.5); }
  .signal-bars span:nth-child(1) { height: 3px; } .signal-bars span:nth-child(2) { height: 5px; } .signal-bars span:nth-child(3) { height: 8px; } .signal-bars span:nth-child(4) { height: 11px; }
  .sidebar-version { padding: 10px 15px; border-top: 1px solid #101f31; color: #40566f; font: 8px Consolas, monospace; letter-spacing: .12em; }
  .workspace { background: rgba(2, 7, 15, 0.64); }
  .workspace-header {
    display: flex;
    min-height: 73px;
    flex: 0 0 auto;
    align-items: center;
    justify-content: space-between;
    padding: 0 22px;
    border-bottom: 1px solid var(--color-border);
    background: rgba(6, 15, 29, 0.88);
  }
  .eyebrow { color: var(--color-accent); font: 8px Consolas, monospace; letter-spacing: .22em; }
  .workspace-header h1 { margin: 3px 0 0; font-size: 20px; font-weight: 500; letter-spacing: .04em; }
  .telemetry { display: flex; align-items: center; gap: 24px; }
  .telemetry > div { display: grid; gap: 2px; border-left: 1px solid #1b3a54; padding-left: 10px; }
  .telemetry small { color: #4f6780; font: 8px Consolas, monospace; letter-spacing: .15em; }
  .telemetry strong { color: var(--color-success); font: 10px Consolas, monospace; font-weight: 600; }
  .radar-orbit { position: relative; width: 34px; height: 34px; border: 1px solid rgba(53,242,255,.28); border-radius: 50%; }
  .radar-orbit::before, .radar-orbit::after { content: ""; position: absolute; background: rgba(53,242,255,.2); }
  .radar-orbit::before { width: 1px; inset: 4px auto 4px 50%; } .radar-orbit::after { height: 1px; inset: 50% 4px auto; }
  .radar-orbit i { position: absolute; top: 6px; left: 19px; width: 4px; height: 4px; border-radius: 50%; background: var(--color-accent); box-shadow: 0 0 8px var(--color-accent); }
  .alert-strip { flex: 0 0 auto; padding: 8px 16px; border-bottom: 1px solid; font-size: 12px; }
  .alert-strip span { margin-right: 8px; font: 9px Consolas, monospace; letter-spacing: .15em; }
  .alert-strip button { float: right; cursor: pointer; text-decoration: underline; }
  .alert-strip.danger { border-color: #612039; background: #2a0d1b; color: #ff7892; }
  .alert-strip.warning { border-color: #614722; background: #241a0d; color: #ffd277; }
  .content-stage { position: relative; overflow: hidden; }
  .content-stage::before { content: ""; position: absolute; inset: 0; z-index: 50; pointer-events: none; box-shadow: inset 0 0 80px rgba(0,0,0,.18); }
  @media (max-width: 980px) {
    .sidebar { width: 76px; }
    .brand-block { padding-inline: 13px; }
    .brand-block :global(.wordmark), .nav-label, .nav-item span:not(.nav-index), .nav-index, .signal-card strong, .signal-card small, .signal-heading { display: none; }
    .nav-item { grid-template-columns: 1fr 3px; justify-items: center; padding: 0 8px; }
    .signal-card { height: 48px; padding: 10px; }
    .signal-bars { justify-content: center; margin-top: 7px; }
    .telemetry > div { display: none; }
  }
</style>
