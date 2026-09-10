<script lang="ts">
  let { size = 44, compact = false }: { size?: number; compact?: boolean } = $props();
</script>

<div class="pulse-brand" class:compact style={`--logo-size: ${size}px`} aria-label="Isle Pulse">
  <svg class="pulse-mark" viewBox="0 0 64 64" aria-hidden="true">
    <defs>
      <linearGradient id="pulse-gradient" x1="8" y1="7" x2="56" y2="57" gradientUnits="userSpaceOnUse">
        <stop stop-color="#35f2ff" />
        <stop offset="1" stop-color="#a855f7" />
      </linearGradient>
    </defs>
    <path class="outer" d="M32 3 55.4 16.5v27L32 57 8.6 43.5v-27L32 3Z" />
    <path class="scan" d="M14 34h9l4-12 7.5 24L40 29l3 5h7" />
    <path class="needle" d="m32 10 4.3 8.8L32 22l-4.3-3.2L32 10Z" />
    <circle cx="32" cy="34" r="2.8" />
  </svg>
  {#if !compact}
    <div class="wordmark">
      <strong>ISLE<span>//</span>PULSE</strong>
      <small>LIVE SURVIVAL SYSTEM</small>
    </div>
  {/if}
</div>

<style>
  .pulse-brand {
    display: flex;
    align-items: center;
    gap: 11px;
    min-width: 0;
  }
  .pulse-mark {
    width: var(--logo-size);
    height: var(--logo-size);
    flex: 0 0 auto;
    overflow: visible;
    filter: drop-shadow(0 0 8px rgba(53, 242, 255, 0.28));
  }
  .outer {
    fill: rgba(5, 12, 24, 0.92);
    stroke: url(#pulse-gradient);
    stroke-width: 2.2;
  }
  .scan {
    fill: none;
    stroke: url(#pulse-gradient);
    stroke-width: 3;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .needle,
  circle {
    fill: #35f2ff;
  }
  .wordmark {
    display: grid;
    gap: 1px;
    min-width: 0;
  }
  strong {
    color: #f4f7ff;
    font-size: 15px;
    letter-spacing: 0.12em;
    line-height: 1;
    white-space: nowrap;
  }
  strong span {
    color: #35f2ff;
  }
  small {
    color: #70809d;
    font-size: 7px;
    font-weight: 700;
    letter-spacing: 0.23em;
    white-space: nowrap;
  }
  .compact {
    justify-content: center;
  }
</style>
