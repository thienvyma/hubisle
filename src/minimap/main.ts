// Minimap overlay entry. Deliberately tiny: no Skeleton, no Leaflet, no
// framework — this webview runs beside the game for hours. Rendering is
// event-driven only (zero idle CPU: no rAF loop, no animations, no timers).
// Draw synchronously on every position packet. WebView2 can suspend animation
// frames when a fullscreen game occludes this window, even though Tauri events
// are still delivered; relying on rAF made the heading appear frozen until the
// user Alt-Tabbed back out of the game.

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { error } from "@tauri-apps/plugin-log";
import { installGlobalErrorLog } from "../lib/errlog";
import { acceptHeading, emptyHeading, headingSourceLabel, type HeadingUpdate } from "../lib/heading";
import { ANIMAL_GLYPHS, waypointGlyph } from "../lib/theme";
import {
  PANEL_H,
  PANEL_ROW_H,
  QUEST_HEADER_H,
  QUEST_PAD_H,
  QUEST_ROW_H,
  render,
  type CombatAlert,
  type DinoBars,
  type MinimapState,
  type PoiDot,
  type QuestRow,
} from "./render";

installGlobalErrorLog("minimap");

// Local minimal types — this bundle stays free of the main window's modules.
interface PositionUpdate extends HeadingUpdate {
  xCm: number;
  yCm: number;
  px: number;
  py: number;
  headingDeg: number | null;
  headingSource: "local-camera" | "provider-camera" | "movement" | null;
  compassKey: string | null;
}
interface PoiLayer {
  key: string;
  kind: string;
  items: { label: string; px: number; py: number; xCm: number; yCm: number }[];
}
interface ProviderState {
  provider: "isle-pilot" | "era" | "titan" | null;
  status: string;
  dataStale: boolean;
}
interface ProviderStatBar {
  current?: number | null;
  max?: number | null;
  percent: number;
}
interface ProviderSnapshot {
  player: {
    growthPct: number | null;
    health: ProviderStatBar | null;
    hunger: ProviderStatBar | null;
    thirst: ProviderStatBar | null;
    stamina: ProviderStatBar | null;
    primeQuests: QuestRow[];
  } | null;
  friends: {
    slot: number | null;
    name: string;
    dinoName: string | null;
    online: boolean;
    positionCm: [number, number, number] | null;
    positionPx: [number, number] | null;
  }[];
}
interface CombatEvent {
  id: string;
  direction: "incoming" | "outgoing" | "death";
  opponentName: string | null;
  opponentSpecies: string | null;
  damage: number | null;
  source: string;
}
type Settings = Record<string, any>;

const LAYER_COLORS: Record<string, string> = {
  water: "#35bdf2",
  saltlick: "#ffc857",
  mudwallow: "#9c7b4f",
  sanctuary: "#a855f7",
  migration: "#45f5a2",
  food: "#ff5678",
  animal: "#ff62bc",
};

// Compass letters + strings per language (kept inline: no i18n bundle here).
const STRINGS = {
  vi: {
    letters: ["Bắc", "Đông", "Nam", "Tây"] as [string, string, string, string],
    hint: "Đang chờ vị trí từ server…",
    retrying: "Đang kết nối lại server…",
    stale: "Dữ liệu cũ · đang kết nối lại",
    unknown: "Chưa rõ hướng",
    combat: {
      health: "Mất máu",
      incoming: "Bị tấn công",
      outgoing: "Đã tấn công",
      death: "Đã chết",
      unknown: "không rõ đối thủ",
    },
    dirs: {
      "dir.N": "Bắc", "dir.NE": "Đông Bắc", "dir.E": "Đông", "dir.SE": "Đông Nam",
      "dir.S": "Nam", "dir.SW": "Tây Nam", "dir.W": "Tây", "dir.NW": "Tây Bắc",
    } as Record<string, string>,
  },
  en: {
    letters: ["N", "E", "S", "W"] as [string, string, string, string],
    hint: "Waiting for the live server position…",
    retrying: "Reconnecting to the server…",
    stale: "Last data · reconnecting",
    unknown: "Heading unknown",
    combat: {
      health: "Health lost",
      incoming: "Attacked",
      outgoing: "You attacked",
      death: "Died",
      unknown: "unknown opponent",
    },
    dirs: {
      "dir.N": "N", "dir.NE": "NE", "dir.E": "E", "dir.SE": "SE",
      "dir.S": "S", "dir.SW": "SW", "dir.W": "W", "dir.NW": "NW",
    } as Record<string, string>,
  },
};

const canvas = document.getElementById("minimap") as HTMLCanvasElement;

let allPois: PoiDot[] = [];
let poiLayers: PoiLayer[] = [];
let settings: Settings = {};
let providerActive = false;
let providerRetrying = false;
let providerStale = false;

const state: MinimapState = {
  position: null,
  headingDeg: null,
  headingSourceLabel: "",
  trailPx: [],
  pois: [],
  friends: [],
  waypoints: [],
  nearestWaypoint: null,
  basemap: null,
  freshwater: null,
  miniScale: 1,
  pxPerM: 0.7,
  sizePx: 260,
  radiusM: 600,
  opacity: 0.85,
  showTrail: true,
  showWaypoints: true,
  showFreshwater: true,
  panelH: 0,
  dino: null,
  questsH: 0,
  quests: [],
  questLang: "vi",
  compassLetters: STRINGS.vi.letters,
  hintText: STRINGS.vi.hint,
  headingLabel: "",
  headingUnknown: STRINGS.vi.unknown,
  combatAlerts: [],
  staleText: "",
};

let lastHeadingKey: string | null = null;
let lastHeadingDeg: number | null = null;
let lastHeadingSource: PositionUpdate["headingSource"] = null;
let currentHeading = emptyHeading();

function applyHeading(p: HeadingUpdate) {
  currentHeading = acceptHeading(currentHeading, p);
  lastHeadingKey = currentHeading.compassKey;
  lastHeadingDeg = currentHeading.headingDeg;
  lastHeadingSource = currentHeading.headingSource;
  state.headingDeg = currentHeading.headingDeg;
  refreshHeadingLabel(settings.language === "en" ? "en" : "vi");
}

function applySettings(s: Settings) {
  settings = s;
  const mm = s.minimap ?? {};
  state.sizePx = Number(mm.size_px ?? 260);
  state.radiusM = Number(mm.radius_m ?? 600);
  state.opacity = Number(mm.opacity ?? 0.85);
  state.showTrail = Boolean(mm.show_trail ?? true);
  state.showWaypoints = Boolean(mm.show_waypoints ?? true);
  state.showFreshwater = Boolean((s.layers ?? {}).freshwater ?? true);
  recomputePanelH();
  const lang = (s.language === "en" ? "en" : "vi") as keyof typeof STRINGS;
  state.questLang = lang;
  recomputeQuestsH();
  state.compassLetters = STRINGS[lang].letters;
  state.hintText = providerRetrying ? STRINGS[lang].retrying : STRINGS[lang].hint;
  state.staleText = providerStale ? STRINGS[lang].stale : "";
  state.headingUnknown = STRINGS[lang].unknown;
  refreshHeadingLabel(lang);
  refreshPoiFilter();
}

/** Window height for the stats strip is Rust's job; this mirrors its formula. */
function recomputePanelH() {
  const ip = settings.islepilot ?? {};
  state.panelH =
    providerActive && (ip.show_overlay_panel ?? true)
      ? PANEL_H + (state.dino?.stamina ? PANEL_ROW_H : 0)
      : 0;
}

/** Window height for the quest card is Rust's job (minimap.rs quests_h reads
 * the same count from the poller); this only has to agree on the formula. */
function recomputeQuestsH() {
  const ip = settings.islepilot ?? {};
  state.questsH =
    providerActive && (ip.show_quests_panel ?? false) && state.quests.length > 0
      ? QUEST_HEADER_H + state.quests.length * QUEST_ROW_H + QUEST_PAD_H
      : 0;
}

function clearProviderDisplay() {
  state.dino = null;
  state.quests = [];
  state.friends = [];
  recomputePanelH();
  recomputeQuestsH();
}

function applyProviderState(value: ProviderState) {
  providerRetrying = value.status === "temporary-error";
  providerStale = providerRetrying && value.dataStale;
  const lang = settings.language === "en" ? "en" : "vi";
  state.hintText = providerRetrying ? STRINGS[lang].retrying : STRINGS[lang].hint;
  state.staleText = providerStale ? STRINGS[lang].stale : "";
  providerActive =
    value.provider !== null &&
    ["authenticated-online", "authenticated-offline", "temporary-error"].includes(value.status);
  if (value.provider === null || (providerRetrying && !providerStale)) clearProviderDisplay();
  recomputePanelH();
  recomputeQuestsH();
}

function applyProviderSnapshot(snapshot: ProviderSnapshot | null) {
  state.friends = (snapshot?.friends ?? []).flatMap((friend) => {
    if (!friend.online || !friend.positionCm || !friend.positionPx) return [];
    return [{
      slot: friend.slot,
      name: friend.name,
      dinoName: friend.dinoName,
      xCm: friend.positionCm[0],
      yCm: friend.positionCm[1],
      px: friend.positionPx[0],
      py: friend.positionPx[1],
    }];
  });
  const player = snapshot?.player;
  if (!player) {
    state.dino = null;
    state.quests = [];
    recomputePanelH();
    recomputeQuestsH();
    return;
  }
  const toBar = (stat: ProviderStatBar | null): DinoBars["hp"] => ({
    current: stat?.current ?? null,
    max: stat?.max ?? null,
    percent: stat?.percent ?? 0,
  });
  state.dino = {
    hp: toBar(player.health),
    hunger: toBar(player.hunger),
    thirst: toBar(player.thirst),
    stamina: player.stamina ? toBar(player.stamina) : null,
    growthPct: player.growthPct,
  };
  state.quests = player.primeQuests ?? [];
  recomputePanelH();
  recomputeQuestsH();
}

function refreshHeadingLabel(lang: keyof typeof STRINGS) {
  state.headingSourceLabel = headingSourceLabel(lastHeadingSource, lang);
  const estimate = lastHeadingSource === "movement" ? "≈ " : "";
  state.headingLabel =
    lastHeadingKey && lastHeadingDeg !== null
      ? `${estimate}${STRINGS[lang].dirs[lastHeadingKey] ?? ""} ${Math.round(lastHeadingDeg)}°`
      : "";
}

function combatAlert(event: CombatEvent): CombatAlert {
  const lang = settings.language === "en" ? "en" : "vi";
  const strings = STRINGS[lang].combat;
  const action =
    event.source === "health-delta"
      ? strings.health
      : event.direction === "outgoing"
        ? strings.outgoing
        : event.direction === "death"
          ? strings.death
          : strings.incoming;
  const identity = [event.opponentName, event.opponentSpecies].filter(Boolean).join(" · ");
  const amount = event.damage === null ? "" : ` −${Math.round(event.damage * 10) / 10}%`;
  return {
    id: event.id,
    text: `${action}${amount} · ${identity || strings.unknown}`,
    tone: event.source === "health-delta" ? "estimated" : event.direction,
  };
}

function showCombatEvent(event: CombatEvent) {
  const alert = combatAlert(event);
  state.combatAlerts = [
    alert,
    ...state.combatAlerts.filter((item) => item.id !== alert.id),
  ].slice(0, 3);
  draw();
  window.setTimeout(() => {
    state.combatAlerts = state.combatAlerts.filter((item) => item.id !== alert.id);
    draw();
  }, 15_000);
}

function refreshPoiFilter() {
  const visible = settings.layers ?? {};
  state.pois = allPois.filter((p) => visible[(p as any).layerKey] ?? true);
}

function flattenPois() {
  allPois = [];
  for (const layer of poiLayers) {
    if (layer.kind !== "point") continue; // zones are full-map only
    const color = LAYER_COLORS[layer.key] ?? "#35f2ff";
    for (const item of layer.items) {
      allPois.push({
        xCm: item.xCm,
        yCm: item.yCm,
        px: item.px,
        py: item.py,
        color,
        // Animals draw as their species glyph instead of a dot.
        glyph: layer.key === "animal" ? ANIMAL_GLYPHS[item.label] : undefined,
        // carried for the visibility filter
        ...( { layerKey: layer.key } as object ),
      });
    }
  }
  refreshPoiFilter();
}

const draw = () => render(canvas, state);

let imageWidthPx = 7800;
// Which basemap imagery this webview currently renders — compared against
// settings broadcasts to reload only on a real switch.
let currentSource = "vulnona";
// Fresh-water overlay descriptor from get_map_info (bounds already in the
// ACTIVE calibration's px space); null when the file is not on disk yet.
let overlayInfo: { url: string; boundsPx: [number, number, number, number] } | null = null;

type MapInfoPayload = {
  imageWidthPx: number;
  pxPerMX: number;
  source: string;
  overlays?: { key: string; path: string; boundsPx: [number, number, number, number] }[];
};

function applyMapInfo(info: MapInfoPayload) {
  state.pxPerM = info.pxPerMX;
  imageWidthPx = info.imageWidthPx;
  currentSource = info.source;
  overlayInfo = null;
  for (const ov of info.overlays ?? []) {
    if (ov.key === "freshwater") {
      overlayInfo = { url: convertFileSrc(ov.path), boundsPx: ov.boundsPx };
    }
  }
}

/// (Re)load basemap + POIs. Called at init AND whenever the first-run /
/// re-download fetch finishes — the data may not exist yet when this webview
/// first starts, and it must pick it up without an app restart.
async function loadData() {
  try {
    poiLayers = await invoke<PoiLayer[]>("get_pois_render");
    flattenPois();
  } catch {
    // POI data missing (first run): map still works without dots.
  }
  try {
    const paths = await invoke<{ minimap: string; minimapDecodeWidth: number | null }>(
      "get_basemap_paths",
    );
    const resp = await fetch(convertFileSrc(paths.minimap));
    if (resp.ok) {
      // The islemaps PNGs decode to ~25 MB; the hint downscales them at
      // decode so the always-resident bitmap stays small. miniScale
      // normalises by bitmap width, so a downscaled decode needs no other
      // change anywhere.
      const blob = await resp.blob();
      const bitmap = await createImageBitmap(
        blob,
        paths.minimapDecodeWidth
          ? { resizeWidth: paths.minimapDecodeWidth, resizeQuality: "high" }
          : {},
      );
      state.basemap?.close(); // release the old pixels promptly
      state.basemap = bitmap;
      state.miniScale = state.basemap.width / imageWidthPx;
    }
  } catch {
    // Missing basemap: the disc just stays unfilled until data arrives.
  }
  try {
    if (overlayInfo) {
      const resp = await fetch(overlayInfo.url);
      if (resp.ok) {
        // Same downscale reasoning as the islemaps basemap: ~6 MB resident
        // instead of ~25 MB; the draw stretches to px bounds so resolution
        // only affects sharpness.
        const bmp = await createImageBitmap(await resp.blob(), {
          resizeWidth: 1250,
          resizeQuality: "high",
        });
        const [left, top, right, bottom] = overlayInfo.boundsPx;
        state.freshwater?.bitmap.close();
        state.freshwater = { bitmap: bmp, x: left, y: top, w: right - left, h: bottom - top };
      }
    } else if (state.freshwater) {
      state.freshwater.bitmap.close();
      state.freshwater = null;
    }
  } catch {
    // Overlay missing: the layer is simply absent.
  }
  draw();
}

/// Waypoints for the disc + the nearest-waypoint rim arrow. Both piggyback
/// on events (waypoints://changed, position updates) — no polling.
interface WaypointPx {
  id: string;
  name: string;
  /** world cm (legacy field names) */
  x: number;
  y: number;
  px: number;
  py: number;
  color: string | null;
}
let waypointsPx: WaypointPx[] = [];

async function refreshWaypoints() {
  try {
    waypointsPx = await invoke<WaypointPx[]>("list_waypoints_px");
  } catch {
    waypointsPx = [];
  }
  state.waypoints = waypointsPx.map((w) => ({
    xCm: w.x,
    yCm: w.y,
    px: w.px,
    py: w.py,
    color: w.color,
    glyph: waypointGlyph(w.name),
  }));
  await refreshNearest();
  draw();
}

async function refreshNearest() {
  try {
    const near = await invoke<{
      id: string;
      bearingDeg: number;
      distanceM: number;
    } | null>("nearest_waypoint");
    const target = near ? waypointsPx.find((w) => w.id === near.id) : undefined;
    state.nearestWaypoint = near
      ? {
          bearingDeg: near.bearingDeg,
          distanceM: near.distanceM,
          color: target?.color ?? null,
          glyph: target ? waypointGlyph(target.name) : undefined,
        }
      : null;
  } catch {
    state.nearestWaypoint = null;
  }
}

let nearestRefreshTimer: number | null = null;
function scheduleNearestRefresh() {
  if (nearestRefreshTimer !== null) return;
  nearestRefreshTimer = window.setTimeout(() => {
    nearestRefreshTimer = null;
    void refreshNearest().then(draw);
  }, 250);
}

/// Full reload after a basemap switch: new geometry, new bitmap, and a
/// defensive position/trail re-fetch (resync events also arrive; this closes
/// the one-stale-frame window in between).
async function reloadMapSource() {
  try {
    applyMapInfo(await invoke<MapInfoPayload>("get_map_info"));
  } catch {
    return; // keep rendering the old frame rather than a mismatched one
  }
  await loadData();
  try {
    const p = await invoke<PositionUpdate | null>("get_current_position");
    if (p) {
      state.position = { xCm: p.xCm, yCm: p.yCm, px: p.px, py: p.py, headingDeg: p.headingDeg };
      applyHeading(p);
    }
    const trail = await invoke<{ segmentsPx: [number, number][][] }>("get_current_trail");
    state.trailPx = trail.segmentsPx;
  } catch {
    // resync events will repaint us shortly anyway
  }
  // Waypoint px is calibration-dependent — refresh in the new frame.
  await refreshWaypoints();
  draw();
}

async function init() {
  const [initialSettings, initialProvider, initialSnapshot] = await Promise.all([
    invoke<Settings>("get_settings"),
    invoke<ProviderState>("provider_state"),
    invoke<ProviderSnapshot | null>("provider_snapshot"),
  ]);
  settings = initialSettings;
  applyProviderState(initialProvider);
  if (initialProvider.status !== "temporary-error" || initialProvider.dataStale) applyProviderSnapshot(initialSnapshot);
  applySettings(settings);

  applyMapInfo(await invoke<MapInfoPayload>("get_map_info"));

  await listen<PositionUpdate>("position://update", (e) => {
    const p = e.payload;
    state.position = {
      xCm: p.xCm,
      yCm: p.yCm,
      px: p.px,
      py: p.py,
      headingDeg: p.headingDeg,
    };
    applyHeading(p);
    draw();
    // Limit the waypoint IPC calculation while Titan is sending 20 position
    // samples per second. The player marker and camera arrow still repaint on
    // every sample.
    scheduleNearestRefresh();
  });
  await listen("position://cleared", () => {
    state.position = null;
    state.nearestWaypoint = null;
    draw();
  });
  await listen<HeadingUpdate>("heading://update", (e) => {
    applyHeading(e.payload);
    draw(); // No waypoint IPC or position/trail mutation on camera updates.
  });
  await listen("waypoints://changed", () => void refreshWaypoints());
  await listen<{ segmentsPx: [number, number][][] }>("trail://changed", (e) => {
    state.trailPx = e.payload.segmentsPx;
    draw();
  });
  await listen<Settings>("settings://changed", (e) => {
    applySettings(e.payload);
    const src = (e.payload.map?.basemap as string) ?? "vulnona";
    if (src !== currentSource) {
      void reloadMapSource();
      return; // reloadMapSource draws when the new frame is ready
    }
    draw();
  });

  // Provider-neutral live stats for the strip under the minimap.
  await listen<ProviderSnapshot>("provider://snapshot", (e) => {
    applyProviderSnapshot(e.payload);
    draw();
  });
  await listen<ProviderState>("provider://state", (e) => {
    applyProviderState(e.payload);
    draw();
  });
  await listen<CombatEvent>("combat://new", (e) => showCombatEvent(e.payload));

  // First-run / re-download / silent top-up completed: pick up the new data
  // live — including overlays that did not exist at init (get_map_info again).
  await listen("fetch://finished", () => void reloadMapSource());

  // Initial state: position/trail otherwise arrive only as events, so a
  // fresh (re)loaded webview would sit on the hint disc until the player's
  // next manual copy.
  try {
    const p = await invoke<PositionUpdate | null>("get_current_position");
    if (p) {
      state.position = { xCm: p.xCm, yCm: p.yCm, px: p.px, py: p.py, headingDeg: p.headingDeg };
      applyHeading(p);
    }
    const trail = await invoke<{ segmentsPx: [number, number][][] }>("get_current_trail");
    state.trailPx = trail.segmentsPx;
  } catch {
    // Stays on the hint disc until the first event.
  }

  // First paint before the window is shown (Rust shows it on this signal).
  applyHeading(await invoke<HeadingUpdate>("get_current_heading"));
  draw();
  await emit("minimap://ready", {});

  // Data load can lag behind the first paint; draws again when ready.
  void loadData();
  void refreshWaypoints();
}

void init().catch((e) => {
  void error(`[minimap] init failed: ${e}`).catch(() => {});
  // A blank-but-alive overlay beats an invisible one: Rust wires up the
  // supervisor on this signal (and has its own 5 s fallback besides).
  void emit("minimap://ready", {});
});
