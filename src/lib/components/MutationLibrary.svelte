<script lang="ts">
  import { locale, t } from "$lib/i18n";
  import { MUTATION_CATALOG, searchMutations } from "$lib/mutations";

  let query = $state("");
  const results = $derived(searchMutations(query, $locale));
</script>

<details class="library">
  <summary>
    <span>
      <strong>{$t("mutations.library_title")}</strong>
      <small>{$t("mutations.library_count", { count: MUTATION_CATALOG.length })}</small>
    </span>
    <span class="chevron">⌄</span>
  </summary>

  <div class="library-body">
    <p class="intro">{$t("mutations.library_hint")}</p>
    <input
      bind:value={query}
      type="search"
      placeholder={$t("mutations.search_placeholder")}
      aria-label={$t("mutations.search_placeholder")}
    />
    {#if results.length}
      <div class="result-grid">
        {#each results as mutation (mutation.nameEn)}
          <article>
            <h4>{mutation.nameEn}</h4>
            <p>{$locale === "vi" ? mutation.descriptionVi : mutation.descriptionEn}</p>
          </article>
        {/each}
      </div>
    {:else}
      <p class="empty">{$t("mutations.search_empty")}</p>
    {/if}
    <p class="source">{$t("mutations.source_note")}</p>
  </div>
</details>

<style>
  .library {
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-panel);
  }
  summary {
    display: flex;
    cursor: pointer;
    list-style: none;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 16px;
  }
  summary::-webkit-details-marker { display: none; }
  summary span:first-child { display: grid; gap: 3px; }
  strong { color: var(--color-accent); font-size: 13px; }
  small, .intro, .source, .empty { color: var(--color-muted); font-size: 11px; }
  .chevron { color: var(--color-accent); font: 18px Consolas, monospace; transition: transform 150ms ease; }
  details[open] .chevron { transform: rotate(180deg); }
  .library-body { border-top: 1px solid var(--color-border); padding: 14px 16px 16px; }
  .intro { margin: 0 0 10px; line-height: 1.5; }
  input {
    width: 100%;
    border: 1px solid #1d3a51;
    border-radius: 3px;
    outline: none;
    background: rgba(3, 12, 23, 0.72);
    color: var(--color-text);
    padding: 9px 11px;
    font: 12px Consolas, monospace;
  }
  input:focus { border-color: rgba(53, 242, 255, 0.75); }
  .result-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(225px, 1fr));
    gap: 8px;
    max-height: 420px;
    margin-top: 12px;
    overflow-y: auto;
    padding-right: 3px;
  }
  article { border: 1px solid rgba(53, 242, 255, 0.16); background: rgba(7, 16, 29, 0.62); padding: 10px 11px; }
  h4 { margin: 0; color: var(--color-accent); font: 700 11px Consolas, monospace; letter-spacing: .035em; }
  article p { margin: 5px 0 0; color: var(--color-muted); font-size: 11px; line-height: 1.55; }
  .empty { margin: 16px 0; }
  .source { margin: 12px 0 0; border-left: 2px solid rgba(53, 242, 255, 0.55); padding-left: 8px; line-height: 1.45; }
</style>
