<script lang="ts">
  import { onMount } from "svelte";
  import {
    getSettings,
    listenerBag,
    onProviderSnapshot,
    onProviderState,
    patchSettings,
    providerLogout,
    providerSnapshot,
    providerState,
    type ProviderSnapshot,
    type ProviderState,
    type Settings,
    type SharedStatBar,
  } from "$lib/api";
  import { locale, t } from "$lib/i18n";
  import { formatStat, providerLabel } from "$lib/provider-ui";

  let settings = $state<Settings | null>(null);
  let connection = $state<ProviderState | null>(null);
  let snapshot = $state<ProviderSnapshot | null>(null);
  let changing = $state(false);

  const player = $derived(
    connection?.status === "temporary-error" && !connection.dataStale ? null : (snapshot?.player ?? null),
  );
  const online = $derived(connection?.status === "authenticated-online");
  const primeDone = $derived(
    player?.primeQuests.filter((quest) => quest.completed).length ?? 0,
  );

  onMount(() => {
    const bag = listenerBag();
    (async () => {
      [settings, connection, snapshot] = await Promise.all([
        getSettings(),
        providerState(),
        providerSnapshot(),
      ]);
      await bag.add(onProviderState((value) => (connection = value)));
      await bag.add(onProviderSnapshot((value) => (snapshot = value)));
    })();
    return () => bag.dispose();
  });

  async function patch(value: object) {
    settings = await patchSettings(value);
  }

  async function changeConnection() {
    changing = true;
    try {
      await providerLogout();
    } finally {
      changing = false;
    }
  }

  function timeStr(ms: number) {
    return new Date(ms).toLocaleTimeString($locale === "vi" ? "vi-VN" : "en-US");
  }

  function statColor(stat: SharedStatBar | null) {
    if (!stat) return "#4a4f43";
    return stat.percent > 50 ? "#45f5a2" : stat.percent > 25 ? "#ffc857" : "#ff5678";
  }
</script>

<div class="mx-auto max-w-3xl space-y-5 p-6">
  <section class="flex flex-wrap items-start justify-between gap-3">
    <div>
      <h2 class="text-lg font-semibold" style="color: var(--color-accent)">
        {$t("dino.title")}
      </h2>
      {#if connection}
        <p class="mt-1 text-sm" style="color: var(--color-muted)">
          {$t("provider.active")}: <strong style="color: var(--color-text)">
            {providerLabel(connection.provider)}
          </strong>
          {#if snapshot?.serverName} · {snapshot.serverName}{/if}
        </p>
      {/if}
    </div>
    <button
      class="cursor-pointer rounded border px-3 py-1.5 text-sm disabled:opacity-50"
      style="border-color: var(--color-border)"
      disabled={changing}
      onclick={() => void changeConnection()}
    >
      {$t("provider.change")}
    </button>
  </section>

  {#if connection?.status === "temporary-error"}
    <section class="rounded border p-3 text-sm" style="border-color: #65472a; color: #ffd277">
      {$t("provider.temporary")}{#if connection.message} · {connection.message}{/if}
      {#if connection.dataStale}<p class="mt-1">{$t("provider.stale")}</p>{/if}
    </section>
  {/if}

  {#if settings}
    <section
      class="flex flex-wrap gap-x-6 gap-y-2 rounded border p-3"
      style="border-color: var(--color-border); background: var(--color-panel)"
    >
      <label class="flex cursor-pointer items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={settings.islepilot.show_overlay_panel}
          onchange={(event) =>
            void patch({ islepilot: { show_overlay_panel: event.currentTarget.checked } })}
        />
        {$t("dino.overlay_panel")}
      </label>
      <label class="flex cursor-pointer items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={settings.islepilot.show_quests_panel}
          onchange={(event) =>
            void patch({ islepilot: { show_quests_panel: event.currentTarget.checked } })}
        />
        {$t("dino.quests_panel")}
      </label>
    </section>
  {/if}

  <section
    class="rounded border p-4"
    style="border-color: var(--color-border); background: var(--color-panel)"
  >
    <div class="mb-4 flex flex-wrap items-center gap-2">
      <span class="text-lg font-semibold">{player?.dinoName ?? "—"}</span>
      <span
        class="rounded-full px-2 py-0.5 text-xs font-medium"
        style={online
          ? "background: #0d3028; color: #45f5a2"
          : "background: #352814; color: #ffd277"}
      >
        {connection?.status === "temporary-error" ? $t("provider.retrying") : online ? $t("dino.online") : $t("dino.offline")}
      </span>
      {#if player?.female !== null && player?.female !== undefined}
        <span class="text-xs" style="color: var(--color-muted)">
          {player.female ? `♀ ${$t("dino.sex_female")}` : `♂ ${$t("dino.sex_male")}`}
        </span>
      {/if}
      {#if snapshot}
        <span class="ml-auto text-xs" style="color: var(--color-muted)">
          {$t("dino.updated", { time: timeStr(snapshot.sourceTimestampMs ?? snapshot.receivedAtMs) })}
        </span>
      {/if}
    </div>

    {#if player}
      <div class="grid gap-3 sm:grid-cols-2">
        {#each [["dino.health", player.health], ["dino.stamina", player.stamina], ["dino.hunger", player.hunger], ["dino.thirst", player.thirst]] as [key, value]}
          <div>
            <div class="mb-1 flex justify-between text-sm">
              <span>{$t(key as never)}</span>
              <span class="font-mono">{formatStat(value as SharedStatBar | null)}</span>
            </div>
            <div class="h-2 overflow-hidden rounded" style="background: #101d2c">
              <div
                class="h-full rounded"
                style={`width: ${Math.max(0, Math.min(100, (value as SharedStatBar | null)?.percent ?? 0))}%; background: ${statColor(value as SharedStatBar | null)}`}
              ></div>
            </div>
          </div>
        {/each}
      </div>
      <div class="mt-4 text-sm">
        {$t("dino.growth")}: <strong>{player.growthPct == null ? "—" : `${Math.round(player.growthPct)}%`}</strong>
      </div>

      {#if player.mutations.length > 0}
        <div class="mt-4 border-t pt-3" style="border-color: var(--color-border)">
          <div class="mb-1 text-sm font-semibold" style="color: var(--color-accent)">
            {$t("provider.mutations")}
          </div>
          <p class="text-sm">{player.mutations.join(", ")}</p>
        </div>
      {/if}

      {#if player.primeQuests.length > 0}
        <div class="mt-4 border-t pt-3" style="border-color: var(--color-border)">
          <div class="mb-3 flex items-center justify-between gap-3">
            <div class="text-sm font-semibold" style="color: var(--color-accent)">
              {$t("dino.prime")}
            </div>
            <span
              class="rounded border px-2 py-0.5 font-mono text-xs"
              style="border-color: rgba(53, 242, 255, 0.35); color: var(--color-accent); background: rgba(53, 242, 255, 0.07)"
            >
              {primeDone}/{player.primeQuests.length}
            </span>
          </div>
          <div class="grid gap-2">
            {#each player.primeQuests as quest, index}
              <div
                class="grid grid-cols-[28px_1fr_auto] items-center gap-3 rounded border px-3 py-2"
                style={`border-color: ${quest.completed ? 'rgba(69, 245, 162, 0.32)' : 'var(--color-border)'}; background: ${quest.completed ? 'rgba(69, 245, 162, 0.055)' : 'rgba(7, 16, 29, 0.62)'}`}
              >
                <span
                  class="flex h-7 w-7 items-center justify-center rounded font-mono text-xs font-semibold"
                  style={`background: ${quest.completed ? '#45f5a2' : '#112337'}; color: ${quest.completed ? '#03110c' : '#7890aa'}`}
                >
                  {quest.completed ? "✓" : String(index + 1).padStart(2, "0")}
                </span>
                <span class="text-sm leading-snug" style="color: {quest.completed ? '#d9fff0' : 'var(--color-text)'}">
                  {$locale === "vi" ? (quest.textVi ?? quest.text) : quest.text}
                </span>
                <span
                  class="hidden whitespace-nowrap font-mono text-[9px] uppercase tracking-wider sm:block"
                  style="color: {quest.completed ? '#45f5a2' : '#617890'}"
                >
                  {quest.completed ? $t("dino.prime_done") : $t("dino.prime_pending")}
                </span>
              </div>
            {/each}
          </div>
        </div>
      {:else if connection?.provider === "era"}
        <div class="mt-4 border-t pt-3" style="border-color: var(--color-border)">
          <div class="text-sm font-semibold" style="color: var(--color-accent)">{$t("dino.prime")}</div>
          <p class="mt-2 text-sm" style="color: var(--color-muted)">{$t("dino.era_prime_unavailable")}</p>
        </div>
      {/if}
      {#if connection?.provider === "era"}
        <p class="mt-4 text-xs" style="color: var(--color-muted)">{$t("dino.era_heading_cadence")}</p>
      {/if}

      {#if !player.health && !player.stamina && !player.hunger && !player.thirst}
        <p class="mt-4 text-xs" style="color: var(--color-muted)">
          {$t("provider.capability_missing")}
        </p>
      {/if}
    {:else}
      <p class="text-sm" style="color: var(--color-muted)">
        {connection?.status === "temporary-error"
          ? $t("provider.temporary")
          : $t("provider.offline_hint")}
      </p>
    {/if}
  </section>
</div>
