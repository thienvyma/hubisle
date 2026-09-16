<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import type { MutationOverlayPayload } from "$lib/mutation-overlay-api";

  let payload = $state<MutationOverlayPayload | null>(null);
  let calibrating = $state(false);
  let listeners: UnlistenFn[] = [];

  onMount(() => {
    void Promise.all([
      listen<MutationOverlayPayload>("mutation-overlay://payload", (event) => {
        payload = event.payload;
      }),
      listen<boolean>("mutation-overlay://calibration", (event) => {
        calibrating = event.payload;
      }),
      listen("mutation-overlay://clear", () => {
        payload = null;
      }),
    ]).then((ready) => {
      listeners = ready;
    });

    return () => {
      for (const unlisten of listeners) unlisten();
      listeners = [];
    };
  });
</script>

<div class:calibrating class="surface">
  {#if calibrating}
    <div class="calibration-label">MUTATION NAME + DESCRIPTION</div>
    <div class="calibration-hint">Kéo và đổi kích thước khung này cho khớp vùng mô tả trong The Isle</div>
  {:else if payload}
    <div class="mask" data-source={payload.source}>
      <div class="name">{payload.nameEn}</div>
      <div class="description">{payload.descriptionVi}</div>
    </div>
  {/if}
</div>

<style>
  :global(html),
  :global(body),
  :global(#app) {
    width: 100%;
    height: 100%;
    margin: 0;
    overflow: hidden;
    background: transparent !important;
  }

  :global(*) { box-sizing: border-box; }

  .surface {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: flex-start;
    justify-content: flex-start;
    pointer-events: none;
    font-family: "Segoe UI", Arial, sans-serif;
  }

  .mask {
    width: 100%;
    min-height: 100%;
    padding: 10px 12px 12px;
    background: rgba(8, 10, 12, 0.92);
    color: #f3f5f6;
    text-shadow: 0 1px 1px rgba(0, 0, 0, 0.8);
  }

  .name {
    margin-bottom: 6px;
    font-size: clamp(13px, 1.35vw, 21px);
    font-weight: 600;
    line-height: 1.16;
    color: rgba(245, 246, 247, 0.98);
  }

  .description {
    font-size: clamp(12px, 1.05vw, 18px);
    font-weight: 400;
    line-height: 1.34;
    color: rgba(231, 234, 236, 0.96);
  }

  .surface.calibrating {
    display: block;
    border: 2px solid #35f2ff;
    background: rgba(5, 20, 25, 0.25);
  }

  .calibration-label {
    display: inline-block;
    margin: 8px;
    padding: 4px 6px;
    background: rgba(3, 12, 23, 0.9);
    color: #35f2ff;
    font: 10px Consolas, monospace;
    letter-spacing: 0.08em;
  }

  .calibration-hint {
    position: absolute;
    left: 8px;
    right: 8px;
    bottom: 8px;
    padding: 5px 7px;
    background: rgba(3, 12, 23, 0.9);
    color: #d4e5ee;
    font: 10px/1.35 "Segoe UI", Arial, sans-serif;
  }
</style>
