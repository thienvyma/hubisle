import { json, sha256Hex } from "./util";
import type { Env } from "./env";

const ISLEPILOT_ORIGIN = "https://islepilot.eu";
const PRESENCE_TTL_S = 60;
const MAX_FRIENDS = 32;

interface OverlayMe {
  hasData?: boolean;
  steamId?: string;
  server?: string;
  online?: boolean;
}

interface OverlayFriend {
  steamId?: string;
  status?: string;
}

interface OverlayFriends {
  shareLocation?: boolean;
  friends?: OverlayFriend[];
}

interface PresenceRow {
  user_hash: string;
  x_cm: number;
  y_cm: number;
  z_cm: number;
  updated_at: number;
}

const isSteamId = (value: unknown): value is string =>
  typeof value === "string" && /^\d{17}$/.test(value);

function finiteCoordinate(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) && Math.abs(value) <= 10_000_000
    ? value
    : null;
}

function normalServer(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim().replace(/\s+/g, " ").toLocaleLowerCase("en-US");
  return normalized && normalized.length <= 160 ? normalized : null;
}

async function privateHash(env: Env, kind: "player" | "server", value: string): Promise<string> {
  // A domain separator prevents the same digest from being meaningful in a
  // different table. ATTEST_MASTER never leaves the Worker.
  return sha256Hex(`${env.ATTEST_MASTER}\0presence:${kind}\0${value}`);
}

async function officialJson<T>(path: string, token: string): Promise<T | null> {
  const response = await fetch(`${ISLEPILOT_ORIGIN}${path}`, {
    headers: {
      authorization: `Bearer ${token}`,
      accept: "application/json",
      "x-overlay-version": "2",
    },
  });
  if (!response.ok) return null;
  const value = (await response.json()) as unknown;
  return value && typeof value === "object" ? (value as T) : null;
}

/**
 * Exchange short-lived positions between users of this hub.
 *
 * The IslePilot bearer token is used only to ask IslePilot who the caller is,
 * which server they are on, and which relationships are accepted. It is never
 * written to D1. D1 stores only secret-peppered identifiers and coordinates,
 * and callers can read only accepted friends on the same server.
 */
export async function handlePresence(req: Request, env: Env): Promise<Response> {
  const authorization = req.headers.get("authorization") ?? "";
  const match = /^Bearer\s+([^\s]{16,4096})$/i.exec(authorization);
  if (!match) return new Response(null, { status: 401 });
  const token = match[1];

  const { success } = await env.RL_PRESENCE.limit({ key: await sha256Hex(token) });
  if (!success) return new Response(null, { status: 429 });

  const declared = Number(req.headers.get("content-length") ?? "0");
  if (declared > 2048) return new Response(null, { status: 413 });
  let body: Record<string, unknown>;
  try {
    const raw = await req.text();
    if (raw.length > 2048) return new Response(null, { status: 413 });
    const value = JSON.parse(raw) as unknown;
    if (!value || typeof value !== "object" || Array.isArray(value)) {
      return new Response(null, { status: 400 });
    }
    body = value as Record<string, unknown>;
  } catch {
    return new Response(null, { status: 400 });
  }

  const [me, relationships] = await Promise.all([
    officialJson<OverlayMe>("/api/overlay/me", token),
    officialJson<OverlayFriends>("/api/overlay/friends", token),
  ]);
  if (!me || !relationships || !isSteamId(me.steamId)) {
    return new Response(null, { status: 401 });
  }

  const server = normalServer(me.server);
  const position = Array.isArray(body.positionCm) ? body.positionCm : [];
  const x = finiteCoordinate(position[0]);
  const y = finiteCoordinate(position[1]);
  const z = finiteCoordinate(position[2]) ?? 0;
  const sharing = relationships.shareLocation === true;
  const active = me.hasData === true && me.online === true && server !== null;
  const now = Math.floor(Date.now() / 1000);
  const selfHash = await privateHash(env, "player", me.steamId);

  if (sharing && active && x !== null && y !== null) {
    const serverHash = await privateHash(env, "server", server);
    await env.DB.prepare(
      `INSERT INTO friend_presence (user_hash, server_hash, x_cm, y_cm, z_cm, updated_at)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6)
       ON CONFLICT (user_hash) DO UPDATE SET
         server_hash = excluded.server_hash,
         x_cm = excluded.x_cm,
         y_cm = excluded.y_cm,
         z_cm = excluded.z_cm,
         updated_at = excluded.updated_at`,
    ).bind(selfHash, serverHash, x, y, z, now).run();
  } else {
    await env.DB.prepare("DELETE FROM friend_presence WHERE user_hash = ?1")
      .bind(selfHash)
      .run();
    return json({ server: me.server ?? null, ttlSeconds: PRESENCE_TTL_S, friends: [] });
  }

  const acceptedSteamIds = (relationships.friends ?? [])
    .filter((friend) => friend.status?.toLocaleLowerCase("en-US") === "accepted")
    .map((friend) => friend.steamId)
    .filter(isSteamId)
    .slice(0, MAX_FRIENDS);
  if (acceptedSteamIds.length === 0) {
    return json({ server: me.server ?? null, ttlSeconds: PRESENCE_TTL_S, friends: [] });
  }

  const hashes = await Promise.all(
    acceptedSteamIds.map((steamId) => privateHash(env, "player", steamId)),
  );
  const serverHash = await privateHash(env, "server", server);
  const placeholders = hashes.map((_, index) => `?${index + 3}`).join(", ");
  const rows = await env.DB.prepare(
    `SELECT user_hash, x_cm, y_cm, z_cm, updated_at
       FROM friend_presence
      WHERE server_hash = ?1 AND updated_at >= ?2
        AND user_hash IN (${placeholders})`,
  )
    .bind(serverHash, now - PRESENCE_TTL_S, ...hashes)
    .all<PresenceRow>();
  const byHash = new Map((rows.results ?? []).map((row) => [row.user_hash, row]));
  const friends = acceptedSteamIds.flatMap((steamId, index) => {
    const row = byHash.get(hashes[index]);
    if (!row) return [];
    return [{
      steamId,
      positionCm: [row.x_cm, row.y_cm, row.z_cm],
      ageSeconds: Math.max(0, now - row.updated_at),
    }];
  });

  return json({ server: me.server ?? null, ttlSeconds: PRESENCE_TTL_S, friends });
}
