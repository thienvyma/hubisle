-- Short-lived hub-to-hub positions. No raw SteamID, player name, token, IP,
-- or server name is stored. Reads are primary-key lookups for the caller's
-- accepted IslePilot friends, so this intentionally has no secondary index.
CREATE TABLE IF NOT EXISTS friend_presence (
  user_hash   TEXT    NOT NULL PRIMARY KEY,
  server_hash TEXT    NOT NULL,
  x_cm        REAL    NOT NULL,
  y_cm        REAL    NOT NULL,
  z_cm        REAL    NOT NULL,
  updated_at  INTEGER NOT NULL
) STRICT, WITHOUT ROWID;
