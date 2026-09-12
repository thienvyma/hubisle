<script lang="ts">
  import { onMount } from "svelte";
  import {
    listenerBag,
    onProviderSnapshot,
    onProviderState,
    providerSnapshot,
    providerState,
    type ProviderSnapshot,
    type ProviderState,
  } from "$lib/api";
  import { locale, t } from "$lib/i18n";
  import { providerLabel } from "$lib/provider-ui";

  let connection = $state<ProviderState | null>(null);
  let snapshot = $state<ProviderSnapshot | null>(null);
  let loading = $state(true);
  let failed = $state(false);

  const friends = $derived(snapshot?.friends ?? []);
  const positioned = $derived(friends.filter((friend) => friend.positionCm !== null));

  onMount(() => {
    const bag = listenerBag();
    void (async () => {
      try {
        connection = await providerState();
        await bag.add(onProviderState((value) => (connection = value)));
        await bag.add(onProviderSnapshot((value) => {
          snapshot = value;
          failed = false;
          loading = false;
        }));
        snapshot = await providerSnapshot();
      } catch {
        failed = true;
      } finally {
        loading = false;
      }
    })();
    return () => bag.dispose();
  });

  function timeStr(ms: number) {
    return new Date(ms).toLocaleTimeString($locale === "vi" ? "vi-VN" : "en-US");
  }
</script>

<div class="mx-auto max-w-3xl space-y-5 p-6">
  <section>
    <h2 class="text-lg font-semibold" style="color: var(--color-accent)">{$t("friends.title")}</h2>
    <p class="mt-1 text-sm" style="color: var(--color-muted)">{$t("friends.subtitle")}</p>
  </section>

  {#if !connection?.provider}
    <section class="rounded border p-4 text-sm" style="border-color: var(--color-border); background: var(--color-panel); color: var(--color-muted)">
      {$t("friends.connect")}
    </section>
  {:else if loading}
    <section class="rounded border p-4 text-sm" style="border-color: var(--color-border); background: var(--color-panel); color: var(--color-muted)">
      {$t("friends.loading")}
    </section>
  {:else if failed}
    <section class="rounded border p-4 text-sm" style="border-color: #65472a; background: var(--color-panel); color: #ffd277">
      {$t("friends.error")}
    </section>
  {:else if !snapshot}
    <section class="rounded border p-4 text-sm" style="border-color: var(--color-border); background: var(--color-panel); color: var(--color-muted)">
      {$t("friends.waiting")}
    </section>
  {:else}
    <section class="rounded border p-4" style="border-color: var(--color-border); background: var(--color-panel)">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div>
          <strong class="text-sm" style="color: var(--color-text)">{providerLabel(snapshot.provider)}</strong>
          {#if snapshot.serverName}<span class="ml-2 text-sm" style="color: var(--color-muted)">· {snapshot.serverName}</span>{/if}
        </div>
        <span class="font-mono text-xs" style="color: var(--color-muted)">
          {$t("friends.positioned", { count: positioned.length, total: friends.length })}
        </span>
      </div>

      {#if friends.length === 0}
        <p class="mt-4 text-sm" style="color: var(--color-muted)">{$t("friends.provider_empty")}</p>
      {:else}
        <div class="mt-4 grid gap-2">
          {#each friends as friend (friend.slot ?? friend.name)}
            <article class="flex flex-wrap items-center gap-x-3 gap-y-1 rounded border px-3 py-2" style="border-color: var(--color-border); background: rgba(7, 16, 29, 0.62)">
              <div class="min-w-0 flex-1">
                <strong class="block truncate text-sm">{friend.name}</strong>
                <span class="text-xs" style="color: var(--color-muted)">{friend.dinoName ?? $t("friends.dino_unknown")}</span>
              </div>
              <span class="rounded-full px-2 py-0.5 text-xs" style={friend.online ? "background: #0d3028; color: #45f5a2" : "background: #352814; color: #ffd277"}>
                {friend.online ? $t("friends.online") : $t("friends.offline")}
              </span>
              <span class="text-xs" style="color: var(--color-muted)">
                {friend.positionCm ? $t("friends.position_available") : $t("friends.position_unavailable")}
              </span>
            </article>
          {/each}
        </div>
      {/if}

      <p class="mt-4 text-xs" style="color: var(--color-muted)">
        {$t("friends.updated", { time: timeStr(snapshot.sourceTimestampMs ?? snapshot.receivedAtMs) })}
      </p>
    </section>
  {/if}
</div>
