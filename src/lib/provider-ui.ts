import type { ConnectionStatus, SharedStatBar } from "$lib/api";

export function providerAllowsMain(status: ConnectionStatus): boolean {
  return (
    status === "authenticated-online" ||
    status === "authenticated-offline" ||
    status === "temporary-error"
  );
}

export function formatStat(stat: SharedStatBar | null): string {
  if (!stat) return "—";
  return stat.current != null && stat.max != null
    ? `${stat.current} / ${stat.max}`
    : `${Math.round(stat.percent)}%`;
}

export function providerLabel(provider: string | null): string {
  if (provider === "era") return "Era Gaming VN";
  if (provider === "titan") return "The Real Server VN";
  if (provider === "isle-pilot") return "IslePilot";
  return "Manual";
}
