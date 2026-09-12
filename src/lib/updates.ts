import { check } from "@tauri-apps/plugin-updater";
import { info, warn } from "@tauri-apps/plugin-log";
import { createUpdateController } from "./update-controller";

export const RELEASES_URL = "https://github.com/thienvyma/hubisle/releases";
// Endpoint and signing key stay in tauri.conf.json; no frontend override.
// Keep a small diagnostic in the app log because the native plugin otherwise
// turns every network/signature failure into the same silent banner state.
export const updates = createUpdateController(async () => {
  try {
    const result = await check({ timeout: 15_000 });
    void info(result ? `updater: available ${result.version}` : "updater: current");
    return result;
  } catch (reason) {
    void warn(`updater: check failed: ${String(reason)}`);
    throw reason;
  }
});
export const updateState = updates.state;
