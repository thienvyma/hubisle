<script lang="ts">
  import { onDestroy } from "svelte";
  import { providerSkinApply, providerSkinState, type ProviderId } from "$lib/api";
  import { t, tNow } from "$lib/i18n";
  import { providerLabel } from "$lib/provider-ui";

  let { provider }: { provider: ProviderId } = $props();
  type Obj = Record<string, unknown>;

  const ERA_NAMES = ["Thân", "Lưng", "Màng cánh", "Cổ", "Mõm", "Đuôi", "Chân & mắt"];
  const TITAN_NAMES = ["Màu con đực", "Hoa văn", "Thân", "Sườn", "Bụng", "Chi tiết", "Mắt"];
  const ERA_DEFAULT = ["#16A34A", "#3F6212", "#EAB308", "#78350F", "#C08457", "#111827", "#F3F4F6"];
  const TITAN_DEFAULT = ["#B06A3C", "#5A3C28", "#7A5A3C", "#6A4A30", "#C8B090", "#3C2C1C", "#D0A020"];
  const ISLEPILOT_DEFAULT = ["#7A5A3C", "#5A3C28", "#C8B090", "#8A6A42", "#A88A5A", "#6A5230", "#D0A020", "#F0E0B8", "#C8C8C8", "#3C2C1C"];
  const ERA_FREE = ["#111827", "#F3F4F6", "#6B7280", "#DC2626", "#7F1D1D", "#F97316", "#EAB308", "#16A34A", "#3F6212", "#2563EB", "#0891B2", "#7C3AED", "#DB2777", "#78350F", "#C08457", "#D6B38B"];
  const PRESETS: Record<string, { era: string[]; titan: string[] }> = {
    forest: {
      era: ["#1C4D3B", "#5CA65E", "#D1B75A", "#55733C", "#D9E6B4", "#203C3A", "#F1E9C8"],
      titan: ["#4A5A2C", "#2C3A1C", "#3E4A26", "#334020", "#8A9A6A", "#1C260F", "#C8D020"],
    },
    desert: {
      era: ["#8B2C21", "#E15F2D", "#F2C24D", "#B74325", "#FFF0A0", "#682D26", "#FFF7D1"],
      titan: ["#C8A86A", "#9A7A4A", "#BFA070", "#A88A5A", "#E0D0A8", "#6A5230", "#E0B020"],
    },
    shadow: {
      era: ["#22234D", "#4543A1", "#A84EAF", "#34356F", "#E08FD0", "#272746", "#F3D7F2"],
      titan: ["#3A3A42", "#202028", "#2C2C34", "#26262C", "#4A4A52", "#101014", "#8060C0"],
    },
    snow: {
      era: ["#123F4B", "#1B8C83", "#63D3A4", "#246272", "#D9F4B8", "#102B3D", "#E5FFF2"],
      titan: ["#D8DDE4", "#A8B0BC", "#C4CCD6", "#B4BCC8", "#EEF2F6", "#7A8290", "#60A0D0"],
    },
  };

  const obj = (value: unknown): Obj =>
    value !== null && typeof value === "object" && !Array.isArray(value)
      ? (value as Obj)
      : {};
  const finite = (value: unknown): number | null =>
    typeof value === "number" && Number.isFinite(value) ? value : null;
  const text = (value: unknown): string | null =>
    typeof value === "string" && value.trim() ? value.trim() : null;
  const validColor = (value: unknown): value is string =>
    typeof value === "string" && /^#[0-9a-f]{6}$/i.test(value);

  let raw = $state<Obj>({});
  let fieldKeys = $state<string[]>([]);
  let fieldLabels = $state<string[]>([]);
  let colors = $state<string[]>([...TITAN_DEFAULT]);
  let variation = $state(0);
  let activeZone = $state(0);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let note = $state<string | null>(null);
  let cooldownUntil = $state(0);
  let clock = $state(Date.now());
  let providerSeen = $state<ProviderId | null>(null);

  const names = $derived(provider === "era" ? ERA_NAMES : provider === "titan" ? TITAN_NAMES : fieldLabels);
  const fullColor = $derived(provider === "isle-pilot" || provider !== "era" || raw.arbitraryHex === true);
  const lockedZones = $derived(
    Array.isArray(raw.lockedZones)
      ? raw.lockedZones.filter((zone): zone is string => typeof zone === "string")
      : [],
  );
  const available = $derived(
    provider === "era"
      ? raw.success === true && raw.available === true
      : provider === "titan"
        ? raw.ok === true
        : raw.skinLiveEnabled !== false && raw.allowed !== false && raw.hasDino !== false,
  );
  const cooldown = $derived(Math.max(0, Math.ceil((cooldownUntil - clock) / 1000)));

  const timer = window.setInterval(() => (clock = Date.now()), 1_000);
  onDestroy(() => window.clearInterval(timer));

  $effect(() => {
    if (provider !== providerSeen) {
      providerSeen = provider;
      raw = {};
      note = null;
      error = null;
      colors = loadDraft(provider);
      variation = loadVariation(provider);
      if (provider !== "isle-pilot") {
        fieldKeys = [];
        fieldLabels = [];
      }
      void refresh();
    }
  });

  function storageKey(current: ProviderId) {
    return `isle-pulse.skin.${current}.v1`;
  }
  function loadDraft(current: ProviderId): string[] {
    const fallback = current === "era" ? ERA_DEFAULT : current === "titan" ? TITAN_DEFAULT : ISLEPILOT_DEFAULT;
    try {
      const value = JSON.parse(localStorage.getItem(storageKey(current)) ?? "null");
      if (Array.isArray(value) && value.length === fallback.length && value.every(validColor)) {
        return value.map((color) => color.toUpperCase());
      }
    } catch (_) {}
    return [...fallback];
  }
  function loadVariation(current: ProviderId): number {
    const value = Number(localStorage.getItem(`${storageKey(current)}.variation`) ?? 0);
    return Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0;
  }
  function saveDraft() {
    localStorage.setItem(storageKey(provider), JSON.stringify(colors));
    localStorage.setItem(`${storageKey(provider)}.variation`, String(variation));
  }
  function setColor(index: number, color: string) {
    if (!validColor(color)) return;
    colors[index] = color.toUpperCase();
    colors = [...colors];
    saveDraft();
  }
  function applyPreset(name: string) {
    if (provider === "isle-pilot") return;
    const preset = PRESETS[name];
    if (!preset) return;
    const next = provider === "era" ? preset.era : preset.titan;
    colors = fullColor ? [...next] : next.map(nearestFreeColor);
    saveDraft();
  }
  function nearestFreeColor(value: string): string {
    if (ERA_FREE.includes(value.toUpperCase())) return value.toUpperCase();
    const rgb = (hex: string) => [1, 3, 5].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16));
    const source = rgb(value);
    return ERA_FREE.reduce(
      (best, item) => {
        const candidate = rgb(item);
        const distance = candidate.reduce((sum, channel, index) => sum + (channel - source[index]) ** 2, 0);
        return distance < best.distance ? { color: item, distance } : best;
      },
      { color: ERA_FREE[0], distance: Number.POSITIVE_INFINITY },
    ).color;
  }
  function formatTime(seconds: number): string {
    return `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(seconds % 60).padStart(2, "0")}`;
  }
  function setCooldown(seconds: unknown) {
    const value = Math.max(0, Math.trunc(finite(seconds) ?? 0));
    cooldownUntil = value ? Date.now() + value * 1_000 : 0;
  }
  function readIslePilotFields(source: Obj) {
    const fields = Array.isArray(source.fields) ? source.fields.map(obj) : [];
    fieldKeys = fields
      .map((field) => (typeof field.key === "string" ? field.key : null))
      .filter((key): key is string => key !== null);
    fieldLabels = fields
      .map((field, index) => text(field.label) ?? fieldKeys[index] ?? `Zone ${index + 1}`);
    const current = obj(source.current);
    const defaults = obj(source.defaults);
    const fallback = loadDraft("isle-pilot");
    const next = fieldKeys.map((key, index) => {
      const value = current[key] ?? defaults[key] ?? fallback[index] ?? ISLEPILOT_DEFAULT[index % ISLEPILOT_DEFAULT.length];
      return validColor(value) ? value.toUpperCase() : ISLEPILOT_DEFAULT[index % ISLEPILOT_DEFAULT.length];
    });
    if (next.length > 0) {
      colors = next;
      saveDraft();
    }
  }
  function zoneLocked(index: number): boolean {
    return provider === "isle-pilot" && lockedZones.includes(fieldKeys[index] ?? "");
  }

  async function refresh() {
    loading = true;
    try {
      const response = await providerSkinState();
      if (response.provider !== provider) return;
      raw = obj(response.data);
      if (provider === "era") {
        if (raw.success !== true) throw new Error(String(raw.message ?? tNow("skin.failed")));
        const saved = raw.savedColors;
        if (Array.isArray(saved) && saved.length === 7 && saved.every(validColor)) {
          colors = saved.map((color) => color.toUpperCase());
          saveDraft();
        }
        setCooldown(raw.cooldownRemainingSeconds);
      } else {
        if (provider === "titan") {
          if (raw.ok !== true) throw new Error(String(raw.error ?? tNow("skin.failed")));
          setCooldown(raw.con_giay);
        } else {
          readIslePilotFields(raw);
          setCooldown(0);
        }
      }
      error = null;
    } catch (reason) {
      error = String(reason);
    } finally {
      loading = false;
    }
  }

  async function apply() {
    loading = true;
    error = null;
    note = null;
    try {
      const response = await providerSkinApply(colors, variation);
      const data = obj(response.data);
      const success = provider === "era" ? data.success === true : data.ok !== false;
      if (!success) throw new Error(String(data.message ?? data.error ?? tNow("skin.failed")));
      setCooldown(provider === "era" ? data.cooldownRemainingSeconds : data.skin_con_giay);
      note = String(data.message ?? tNow("skin.done"));
      saveDraft();
    } catch (reason) {
      error = String(reason);
    } finally {
      loading = false;
    }
  }
</script>

<div class="skin-page">
  <header class="page-head">
    <div>
      <span class="eyebrow">LIVE CUSTOMIZER</span>
      <h2>{$t("skin.title")}</h2>
      <p>{$t("skin.subtitle", { provider: providerLabel(provider) })}</p>
    </div>
    <button class="outline" disabled={loading} onclick={() => void refresh()}>{$t("garage.refresh")}</button>
  </header>

  <div class="tier-row">
    <span class:good={available}>{available ? "READY" : "LOCKED"}</span>
    {#if provider === "era"}<span>{fullColor ? "VIP · FULL COLOR" : "FREE · 16 COLOR"}</span>{/if}
    {#if provider === "isle-pilot" && raw.glitchEnabled === true}<span>GLITCH</span>{/if}
    {#if cooldown > 0}<span class="waiting">{$t("skin.cooldown", { time: formatTime(cooldown) })}</span>{/if}
  </div>

  {#if error}<div class="notice error">{error}</div>{/if}
  {#if note}<div class="notice success">{note}</div>{/if}

  <section class="preview-panel">
    <div class="preview-head"><span>{$t("skin.preview")}</span><strong>{providerLabel(provider)}</strong></div>
    <div class="preview-strip">
      {#each colors as color}<i style:background={color}></i>{/each}
    </div>
  </section>

  {#if provider !== "isle-pilot"}
    <section class="preset-row">
      <span>{$t("skin.presets")}</span>
      {#each Object.keys(PRESETS) as preset}
        <button onclick={() => applyPreset(preset)}>{$t(`skin.preset_${preset}` as "skin.preset_forest")}</button>
      {/each}
    </section>
  {/if}

  <div class="color-grid">
    {#each names as name, index}
      <article class:active={!fullColor && activeZone === index} class:locked={zoneLocked(index)}>
        <span class="zone-index">{String(index + 1).padStart(2, "0")}</span>
        <button class="zone-name" onclick={() => (activeZone = index)} disabled={zoneLocked(index)}>{name}</button>
        <input id={`skin-${index}`} aria-label={name} type="color" value={colors[index]} disabled={!fullColor || zoneLocked(index)} oninput={(event) => setColor(index, event.currentTarget.value)} />
        <code>{colors[index]}</code>
      </article>
    {/each}
  </div>

  {#if provider === "era" && !fullColor}
    <section class="free-palette">
      <span>{$t("skin.choose_for", { zone: names[activeZone] })}</span>
      <div>
        {#each ERA_FREE as color}
          <button class:selected={colors[activeZone] === color} title={color} style:background={color} onclick={() => setColor(activeZone, color)}></button>
        {/each}
      </div>
    </section>
  {/if}

  {#if provider === "titan"}
    <label class="variation">
      <span>{$t("skin.variation")} <strong>{Math.round(variation * 100)}%</strong></span>
      <input type="range" min="0" max="1" step="0.01" bind:value={variation} oninput={saveDraft} />
    </label>
  {/if}

  <div class="apply-row">
    <p>{$t("skin.online_only")}</p>
    <button class="apply" disabled={loading || !available || cooldown > 0 || colors.length === 0} onclick={() => void apply()}>
      {loading ? $t("skin.applying") : cooldown > 0 ? $t("skin.cooldown", { time: formatTime(cooldown) }) : $t("skin.apply")}
    </button>
  </div>
</div>

<style>
  .skin-page { max-width:1050px; margin:0 auto; padding:28px; color:var(--color-text); }
  .page-head { display:flex; justify-content:space-between; align-items:flex-start; gap:24px; }
  .eyebrow { color:var(--color-accent); font:9px Consolas,monospace; letter-spacing:.2em; }
  h2 { margin:5px 0 3px; font-size:22px; }
  p { color:var(--color-muted); font-size:12px; }
  button { border:1px solid rgba(53,242,255,.35); background:rgba(53,242,255,.08); color:var(--color-text); padding:7px 11px; font:10px Consolas,monospace; cursor:pointer; }
  button:hover:not(:disabled) { border-color:var(--color-accent); } button:disabled { opacity:.35; cursor:not-allowed; }
  .tier-row { display:flex; flex-wrap:wrap; gap:8px; margin:18px 0; }
  .tier-row span { border:1px solid var(--color-border); padding:5px 9px; color:var(--color-muted); font:9px Consolas,monospace; }
  .tier-row .good { border-color:rgba(69,245,162,.45); color:#45f5a2; } .tier-row .waiting { color:#ffd591; }
  .notice { margin:12px 0; border:1px solid var(--color-border); padding:10px 12px; font-size:12px; }
  .notice.error { color:#ff8a80; } .notice.success { color:#72d653; }
  .preview-panel { margin:16px 0; border:1px solid #173951; background:rgba(6,18,32,.85); padding:14px; }
  .preview-head { display:flex; justify-content:space-between; margin-bottom:9px; color:#68839b; font:9px Consolas,monospace; letter-spacing:.1em; }
  .preview-head strong { color:var(--color-accent); }
  .preview-strip { display:flex; height:40px; overflow:hidden; clip-path:polygon(0 0,calc(100% - 10px) 0,100% 10px,100% 100%,0 100%); }
  .preview-strip i { flex:1; box-shadow:inset -1px 0 rgba(0,0,0,.25); }
  .preset-row { display:flex; flex-wrap:wrap; align-items:center; gap:7px; margin:14px 0; }
  .preset-row > span { margin-right:5px; color:var(--color-muted); font:9px Consolas,monospace; }
  .color-grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(235px,1fr)); gap:10px; }
  .color-grid article { display:grid; grid-template-columns:25px 1fr 44px; grid-template-rows:auto auto; align-items:center; column-gap:10px; border:1px solid #16354e; background:linear-gradient(135deg,rgba(8,23,39,.95),rgba(4,11,22,.95)); padding:11px; cursor:pointer; }
  .color-grid article.active { border-color:var(--color-accent); box-shadow:inset 3px 0 var(--color-accent); }
  .color-grid article.locked { opacity:.45; }
  .zone-index { grid-row:1/3; color:#48647c; font:9px Consolas,monospace; }
  .zone-name { border:0; background:transparent; padding:0; color:var(--color-text); font:inherit; font-size:12px; text-align:left; }
  .color-grid input { grid-row:1/3; grid-column:3; width:42px; height:34px; border:0; background:none; }
  .color-grid code { color:#607b93; font-size:9px; }
  .free-palette { margin:15px 0; border:1px solid var(--color-border); padding:13px; }
  .free-palette > span { display:block; margin-bottom:9px; color:var(--color-muted); font-size:11px; }
  .free-palette div { display:flex; flex-wrap:wrap; gap:7px; }
  .free-palette button { width:30px; height:30px; padding:0; border:2px solid transparent; }
  .free-palette button.selected { border-color:white; box-shadow:0 0 10px rgba(255,255,255,.35); }
  .variation { display:block; margin:16px 0; border:1px solid var(--color-border); padding:13px; }
  .variation span { display:flex; justify-content:space-between; color:var(--color-muted); font-size:11px; }
  .variation input { width:100%; margin-top:10px; accent-color:var(--color-accent); }
  .apply-row { display:flex; align-items:center; justify-content:space-between; gap:20px; margin-top:18px; }
  .apply-row p { max-width:620px; line-height:1.5; }
  button.apply { min-width:190px; border-color:var(--color-accent); background:var(--color-accent); color:#03101a; font-weight:bold; }
</style>
