// Isle Pulse HUD palette. No display strings here (see i18n/).

export const COLORS = {
  bg: "#030711",
  panel: "#08111f",
  panelBorder: "#1a3852",
  text: "#edf6ff",
  textMuted: "#7890aa",
  accent: "#35f2ff",
  player: "#c65cff",
  playerArrow: "#35f2ff",
  playerArrowOutline: "#020711",
  trail: "#c65cff",
  waypoint: "#45f5a2",
} as const;

// Keys match pois_gateway.json layer keys (+ image-overlay layer keys).
export const LAYER_COLORS: Record<string, string> = {
  freshwater: "#149af2", // islemaps.com's own fresh-water blue
  water: "#35bdf2",
  saltlick: "#ffc857",
  mudwallow: "#9c7b4f",
  sanctuary: "#a855f7",
  migration: "#45f5a2",
  food: "#ff5678",
  patrol: "#ff708c",
  animal: "#ff62bc",
  region: "#edf6ff",
  landmark: "#9cb1c7",
  islepilot: "#35f2ff",
};

// Draw order: image overlays lowest, big zones next, small dots after, text
// labels on top.
export const LAYER_ORDER = [
  "freshwater",
  "islepilot",
  "patrol",
  "migration",
  "sanctuary",
  "food",
  "water",
  "mudwallow",
  "saltlick",
  "animal",
  "landmark",
  "region",
];

// Waypoint icon presets (offered in the naming prompt). A waypoint whose
// name STARTS with one of these renders as that glyph on both maps instead
// of a colour dot — the name itself is the single source of truth, so the
// on-disk waypoint format stays byte-compatible.
export const WAYPOINT_GLYPHS = ["🚩", "💀", "🏠", "💧", "⚠️", "🍖"];

/** The glyph a waypoint renders as, or undefined for the plain colour dot. */
export function waypointGlyph(name: string): string | undefined {
  return WAYPOINT_GLYPHS.find((g) => name.startsWith(g));
}

// One recognisable glyph per animal species (labels from the islemaps
// sighting data). Rendered as text: Segoe UI Emoji covers all of these on
// Windows. Species without a glyph fall back to the layer-colour dot.
export const ANIMAL_GLYPHS: Record<string, string> = {
  Boar: "🐗",
  Bunny: "🐰",
  Chicken: "🐔",
  Crab: "🦀",
  Deer: "🦌",
  Frog: "🐸",
  Goat: "🐐",
  Teno: "🦕",
  Turtle: "🐢",
};

export const ZONE_FILL_OPACITY = 60 / 255;
export const ZONE_STROKE_OPACITY = 190 / 255;

export const POI_DOT_RADIUS = 5;
export const PLAYER_DOT_RADIUS = 7;
export const WAYPOINT_RADIUS = 6;

// Basemap geometry deliberately lives in Rust (get_map_info) — it varies with
// the selected basemap source, so no pixel constants belong here.
