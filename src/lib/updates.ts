import { check } from "@tauri-apps/plugin-updater";
import { createUpdateController } from "./update-controller";

export const RELEASES_URL = "https://github.com/thienvyma/hubisle/releases";
// Endpoint and signing key stay in tauri.conf.json; no frontend override.
export const updates = createUpdateController(() => check({ timeout: 15_000 }));
export const updateState = updates.state;
