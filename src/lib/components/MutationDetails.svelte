<script lang="ts">
  import { locale } from "$lib/i18n";
  import { mutationDisplay } from "$lib/mutations";

  let {
    names,
    compact = false,
  }: {
    names: string[];
    compact?: boolean;
  } = $props();

  const mutations = $derived(names.map((name) => mutationDisplay(name, $locale)));
</script>

{#if compact}
  <div class="compact-list">
    {#each mutations as mutation (mutation.name)}
      <details>
        <summary>{mutation.name}</summary>
        <p>{mutation.description}</p>
      </details>
    {/each}
  </div>
{:else}
  <div class="mutation-grid">
    {#each mutations as mutation (mutation.name)}
      <article class:unknown={!mutation.known}>
        <h4>{mutation.name}</h4>
        <p>{mutation.description}</p>
      </article>
    {/each}
  </div>
{/if}

<style>
  .mutation-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 8px;
  }
  article {
    border: 1px solid rgba(53, 242, 255, 0.18);
    border-left: 2px solid rgba(53, 242, 255, 0.72);
    border-radius: 3px;
    background: rgba(7, 16, 29, 0.64);
    padding: 10px 11px;
  }
  article.unknown { border-left-color: #7c8794; }
  h4, summary {
    color: var(--color-accent);
    font-family: Consolas, monospace;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.035em;
  }
  p {
    margin: 5px 0 0;
    color: var(--color-muted);
    font-size: 11px;
    line-height: 1.55;
  }
  .compact-list { display: grid; gap: 5px; margin-top: 10px; }
  details {
    border: 1px solid #1d3a51;
    background: rgba(3, 12, 23, 0.52);
    padding: 6px 8px;
  }
  summary { cursor: pointer; color: #9db2c7; }
</style>
