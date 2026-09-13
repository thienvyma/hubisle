export const DEFAULT_PROVIDER_WEBSITE = "https://islepilot.eu";

/**
 * Seed the connection form for a fresh install without replacing an existing
 * user's saved server choice.
 */
export function connectionWebsite(configured: string | null | undefined): string {
  const value = configured?.trim();
  return value || DEFAULT_PROVIDER_WEBSITE;
}
