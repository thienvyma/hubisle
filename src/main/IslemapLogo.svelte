<script lang="ts">
  let { size = 44, compact = false }: { size?: number; compact?: boolean } = $props();
</script>

<div class="islemap-brand" class:compact style={`--logo-size: ${size}px`} aria-label="islemap-thienvyma">
  <svg class="islemap-mark" viewBox="0 0 64 64" aria-hidden="true">
    <defs>
      <linearGradient id="islemap-gradient" x1="9" y1="7" x2="55" y2="57" gradientUnits="userSpaceOnUse">
        <stop stop-color="#35f2ff" />
        <stop offset="1" stop-color="#a855f7" />
      </linearGradient>
    </defs>
    <path class="outer" d="M32 4 55 17.2v29.6L32 60 9 46.8V17.2L32 4Z" />
    <circle class="ring" cx="32" cy="32" r="15.5" />
    <path class="arrow" d="m39.8 19.2-5.2 14.4-14.4 5.2 5.2-14.4 14.4-5.2Z" />
    <circle cx="32" cy="32" r="2.2" />
  </svg>
  {#if !compact}
    <div class="wordmark">
      <strong>ISLEMAP</strong>
      <small>THIENVYMA</small>
    </div>
  {/if}
</div>

<style>
  .islemap-brand {
    display: flex;
    align-items: center;
    gap: 11px;
    min-width: 0;
  }
  .islemap-mark {
    width: var(--logo-size);
    height: var(--logo-size);
    flex: 0 0 auto;
    overflow: visible;
    filter: drop-shadow(0 0 8px rgba(53, 242, 255, 0.28));
  }
  .outer { fill: rgba(5, 12, 24, 0.92); stroke: url(#islemap-gradient); stroke-width: 2.2; }
  .ring { fill: none; stroke: url(#islemap-gradient); stroke-width: 1.7; opacity: .9; }
  .arrow { fill: #35f2ff; stroke: #a855f7; stroke-width: 1.7; stroke-linejoin: round; }
  circle:last-child { fill: #f4f7ff; }
  .wordmark { display: grid; gap: 2px; min-width: 0; }
  strong { color: #f4f7ff; font-size: 14px; letter-spacing: .15em; line-height: 1; white-space: nowrap; }
  small { color: #35f2ff; font-size: 8px; font-weight: 700; letter-spacing: .27em; line-height: 1; white-space: nowrap; }
  .compact { justify-content: center; }
</style>
