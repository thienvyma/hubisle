<script lang="ts">
  import { onDestroy } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import {
    providerGarage,
    providerGarageAction,
    type ProviderId,
  } from "$lib/api";
  import { t, tNow } from "$lib/i18n";
  import { providerLabel } from "$lib/provider-ui";

  let { provider }: { provider: ProviderId } = $props();

  type Obj = Record<string, unknown>;
  interface GarageSlot {
    number: number;
    apiSlot: number;
    stored: boolean;
    locked: boolean;
    restoring: boolean;
    species: string | null;
    gender: string | null;
    growthPct: number | null;
    prime: boolean | null;
    mutations: string[];
    stateHash: string | null;
  }
  interface GarageView {
    slots: GarageSlot[];
    entitlement: number;
    vip: boolean;
    online: boolean;
    bridge: boolean;
    uncertain: boolean;
    busy: boolean;
    countdownSeconds: number | null;
  }

  const obj = (value: unknown): Obj =>
    value !== null && typeof value === "object" && !Array.isArray(value)
      ? (value as Obj)
      : {};
  const bool = (value: unknown): boolean => value === true;
  const text = (value: unknown): string | null =>
    typeof value === "string" && value.trim() ? value.trim() : null;
  const finite = (value: unknown): number | null =>
    typeof value === "number" && Number.isFinite(value) ? value : null;
  const clampSlots = (value: unknown, fallback = 3): number =>
    Math.max(1, Math.min(5, Math.trunc(finite(value) ?? fallback)));
  const growthPercent = (value: unknown): number | null => {
    const number = finite(value);
    if (number === null) return null;
    return Math.max(0, Math.min(100, number <= 1.5 ? number * 100 : number));
  };
  const mutationList = (value: unknown): string[] =>
    Array.isArray(value)
      ? value.filter((item): item is string => typeof item === "string" && item.trim().length > 0)
      : [];

  let raw = $state<Obj>({});
  let loading = $state(false);
  let error = $state<string | null>(null);
  let note = $state<string | null>(null);
  let loadedAt = $state<number | null>(null);
  let providerSeen = $state<ProviderId | null>(null);
  let clock = $state(Date.now());

  const timer = window.setInterval(() => {
    clock = Date.now();
    if (current.busy && !loading) void load(false);
  }, 3_000);
  onDestroy(() => window.clearInterval(timer));

  function normalizeEra(source: Obj): GarageView {
    const data = obj(source.data);
    const entitlement = clampSlots(data.slotCount);
    const rows = Array.isArray(data.slots) ? data.slots.map(obj) : [];
    const slots = Array.from({ length: entitlement }, (_, index): GarageSlot => {
      const number = index + 1;
      const row = rows.find((item) => finite(item.slot) === number) ?? {};
      const dino = obj(row.storedDino);
      return {
        number,
        apiSlot: number,
        stored: bool(row.stored),
        locked: bool(row.stored) && row.parkCommitted === false,
        restoring: bool(row.restoreInProgress),
        species: text(dino.species),
        gender: text(dino.gender),
        growthPct: growthPercent(dino.growthPercent),
        prime: typeof dino.primeEligible === "boolean" ? dino.primeEligible : null,
        mutations: mutationList(dino.mutations),
        stateHash: text(dino.stateHash),
      };
    });
    return {
      slots,
      entitlement,
      vip: bool(data.isVip),
      online: bool(data.onlinePawn),
      bridge: true,
      uncertain: false,
      busy: bool(data.restoreInProgress) || slots.some((slot) => slot.restoring),
      countdownSeconds: null,
    };
  }

  function normalizeTitan(source: Obj): GarageView {
    const entitlement = clampSlots(source.max);
    const rows = obj(source.slots);
    const pending = obj(source.pending);
    // Titan always displays all five physical cells. Empty cells above the
    // account entitlement remain locked, while a dino stored during an older
    // VIP period can still be restored or deleted.
    const slots = Array.from({ length: 5 }, (_, index): GarageSlot => {
      const dino = obj(rows[String(index)]);
      const stored = Object.keys(dino).length > 0;
      const female = dino.female;
      return {
        number: index + 1,
        apiSlot: index,
        stored,
        locked: index >= entitlement && !stored,
        restoring: false,
        species: text(dino.class),
        gender: female === true ? "Female" : female === false ? "Male" : null,
        growthPct: growthPercent(dino.growth),
        prime: typeof dino.prime === "boolean" ? dino.prime : null,
        mutations: mutationList(dino.mutations),
        stateHash: null,
      };
    });
    const countdown = finite(source.dem_nguoc);
    const age = finite(pending.tuoi) ?? 0;
    return {
      slots,
      entitlement,
      vip: bool(source.is_vip),
      online: bool(source.online),
      bridge: source.bridge !== false,
      uncertain: bool(source.khong_ro),
      busy: Object.keys(pending).length > 0,
      countdownSeconds:
        pending.action === "garage_store" && countdown !== null
          ? Math.max(0, Math.ceil(countdown - age - (clock - loadedAtSafe()) / 1000))
          : null,
    };
  }

  function loadedAtSafe(): number {
    return loadedAt ?? clock;
  }

  const current = $derived(
    provider === "era" ? normalizeEra(raw) : normalizeTitan(raw),
  );

  $effect(() => {
    if (provider !== providerSeen) {
      providerSeen = provider;
      raw = {};
      note = null;
      void load(true);
    }
  });

  async function load(showLoading: boolean) {
    if (showLoading) loading = true;
    try {
      const response = await providerGarage();
      if (response.provider !== provider) return;
      const next = obj(response.data);
      const success = provider === "era" ? next.success === true : next.ok === true;
      if (!success) {
        throw new Error(text(next.message) ?? text(next.error) ?? tNow("server_garage.failed"));
      }
      raw = next;
      error = null;
      loadedAt = Date.now();
    } catch (reason) {
      error = String(reason);
    } finally {
      loading = false;
    }
  }

  function responseError(source: Obj): string | null {
    const success = provider === "era" ? source.success === true : source.ok === true;
    if (success) return null;
    return text(source.message) ?? text(source.error) ?? tNow("server_garage.failed");
  }

  async function run(action: "park" | "restore" | "delete" | "cancel", slot?: GarageSlot) {
    const name = slot?.species ?? `Slot ${slot?.number ?? ""}`;
    const prompt = action === "park"
      ? tNow(provider === "titan" ? "server_garage.confirm_park_titan" : "server_garage.confirm_park_era", { slot: slot?.number ?? "" })
      : action === "restore"
        ? tNow("server_garage.confirm_restore", { name })
        : action === "delete"
          ? tNow("server_garage.confirm_delete", { name })
          : null;
    if (prompt && !(await ask(prompt, { title: tNow("garage.title"), kind: "warning" }))) return;
    loading = true;
    error = null;
    note = null;
    try {
      const response = await providerGarageAction(
        action,
        slot?.apiSlot ?? null,
        action === "delete" ? slot?.stateHash ?? null : null,
      );
      const data = obj(response.data);
      const failure = responseError(data);
      if (failure) throw new Error(failure);
      note = text(data.message) ?? tNow("garage.done");
      await load(false);
    } catch (reason) {
      const actionError = String(reason);
      await load(false);
      if (current.busy) note = tNow("server_garage.pending");
      else error = actionError;
    } finally {
      loading = false;
    }
  }

  function canUse(slot: GarageSlot, action: "park" | "restore" | "delete"): boolean {
    if (loading || current.busy || slot.restoring) return false;
    if (slot.locked && action !== "delete") return false;
    if (provider === "titan" && (!current.online || !current.bridge || current.uncertain)) return false;
    if (action === "park") return current.online && !slot.stored;
    if (action === "restore") return current.online && slot.stored;
    return slot.stored && (provider !== "era" || /^[0-9a-f]{64}$/i.test(slot.stateHash ?? ""));
  }
</script>

<div class="garage-page">
  <header class="page-head">
    <div>
      <span class="eyebrow">SERVER STORAGE</span>
      <h2>{$t("server_garage.title")}</h2>
      <p>{$t("server_garage.subtitle", { provider: providerLabel(provider) })}</p>
    </div>
    <button class="outline" disabled={loading} onclick={() => void load(true)}>{$t("garage.refresh")}</button>
  </header>

  <div class="status-row">
    <span class:good={current.online}>{current.online ? "ONLINE" : "OFFLINE"}</span>
    <span>{current.vip ? "VIP" : $t("server_garage.standard")} · {current.entitlement} SLOT</span>
    {#if !current.bridge}<span class="bad">BRIDGE OFFLINE</span>{/if}
    {#if current.uncertain}<span class="bad">{$t("server_garage.uncertain")}</span>{/if}
  </div>

  {#if current.busy}
    <div class="notice warning">
      {$t("server_garage.pending")}
      {#if current.countdownSeconds !== null}
        <strong>{$t("server_garage.countdown", { seconds: current.countdownSeconds })}</strong>
        <button class="danger" disabled={loading} onclick={() => void run("cancel")}>{$t("btn.cancel")}</button>
      {/if}
    </div>
  {/if}
  {#if error}<div class="notice error">{error}</div>{/if}
  {#if note}<div class="notice success">{note}</div>{/if}

  <div class="slot-grid">
    {#each current.slots as slot (slot.number)}
      <article class:locked={slot.locked} class:filled={slot.stored} class="slot-card">
        <div class="slot-head">
          <span>SLOT // {String(slot.number).padStart(2, "0")}</span>
          <strong>{slot.locked ? $t("server_garage.locked") : slot.restoring ? $t("server_garage.restoring") : slot.stored ? $t("server_garage.stored") : $t("server_garage.empty")}</strong>
        </div>
        {#if slot.stored}
          <div class="dino-mark">{slot.species?.slice(0, 2).toUpperCase() ?? "DI"}</div>
          <h3>{slot.species ?? "Dino"}</h3>
          <div class="meta">
            <span>{slot.gender ?? "—"}</span>
            <span>{slot.growthPct === null ? "—" : `${slot.growthPct.toFixed(1)}%`}</span>
            {#if slot.prime !== null}<span class:prime={slot.prime}>PRIME {slot.prime ? "ON" : "OFF"}</span>{/if}
          </div>
          {#if slot.mutations.length}<p class="mutations">{slot.mutations.join(" · ")}</p>{/if}
          <div class="actions">
            <button disabled={!canUse(slot, "restore")} onclick={() => void run("restore", slot)}>{$t("server_garage.restore")}</button>
            <button class="danger" disabled={!canUse(slot, "delete")} onclick={() => void run("delete", slot)}>{$t("server_garage.delete")}</button>
          </div>
        {:else}
          <div class="empty-visual"><i></i><span>{$t("server_garage.empty_hint")}</span></div>
          <div class="actions">
            <button disabled={!canUse(slot, "park")} onclick={() => void run("park", slot)}>{$t("server_garage.park")}</button>
          </div>
        {/if}
      </article>
    {/each}
  </div>

  <p class="foot-note">{$t(provider === "titan" ? "server_garage.titan_note" : "server_garage.era_note")}</p>
</div>

<style>
  .garage-page { max-width: 1100px; margin: 0 auto; padding: 28px; color: var(--color-text); }
  .page-head { display:flex; justify-content:space-between; align-items:flex-start; gap:24px; margin-bottom:18px; }
  .eyebrow { color:var(--color-accent); font:9px Consolas,monospace; letter-spacing:.2em; }
  h2 { margin:5px 0 3px; font-size:22px; } h3 { margin:14px 0 8px; font-size:18px; }
  p { color:var(--color-muted); font-size:12px; }
  button { border:1px solid rgba(53,242,255,.35); background:rgba(53,242,255,.1); color:var(--color-text); padding:8px 12px; font:11px Consolas,monospace; cursor:pointer; }
  button:hover:not(:disabled) { border-color:var(--color-accent); background:rgba(53,242,255,.18); }
  button:disabled { cursor:not-allowed; opacity:.35; }
  button.danger { border-color:rgba(255,86,120,.5); background:rgba(255,86,120,.1); }
  .status-row { display:flex; flex-wrap:wrap; gap:8px; margin:16px 0; }
  .status-row span { border:1px solid var(--color-border); padding:5px 9px; color:var(--color-muted); font:9px Consolas,monospace; letter-spacing:.08em; }
  .status-row .good { border-color:rgba(69,245,162,.45); color:#45f5a2; } .status-row .bad { color:#ff8a80; }
  .notice { display:flex; align-items:center; gap:12px; margin:12px 0; border:1px solid var(--color-border); padding:10px 12px; font-size:12px; }
  .notice.warning { color:#ffd591; } .notice.error { color:#ff8a80; } .notice.success { color:#72d653; }
  .notice button { margin-left:auto; }
  .slot-grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(255px,1fr)); gap:13px; }
  .slot-card { position:relative; min-height:230px; overflow:hidden; border:1px solid #16354e; background:linear-gradient(145deg,rgba(9,25,42,.96),rgba(4,11,22,.96)); padding:16px; clip-path:polygon(0 0,calc(100% - 14px) 0,100% 14px,100% 100%,0 100%); }
  .slot-card.filled { border-color:rgba(53,242,255,.37); } .slot-card.locked { opacity:.48; }
  .slot-card::after { content:""; position:absolute; width:120px; height:120px; right:-55px; top:-55px; border:1px solid rgba(53,242,255,.1); border-radius:50%; }
  .slot-head { display:flex; justify-content:space-between; color:#58738d; font:9px Consolas,monospace; letter-spacing:.1em; }
  .slot-head strong { color:var(--color-accent); }
  .dino-mark { display:flex; align-items:center; justify-content:center; width:48px; height:48px; margin-top:20px; border:1px solid rgba(198,92,255,.55); border-radius:50%; color:#dca2ff; font:15px Consolas,monospace; box-shadow:0 0 25px rgba(198,92,255,.1); }
  .meta { display:flex; flex-wrap:wrap; gap:6px; }
  .meta span { border:1px solid #1d3a51; padding:4px 7px; color:var(--color-muted); font:9px Consolas,monospace; }
  .meta span.prime { color:#ffd166; border-color:rgba(255,209,102,.45); }
  .mutations { min-height:24px; margin:10px 0; color:#9db2c7; }
  .actions { display:flex; gap:8px; margin-top:18px; }
  .empty-visual { display:flex; min-height:143px; flex-direction:column; align-items:center; justify-content:center; gap:12px; color:#50677d; font:10px Consolas,monospace; text-align:center; }
  .empty-visual i { width:44px; height:44px; border:1px dashed #29465d; transform:rotate(45deg); }
  .foot-note { margin-top:18px; border-left:2px solid var(--color-accent); padding-left:10px; line-height:1.5; }
</style>
