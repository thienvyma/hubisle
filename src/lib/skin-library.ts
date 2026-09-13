export interface SavedSkinPreset {
  name: string;
  colors: string[];
  variation: number;
}

const MAX_PRESETS = 20;
const MAX_NAME_LENGTH = 40;

const isColor = (value: unknown): value is string =>
  typeof value === "string" && /^#[0-9a-f]{6}$/i.test(value);

const normalizedPreset = (value: unknown): SavedSkinPreset | null => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) return null;
  const source = value as Record<string, unknown>;
  const name = typeof source.name === "string" ? source.name.trim() : "";
  if (!name || name.length > MAX_NAME_LENGTH) return null;
  if (!Array.isArray(source.colors) || source.colors.length < 1 || source.colors.length > 32) return null;
  if (!source.colors.every(isColor)) return null;
  const variation = typeof source.variation === "number" && Number.isFinite(source.variation)
    ? Math.max(0, Math.min(1, source.variation))
    : 0;
  return {
    name,
    colors: source.colors.map((color) => color.toUpperCase()),
    variation,
  };
};

export function parseSkinLibrary(raw: string | null): SavedSkinPreset[] {
  if (!raw) return [];
  try {
    const decoded: unknown = JSON.parse(raw);
    if (!Array.isArray(decoded)) return [];
    const result: SavedSkinPreset[] = [];
    const seen = new Set<string>();
    for (const value of decoded) {
      const preset = normalizedPreset(value);
      const key = preset?.name.toLocaleLowerCase();
      if (!preset || !key || seen.has(key)) continue;
      result.push(preset);
      seen.add(key);
      if (result.length === MAX_PRESETS) break;
    }
    return result;
  } catch {
    return [];
  }
}

export function upsertSkinPreset(
  existing: readonly SavedSkinPreset[],
  name: string,
  colors: readonly string[],
  variation: number,
): SavedSkinPreset[] {
  const preset = normalizedPreset({ name, colors: [...colors], variation });
  if (!name.trim() || name.trim().length > MAX_NAME_LENGTH) throw new Error("invalid skin preset name");
  if (!preset) throw new Error("invalid skin preset colors");

  const key = preset.name.toLocaleLowerCase();
  const next = existing.map((item) => ({ ...item, colors: [...item.colors] }));
  const index = next.findIndex((item) => item.name.toLocaleLowerCase() === key);
  if (index >= 0) next[index] = preset;
  else next.unshift(preset);
  return next.slice(0, MAX_PRESETS);
}

export function removeSkinPreset(
  existing: readonly SavedSkinPreset[],
  name: string,
): SavedSkinPreset[] {
  const key = name.trim().toLocaleLowerCase();
  return existing
    .filter((item) => item.name.toLocaleLowerCase() !== key)
    .map((item) => ({ ...item, colors: [...item.colors] }));
}
