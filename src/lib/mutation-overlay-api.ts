import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface NormalizedRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface MutationOverlaySettings {
  enabled: boolean;
  auto_detect: boolean;
  rect: NormalizedRect;
  confidence_threshold: number;
}

export type MutationOverlayState =
  | "disabled"
  | "waiting-game"
  | "recognizing"
  | "recognized"
  | "manual"
  | "calibrating"
  | "needs-calibration"
  | "capture-unavailable"
  | "ocr-unavailable";

export interface MutationOverlayStatus {
  state: MutationOverlayState;
  nameEn: string | null;
  confidence: number | null;
  message: string | null;
}

export interface MutationOverlayPayload {
  nameEn: string;
  descriptionVi: string;
  confidence: number | null;
  source: "auto" | "manual" | "preview";
}

export const DEFAULT_MUTATION_OVERLAY_SETTINGS: MutationOverlaySettings = {
  enabled: false,
  auto_detect: true,
  rect: { x: 0.61, y: 0.28, w: 0.28, h: 0.22 },
  confidence_threshold: 0.82,
};

export const mutationOverlayStatus = () =>
  invoke<MutationOverlayStatus>("mutation_overlay_status");

export const mutationOverlaySetManual = (nameEn: string) =>
  invoke<MutationOverlayStatus>("mutation_overlay_set_manual", { nameEn });

export const mutationOverlayClearManual = () =>
  invoke<MutationOverlayStatus>("mutation_overlay_clear_manual");

export const mutationOverlayBeginCalibration = () =>
  invoke<MutationOverlayStatus>("mutation_overlay_begin_calibration");

export const mutationOverlaySaveCalibration = () =>
  invoke<MutationOverlayStatus>("mutation_overlay_save_calibration");

export const mutationOverlayCancelCalibration = () =>
  invoke<MutationOverlayStatus>("mutation_overlay_cancel_calibration");

export const mutationOverlayPreview = (nameEn: string) =>
  invoke<MutationOverlayStatus>("mutation_overlay_preview", { nameEn });

export const onMutationOverlayState = (
  cb: (status: MutationOverlayStatus) => void,
): Promise<UnlistenFn> =>
  listen<MutationOverlayStatus>("mutation-overlay://state", (event) => cb(event.payload));

export const onMutationOverlayPayload = (
  cb: (payload: MutationOverlayPayload) => void,
): Promise<UnlistenFn> =>
  listen<MutationOverlayPayload>("mutation-overlay://payload", (event) => cb(event.payload));
