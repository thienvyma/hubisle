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
    connection?.status === "temporary-error" ? null : (snapshot?.player ?? null),
  );
  const online = $derived(connection?.status === "authenticated-online");

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
    return stat.percent > 50 ? "#72d653" : stat.percent > 25 ? "#e8a33d" : "#e2664a";
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
    <section class="rounded border p-3 text-sm" style="border-color: #7a5d2b; color: #ffd591">
      {$t("provider.temporary")}{#if connection.message} · {connection.message}{/if}
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
          ? "background: #1e3a2f; color: #72d653"
          : "background: #3a3022; color: #ffd591"}
      >
        {online ? $t("dino.online") : $t("dino.offline")}
      </span>
      {#if player?.female !== null && player?.female !== undefined}
        <span class="text-xs" style="color: var(--color-muted)">
          {player.female ? `♀ ${$t("dino.sex_female")}` : `♂ ${$t("dino.sex_male")}`}
        </span>
      {/if}
      {#if snapshot}
        <span class="ml-auto text-xs" style="color: var(--color-muted)">
          {$t("dino.updated", { time: timeStr(snapshot.receivedAtMs) })}
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
            <div class="h-2 overflow-hidden rounded" style="background: #252a22">
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
          <div class="mb-2 text-sm font-semibold" style="color: var(--color-accent)">
            {$t("dino.prime")}
          </div>
          <div class="grid gap-1 sm:grid-cols-2">
            {#each player.primeQuests as quest}
              <div class="text-xs" style="color: {quest.completed ? '#72d653' : 'var(--color-muted)'}">
                {quest.completed ? "✓" : "○"} {$locale === "vi" ? (quest.textVi ?? quest.text) : quest.text}
              </div>
            {/each}
          </div>
        </div>
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
