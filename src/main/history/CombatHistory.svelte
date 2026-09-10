<script lang="ts">
  import { onMount } from "svelte";
  import {
    combatHistory,
    listenerBag,
    onCombatEvent,
    type CombatDirection,
    type CombatEvent,
  } from "$lib/api";
  import { locale, t } from "$lib/i18n";

  type Filter = "all" | CombatDirection;
  let events = $state<CombatEvent[]>([]);
  let filter = $state<Filter>("all");
  let loading = $state(true);
  let error = $state(false);
  const visible = $derived(
    filter === "all" ? events : events.filter((event) => event.direction === filter),
  );

  onMount(() => {
    const bag = listenerBag();
    void (async () => {
      try {
        events = await combatHistory();
      } catch {
        error = true;
      } finally {
        loading = false;
      }
      await bag.add(
        onCombatEvent((event) => {
          events = [event, ...events.filter((item) => item.id !== event.id)].slice(0, 500);
        }),
      );
    })();
    return () => bag.dispose();
  });

  function eventLabel(event: CombatEvent): string {
    if (event.source === "health-delta") return $t("history.health_drop");
    if (event.direction === "outgoing") return $t("history.outgoing");
    if (event.direction === "death") return $t("history.death");
    return $t("history.incoming");
  }

  function formatTime(value: number): string {
    return new Date(value).toLocaleString($locale === "en" ? "en-US" : "vi-VN", {
      day: "2-digit",
      month: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  function damage(value: number | null): string {
    return value === null ? "—" : `${Math.round(value * 10) / 10}%`;
  }
</script>

<section class="history-shell">
  <div class="intro">
    <div>
      <span class="eyebrow">ERA COMBAT LOG</span>
      <h2>{$t("history.title")}</h2>
      <p>{$t("history.subtitle")}</p>
    </div>
    <div class="live-chip"><i></i> {$t("history.live")}</div>
  </div>

  <div class="source-note">
    <strong>{$t("history.identity_title")}</strong>
    <span>{$t("history.identity_hint")}</span>
  </div>

  <div class="toolbar" role="group" aria-label={$t("history.filter")}>
    {#each ["all", "incoming", "outgoing", "death"] as key}
      <button class:active={filter === key} onclick={() => (filter = key as Filter)}>
        {$t(`history.filter_${key}` as never)}
      </button>
    {/each}
    <span>{visible.length} {$t("history.events")}</span>
  </div>

  {#if loading}
    <div class="empty">{$t("history.loading")}</div>
  {:else if error}
    <div class="empty danger">{$t("history.error")}</div>
  {:else if visible.length === 0}
    <div class="empty">
      <div class="empty-mark">⚔</div>
      <strong>{$t("history.empty")}</strong>
      <span>{$t("history.empty_hint")}</span>
    </div>
  {:else}
    <div class="event-list">
      {#each visible as event (event.id)}
        <article class:outgoing={event.direction === "outgoing"} class:death={event.direction === "death"}>
          <div class="event-sigil">{event.direction === "outgoing" ? "↗" : event.direction === "death" ? "✕" : "↙"}</div>
          <div class="event-main">
            <div class="event-top">
              <strong>{eventLabel(event)}</strong>
              <time>{formatTime(event.timestampMs)}</time>
            </div>
            <div class="identity">
              <span>
                <small>{$t("history.player")}</small>
                <b>{event.opponentName ?? $t("history.unknown")}</b>
              </span>
              <span>
                <small>{$t("history.species")}</small>
                <b>{event.opponentSpecies ?? $t("history.unknown")}</b>
              </span>
              <span>
                <small>{$t("history.your_species")}</small>
                <b>{event.selfSpecies ?? $t("history.unknown")}</b>
              </span>
              <span>
                <small>{$t("history.damage")}</small>
                <b>{damage(event.damage)}</b>
              </span>
            </div>
            <div class="meta">
              <span>{event.serverName ?? "ERAGAMING"}</span>
              <span class:estimated={event.source === "health-delta"}>
                {event.source === "health-delta" ? $t("history.estimated") : $t("history.verified")}
              </span>
            </div>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .history-shell { max-width: 1050px; margin: 0 auto; padding: 34px 38px 52px; }
  .intro { display: flex; align-items: flex-start; justify-content: space-between; gap: 24px; margin-bottom: 20px; }
  .eyebrow { color: var(--color-accent); font: 700 9px Consolas, monospace; letter-spacing: .2em; }
  h2 { margin: 8px 0 7px; font-size: 25px; letter-spacing: .02em; }
  p { max-width: 720px; margin: 0; color: var(--color-muted); font-size: 13px; line-height: 1.55; }
  .live-chip { display: flex; align-items: center; gap: 8px; border: 1px solid rgba(69,245,162,.35); padding: 8px 11px; color: var(--color-success); font: 700 10px Consolas, monospace; letter-spacing: .08em; }
  .live-chip i { width: 6px; height: 6px; border-radius: 50%; background: var(--color-success); box-shadow: 0 0 10px var(--color-success); }
  .source-note { display: grid; grid-template-columns: max-content 1fr; gap: 15px; padding: 13px 15px; border: 1px solid rgba(255,200,87,.25); border-left: 3px solid var(--color-warning); background: rgba(255,200,87,.045); font-size: 12px; }
  .source-note strong { color: var(--color-warning); }
  .source-note span { color: #a9b8c8; }
  .toolbar { display: flex; align-items: center; gap: 7px; margin: 20px 0 12px; }
  .toolbar button { cursor: pointer; border: 1px solid var(--color-border); background: #07101d; color: var(--color-muted); padding: 7px 12px; font-size: 11px; }
  .toolbar button.active { border-color: var(--color-accent); background: rgba(53,242,255,.08); color: var(--color-accent); }
  .toolbar > span { margin-left: auto; color: #526a82; font: 10px Consolas, monospace; text-transform: uppercase; }
  .event-list { display: grid; gap: 9px; }
  article { display: grid; grid-template-columns: 46px 1fr; border: 1px solid rgba(255,86,120,.25); background: linear-gradient(90deg, rgba(255,86,120,.055), rgba(8,17,31,.9) 34%); }
  article.outgoing { border-color: rgba(255,200,87,.25); background: linear-gradient(90deg, rgba(255,200,87,.05), rgba(8,17,31,.9) 34%); }
  article.death { border-color: rgba(168,85,247,.3); background: linear-gradient(90deg, rgba(168,85,247,.06), rgba(8,17,31,.9) 34%); }
  .event-sigil { display: grid; place-items: center; border-right: 1px solid rgba(255,255,255,.07); color: var(--color-danger); font: 700 22px Consolas, monospace; }
  article.outgoing .event-sigil { color: var(--color-warning); }
  article.death .event-sigil { color: var(--color-accent-2); }
  .event-main { padding: 12px 15px 10px; min-width: 0; }
  .event-top { display: flex; align-items: baseline; justify-content: space-between; gap: 14px; margin-bottom: 11px; }
  .event-top strong { font-size: 14px; }
  time { color: #526a82; font: 10px Consolas, monospace; white-space: nowrap; }
  .identity { display: grid; grid-template-columns: 1.2fr 1fr 1fr .55fr; gap: 15px; }
  .identity span { min-width: 0; }
  small { display: block; margin-bottom: 3px; color: #526a82; font: 9px Consolas, monospace; letter-spacing: .09em; text-transform: uppercase; }
  b { display: block; overflow: hidden; color: #c7d8e8; font-size: 12px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }
  .meta { display: flex; gap: 10px; margin-top: 10px; color: #49627a; font: 9px Consolas, monospace; text-transform: uppercase; }
  .meta span + span { color: var(--color-success); }
  .meta span.estimated { color: var(--color-warning); }
  .empty { display: grid; justify-items: center; gap: 7px; padding: 75px 20px; border: 1px dashed var(--color-border); background: rgba(8,17,31,.55); color: var(--color-muted); font-size: 12px; text-align: center; }
  .empty strong { color: #b9cadd; font-size: 14px; }
  .empty-mark { color: #365670; font-size: 35px; }
  .empty.danger { color: var(--color-danger); }
  @media (max-width: 900px) { .identity { grid-template-columns: 1fr 1fr; } .source-note { grid-template-columns: 1fr; gap: 5px; } }
</style>
