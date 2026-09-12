<script lang="ts">
  // Bottom-right credit bar: author-owned links (system browser).
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { t } from "$lib/i18n";
  import { updates, updateState } from "$lib/updates";

  const FACEBOOK_URL = "https://www.facebook.com/thienvyma";
  const GITHUB_URL = "https://github.com/thienvyma/hubisle";
</script>

<footer
  class="flex shrink-0 items-center justify-between gap-3"
>
  <span><b>SYSTEM READY</b> // v{__APP_VERSION__} · {$t("footer.reload_hint")}</span>
  <div class="flex items-center gap-3">
    <button
      disabled={["checking", "downloading", "installing"].includes($updateState.status)}
      onclick={() => void updates.checkNow(true)}
    >{$updateState.status === "checking" ? $t("update.checking") : $t("update.check")}</button>
    <span>islemap-thienvyma · {$t("footer.developed_by")} <em>Huỳnh Vỹ</em></span>
    <button
      class="cursor-pointer underline-offset-2 hover:underline"
      onclick={() => void openUrl(FACEBOOK_URL)}
    >
      Facebook
    </button>
    <button
      class="cursor-pointer underline-offset-2 hover:underline"
      onclick={() => void openUrl(GITHUB_URL)}
    >
      GitHub
    </button>
  </div>
</footer>

<style>
  footer {
    min-height: 29px;
    border-top: 1px solid #14283b;
    padding: 0 14px;
    background: rgba(4, 11, 21, .96);
    color: #536b82;
    font: 8px Consolas, monospace;
    letter-spacing: .06em;
  }
  b { color: var(--color-success); font-weight: 600; }
  em { color: #91a7bb; font-style: normal; }
  button { color: #6b8ba3; cursor: pointer; }
</style>
