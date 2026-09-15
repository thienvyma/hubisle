import { invoke } from "@tauri-apps/api/core";

export type MutationLocaleState =
  | "game-not-found"
  | "not-installed"
  | "installed"
  | "update-available"
  | "incompatible"
  | "corrupt"
  | "game-running";

export interface MutationLocaleStatus {
  state: MutationLocaleState;
  gamePath: string | null;
  packVersion: string;
  matched: number;
  total: number;
  message: string | null;
}

export const mutationLocaleStatus = () =>
  invoke<MutationLocaleStatus>("mutation_locale_status");

export const mutationLocaleInstall = () =>
  invoke<MutationLocaleStatus>("mutation_locale_install");

export const mutationLocaleUninstall = () =>
  invoke<MutationLocaleStatus>("mutation_locale_uninstall");
