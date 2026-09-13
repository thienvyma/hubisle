/** Convert a six-digit CSS colour to its exact sRGB byte channels. */
export function hexToRgb(hex: string): [number, number, number] {
  const normalized = hex.startsWith("#") ? hex.slice(1) : hex;
  if (!/^[0-9a-f]{6}$/i.test(normalized)) {
    throw new Error(`Invalid skin colour: ${hex}`);
  }
  const value = Number.parseInt(normalized, 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}
