<script lang="ts">
  import { onMount } from "svelte";
  import {
    islepilotFriendAction,
    islepilotFriends,
    listenerBag,
    onProviderSnapshot,
    onProviderState,
    providerSnapshot,
    providerState,
    type IslepilotFriend,
    type IslepilotFriendAction,
    type IslepilotFriendsState,
    type ProviderSnapshot,
    type ProviderState,
    type SharedFriend,
  } from "$lib/api";
  import { locale, t } from "$lib/i18n";
  import { providerLabel } from "$lib/provider-ui";

  let connection = $state<ProviderState | null>(null);
  let snapshot = $state<ProviderSnapshot | null>(null);
  let managed = $state<IslepilotFriendsState | null>(null);
  let loading = $state(true);
  let failed = $state(false);
  let managementLoading = $state(false);
  let managementError = $state<string | null>(null);
  let actionBusy = $state<string | null>(null);
  let steamId = $state("");

  const friends = $derived(snapshot?.friends ?? []);
  const positioned = $derived(friends.filter((friend) => friend.positionCm !== null));
  const canManage = $derived(connection?.provider === "isle-pilot");

  async function refreshManaged() {
    if (!canManage) {
      managed = null;
      managementError = null;
      return;
    }
    managementLoading = true;
    managementError = null;
    try {
      managed = await islepilotFriends();
    } catch (error) {
      managementError = String(error);
    } finally {
      managementLoading = false;
    }
  }

  onMount(() => {
    const bag = listenerBag();
    void (async () => {
      try {
        connection = await providerState();
        await bag.add(
          onProviderState((value) => {
            const providerChanged = connection?.provider !== value.provider;
            connection = value;
            if (providerChanged) void refreshManaged();
          }),
        );
        await bag.add(onProviderSnapshot((value) => {
          snapshot = value;
          failed = false;
          loading = false;
        }));
        snapshot = await providerSnapshot();
        await refreshManaged();
      } catch {
        failed = true;
      } finally {
        loading = false;
      }
    })();
    return () => bag.dispose();
  });

  async function runAction(
    action: IslepilotFriendAction,
    value: string | null,
    share: boolean | null = null,
  ) {
    const key = `${action}:${value ?? String(share)}`;
    actionBusy = key;
    managementError = null;
    try {
      managed = await islepilotFriendAction(action, value, share);
      if (action === "add") steamId = "";
    } catch (error) {
      const message = String(error);
      await refreshManaged();
      managementError = message;
    } finally {
      actionBusy = null;
    }
  }

  function submitAdd() {
    const value = steamId.trim();
    if (!/^\d{17}$/.test(value)) {
      managementError = $t("friends.steam_invalid");
      return;
    }
    void runAction("add", value);
  }

  function mapFriend(friend: IslepilotFriend): SharedFriend | undefined {
    const name = friend.name?.trim().toLocaleLowerCase();
    return name ? friends.find((item) => item.name.trim().toLocaleLowerCase() === name) : undefined;
  }

  function friendStatus(friend: IslepilotFriend): string {
    if (friend.status?.toLocaleLowerCase() === "accepted") return $t("friends.accepted");
    return friend.incoming ? $t("friends.incoming") : $t("friends.outgoing");
  }

  function timeStr(ms: number) {
    return new Date(ms).toLocaleTimeString($locale === "vi" ? "vi-VN" : "en-US");
  }
</script>

<div class="mx-auto max-w-3xl space-y-5 p-6">
  <section>
    <h2 class="text-lg font-semibold" style="color: var(--color-accent)">{$t("friends.title")}</h2>
    <p class="mt-1 text-sm" style="color: var(--color-muted)">{$t("friends.subtitle")}</p>
  </section>

  {#if canManage}
    <section class="rounded border p-4" style="border-color: var(--color-border); background: var(--color-panel)">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div>
          <strong class="text-sm" style="color: var(--color-text)">{$t("friends.manage_title")}</strong>
          <p class="mt-1 text-xs" style="color: var(--color-muted)">{$t("friends.manage_hint")}</p>
        </div>
        <button class="rounded border px-3 py-1.5 text-xs disabled:opacity-50" style="border-color: var(--color-border); color: var(--color-accent)" disabled={managementLoading || actionBusy !== null} onclick={() => void refreshManaged()}>{$t("friends.refresh")}</button>
      </div>

      <form class="mt-4 flex gap-2" onsubmit={(event) => { event.preventDefault(); submitAdd(); }}>
        <input class="min-w-0 flex-1 rounded border bg-transparent px-3 py-2 font-mono text-sm outline-none" style="border-color: var(--color-border); color: var(--color-text)" bind:value={steamId} maxlength="17" inputmode="numeric" autocomplete="off" placeholder={$t("friends.steam_placeholder")} aria-label={$t("friends.steam_placeholder")} />
        <button class="rounded border px-4 py-2 text-sm font-semibold disabled:opacity-50" style="border-color: var(--color-accent); color: var(--color-accent); background: rgba(53,242,255,.07)" disabled={actionBusy !== null || managementLoading} type="submit">{actionBusy?.startsWith("add:") ? $t("friends.sending") : $t("friends.add")}</button>
      </form>

      <label class="mt-4 flex cursor-pointer items-center gap-3 rounded border px-3 py-2 text-sm" style="border-color: var(--color-border)">
        <input type="checkbox" checked={managed?.shareLocation === true} disabled={actionBusy !== null || managementLoading || managed === null} onchange={(event) => void runAction("share", null, event.currentTarget.checked)} />
        <span>
          <strong class="block text-xs">{$t("friends.share_location")}</strong>
          <small class="text-xs normal-case tracking-normal" style="color: var(--color-muted)">{$t("friends.share_hint")}</small>
        </span>
      </label>

      {#if managementError}
        <p class="mt-3 rounded border px-3 py-2 text-xs" style="border-color: #65472a; color: #ffd277">{$t("friends.action_error", { error: managementError })}</p>
      {/if}

      {#if managementLoading && !managed}
        <p class="mt-4 text-sm" style="color: var(--color-muted)">{$t("friends.loading")}</p>
      {:else if managed}
        <div class="mt-4 text-xs" style="color: var(--color-muted)">
          {$t("friends.count", { count: managed.used ?? managed.friends.length, limit: managed.limit ?? "∞" })}
        </div>
        {#if managed.friends.length === 0}
          <p class="mt-3 text-sm" style="color: var(--color-muted)">{$t("friends.provider_empty")}</p>
        {:else}
          <div class="mt-3 grid gap-2">
            {#each managed.friends as friend (friend.id ?? friend.steamId ?? friend.name)}
              {@const live = mapFriend(friend)}
              {@const accepted = friend.status?.toLocaleLowerCase() === "accepted"}
              {@const relationId = friend.id ?? ""}
              <article class="flex flex-wrap items-center gap-x-3 gap-y-2 rounded border px-3 py-2" style="border-color: var(--color-border); background: rgba(7,16,29,.62)">
                <div class="min-w-0 flex-1">
                  <strong class="block truncate text-sm">{friend.name ?? friend.steamId ?? $t("friends.unknown_player")}</strong>
                  <span class="text-xs" style="color: var(--color-muted)">{live?.dinoName ?? friend.dinoName ?? friend.species ?? friendStatus(friend)}</span>
                </div>
                <span class="rounded-full px-2 py-0.5 text-xs" style={accepted ? "background:#0d3028;color:#45f5a2" : "background:#352814;color:#ffd277"}>{friendStatus(friend)}</span>
                {#if live}<span class="text-xs" style="color: var(--color-muted)">{live.positionCm ? $t("friends.position_available") : $t("friends.position_unavailable")}</span>{/if}
                {#if !accepted && friend.incoming}
                  <button class="rounded border px-2 py-1 text-xs disabled:opacity-50" style="border-color:#23694e;color:#45f5a2" disabled={!relationId || actionBusy !== null} onclick={() => void runAction("accept", relationId)}>{$t("friends.accept")}</button>
                  <button class="rounded border px-2 py-1 text-xs disabled:opacity-50" style="border-color:#754052;color:#ff7895" disabled={!relationId || actionBusy !== null} onclick={() => void runAction("decline", relationId)}>{$t("friends.decline")}</button>
                {:else}
                  <button class="rounded border px-2 py-1 text-xs disabled:opacity-50" style="border-color:#754052;color:#ff7895" disabled={!relationId || actionBusy !== null} onclick={() => void runAction("remove", relationId)}>{accepted ? $t("friends.remove") : $t("friends.cancel")}</button>
                {/if}
              </article>
            {/each}
          </div>
        {/if}
      {/if}
    </section>
  {/if}

  {#if !connection?.provider}
    <section class="rounded border p-4 text-sm" style="border-color: var(--color-border); background: var(--color-panel); color: var(--color-muted)">{$t("friends.connect")}</section>
  {:else if loading}
    <section class="rounded border p-4 text-sm" style="border-color: var(--color-border); background: var(--color-panel); color: var(--color-muted)">{$t("friends.loading")}</section>
  {:else if failed}
    <section class="rounded border p-4 text-sm" style="border-color:#65472a;background:var(--color-panel);color:#ffd277">{$t("friends.error")}</section>
  {:else if !snapshot}
    <section class="rounded border p-4 text-sm" style="border-color: var(--color-border); background: var(--color-panel); color: var(--color-muted)">{$t("friends.waiting")}</section>
  {:else if !canManage}
    <section class="rounded border p-4" style="border-color: var(--color-border); background: var(--color-panel)">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div><strong class="text-sm" style="color: var(--color-text)">{providerLabel(snapshot.provider)}</strong>{#if snapshot.serverName}<span class="ml-2 text-sm" style="color: var(--color-muted)">· {snapshot.serverName}</span>{/if}</div>
        <span class="font-mono text-xs" style="color: var(--color-muted)">{$t("friends.positioned", { count: positioned.length, total: friends.length })}</span>
      </div>
      {#if friends.length === 0}
        <p class="mt-4 text-sm" style="color: var(--color-muted)">{$t("friends.provider_empty")}</p>
      {:else}
        <div class="mt-4 grid gap-2">
          {#each friends as friend (friend.slot ?? friend.name)}
            <article class="flex flex-wrap items-center gap-x-3 gap-y-1 rounded border px-3 py-2" style="border-color: var(--color-border); background: rgba(7,16,29,.62)">
              <div class="min-w-0 flex-1"><strong class="block truncate text-sm">{friend.name}</strong><span class="text-xs" style="color: var(--color-muted)">{friend.dinoName ?? $t("friends.dino_unknown")}</span></div>
              <span class="rounded-full px-2 py-0.5 text-xs" style={friend.online ? "background:#0d3028;color:#45f5a2" : "background:#352814;color:#ffd277"}>{friend.online ? $t("friends.online") : $t("friends.offline")}</span>
              <span class="text-xs" style="color: var(--color-muted)">{friend.positionCm ? $t("friends.position_available") : $t("friends.position_unavailable")}</span>
            </article>
          {/each}
        </div>
      {/if}
      <p class="mt-4 text-xs" style="color: var(--color-muted)">{$t("friends.updated", { time: timeStr(snapshot.sourceTimestampMs ?? snapshot.receivedAtMs) })}</p>
    </section>
  {/if}
</div>
