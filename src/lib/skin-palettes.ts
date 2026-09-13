export type SkinPaletteProvider = "era" | "titan" | "isle-pilot";

export const SKIN_PRESET_KEYS = [
  "green_black",
  "black",
  "white",
  "forest",
  "desert",
  "shadow",
  "snow",
  "ocean",
  "ember",
] as const;

export type SkinPresetKey = (typeof SKIN_PRESET_KEYS)[number];

type Preset = {
  preview: readonly [string, string, string];
  era: readonly string[];
  titan: readonly string[];
  islePilot: readonly string[];
};

const GREEN_BLACK = [
  "#174A2B", "#050806", "#28683A", "#10351F", "#0A170F",
  "#3B8F50", "#D4B83F", "#E8E2CF", "#552D34", "#050706",
] as const;
const BLACK = [
  "#080A0A", "#000000", "#111414", "#050606", "#181B1B",
  "#020303", "#D6A92C", "#DDD8CA", "#4A252B", "#000000",
] as const;
const WHITE = [
  "#F2F4F2", "#9FA8A8", "#DDE2DF", "#F8F8F3", "#BCC5C2",
  "#FFFFFF", "#75C9E8", "#FFFDF2", "#A96F78", "#A7ADAC",
] as const;
const FOREST = [
  "#3E4A26", "#1C260F", "#4A5A2C", "#8A9A6A", "#334020",
  "#5CA65E", "#C8D020", "#F1E9C8", "#593537", "#18200F",
] as const;
const DESERT = [
  "#B47A45", "#5C3425", "#D2A260", "#E0C28E", "#8A5836",
  "#C56A32", "#E7C33F", "#F1E2C5", "#743A32", "#3B2921",
] as const;
const SHADOW = [
  "#2C2C34", "#101014", "#3A3A42", "#4A4A52", "#202028",
  "#5B4978", "#A980E8", "#E8E2DC", "#5A2D48", "#101014",
] as const;
const SNOW = [
  "#D8DDE4", "#7A8290", "#C4CCD6", "#EEF2F6", "#A8B0BC",
  "#E5FFF2", "#60A0D0", "#FFFDF2", "#9B6875", "#767E88",
] as const;
const OCEAN = [
  "#174A5C", "#071D2B", "#24748A", "#A7D4CF", "#155269",
  "#39A6B5", "#B8EEFF", "#EEF7F3", "#5A304D", "#09202A",
] as const;
const EMBER = [
  "#6B201A", "#190807", "#A13A20", "#D0803C", "#4A1210",
  "#F05B24", "#FFD34E", "#EFE0C8", "#6E1F2B", "#1A0B09",
] as const;

const firstSeven = (colors: readonly string[]) => colors.slice(0, 7);

export const SKIN_PRESETS: Record<SkinPresetKey, Preset> = {
  green_black: {
    preview: [GREEN_BLACK[0], GREEN_BLACK[1], GREEN_BLACK[5]],
    era: firstSeven(GREEN_BLACK), titan: firstSeven(GREEN_BLACK), islePilot: GREEN_BLACK,
  },
  black: {
    preview: [BLACK[0], BLACK[1], BLACK[4]],
    era: firstSeven(BLACK), titan: firstSeven(BLACK), islePilot: BLACK,
  },
  white: {
    preview: [WHITE[0], WHITE[1], WHITE[5]],
    era: firstSeven(WHITE), titan: firstSeven(WHITE), islePilot: WHITE,
  },
  forest: {
    preview: [FOREST[0], FOREST[1], FOREST[5]],
    era: ["#1C4D3B", "#5CA65E", "#D1B75A", "#55733C", "#D9E6B4", "#203C3A", "#F1E9C8"],
    titan: ["#4A5A2C", "#2C3A1C", "#3E4A26", "#334020", "#8A9A6A", "#1C260F", "#C8D020"],
    islePilot: FOREST,
  },
  desert: {
    preview: [DESERT[0], DESERT[1], DESERT[3]],
    era: ["#8B2C21", "#E15F2D", "#F2C24D", "#B74325", "#FFF0A0", "#682D26", "#FFF7D1"],
    titan: ["#C8A86A", "#9A7A4A", "#BFA070", "#A88A5A", "#E0D0A8", "#6A5230", "#E0B020"],
    islePilot: DESERT,
  },
  shadow: {
    preview: [SHADOW[0], SHADOW[1], SHADOW[5]],
    era: ["#22234D", "#4543A1", "#A84EAF", "#34356F", "#E08FD0", "#272746", "#F3D7F2"],
    titan: ["#3A3A42", "#202028", "#2C2C34", "#26262C", "#4A4A52", "#101014", "#8060C0"],
    islePilot: SHADOW,
  },
  snow: {
    preview: [SNOW[0], SNOW[1], SNOW[5]],
    era: ["#123F4B", "#1B8C83", "#63D3A4", "#246272", "#D9F4B8", "#102B3D", "#E5FFF2"],
    titan: ["#D8DDE4", "#A8B0BC", "#C4CCD6", "#B4BCC8", "#EEF2F6", "#7A8290", "#60A0D0"],
    islePilot: SNOW,
  },
  ocean: {
    preview: [OCEAN[0], OCEAN[1], OCEAN[5]],
    era: firstSeven(OCEAN), titan: firstSeven(OCEAN), islePilot: OCEAN,
  },
  ember: {
    preview: [EMBER[0], EMBER[1], EMBER[5]],
    era: firstSeven(EMBER), titan: firstSeven(EMBER), islePilot: EMBER,
  },
};

const validColor = (value: string | undefined): value is string =>
  typeof value === "string" && /^#[0-9a-f]{6}$/i.test(value);

export function presetPalette(
  key: SkinPresetKey,
  provider: SkinPaletteProvider,
  current: readonly string[],
  lockedIndexes: ReadonlySet<number> = new Set(),
): string[] {
  const preset = SKIN_PRESETS[key];
  const source = provider === "era" ? preset.era : provider === "titan" ? preset.titan : preset.islePilot;
  return current.map((color, index) =>
    lockedIndexes.has(index) ? color : source[index % source.length],
  );
}

const toHex = (value: number) => Math.round(Math.max(0, Math.min(255, value))).toString(16).padStart(2, "0");

function hsl(hue: number, saturation: number, lightness: number): string {
  const h = ((hue % 360) + 360) % 360;
  const s = Math.max(0, Math.min(100, saturation)) / 100;
  const l = Math.max(0, Math.min(100, lightness)) / 100;
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  const [r, g, b] = h < 60 ? [c, x, 0]
    : h < 120 ? [x, c, 0]
      : h < 180 ? [0, c, x]
        : h < 240 ? [0, x, c]
          : h < 300 ? [x, 0, c]
            : [c, 0, x];
  return `#${toHex((r + m) * 255)}${toHex((g + m) * 255)}${toHex((b + m) * 255)}`.toUpperCase();
}

/** Build a coherent random palette while retaining provider-locked zones. */
export function randomSkinPalette(
  current: readonly string[],
  lockedIndexes: ReadonlySet<number> = new Set(),
  random: () => number = Math.random,
): string[] {
  const hue = random() * 360;
  const accent = hue + 120 + random() * 120;
  const generated = [
    hsl(hue, 42 + random() * 35, 24 + random() * 24),
    hsl(accent, 45 + random() * 35, 10 + random() * 20),
    hsl(hue + 18, 38 + random() * 35, 30 + random() * 25),
    hsl(hue - 12, 24 + random() * 30, 50 + random() * 24),
    hsl(hue + 35, 38 + random() * 35, 22 + random() * 26),
    hsl(accent + 18, 50 + random() * 35, 35 + random() * 27),
    hsl(accent + 180, 65 + random() * 30, 48 + random() * 30),
    hsl(42, 18 + random() * 12, 82 + random() * 12),
    hsl(350 + random() * 25, 30 + random() * 30, 24 + random() * 24),
    hsl(hue, 12 + random() * 18, 7 + random() * 15),
  ];

  return current.map((color, index) => {
    if (lockedIndexes.has(index)) return color;
    const generatedColor = generated[index % generated.length];
    return validColor(generatedColor) ? generatedColor : "#808080";
  });
}
