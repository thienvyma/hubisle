# Multi-provider login and live-data design

Date: 2026-09-08

## Goal

Keep the public TheIsle Overlay 1.5.2 codebase as the application shell because it already has the full Gateway map, minimap, waypoints, trails, Dino view, settings, global hotkeys, and Windows packaging. Replace its IslePilot-only connection flow with a provider system that initially supports IslePilot, Era Gaming VN, and The Real Server VN (the Titan overlay supplied by the user).

The app must require a valid provider connection on first launch. Once authenticated, a player being offline or at the game menu is a normal state and must not send the user back to login.

The Donate tab remains removed. The custom build must not update itself from the upstream closed-source release channel.

## Evidence and constraints

The source repository after 1.5.2 is closed source, so this work extends the last public code rather than modifying release 2.3.2.

The supplied `TheRealServerVN-Overlay.exe` is a small .NET Framework WebView2 shell. Its local `overlay-config.json` points to `https://therealservervn.com/nguoi-choi/overlay`. The intelligence and player data live on the website, not in the executable.

The authenticated Titan page was observed using `/api/nguoi-choi/dino?sv=<opaque-server-id>` and refreshing every 20 seconds. Its current page exposes the active server identifier in `#dino[data-sv]`. It displays species, sex, growth, stamina, hunger, thirst, mutations, health percentage/current/max, position, and ten Prime conditions. Authentication is a Titan website account linked to Steam, not the IslePilot token and not the Era website session.

Era's public frontend uses authenticated `GET /api/theisle/map`, normally polls around 12 seconds, and exposes `success`, `serverOnline`, `playerOnline`, and `player`. Previously observed live data proved that position changes without manually copying Asset Location. The frontend contract includes class, location, growth/health/stamina/hunger/thirst percentages and optional exact vitals. Complete live Prime and nutrition contracts have not been verified.

IslePilot already has working token and legacy-cookie implementations in the public overlay. One central overlay token follows a player across servers integrated with IslePilot.

A website address alone does not reveal an arbitrary private API. Automatic detection is therefore limited to verified provider adapters. Unknown websites must remain unsupported until their API and authentication contract are verified.

## Architecture

### Provider registry

A provider registry owns URL normalization and exact host matching. It returns a stable provider ID and provider metadata before any credential is accepted or sent.

Initial rules:

- `islepilot.eu`, verified IslePilot subdomains, and verified IslePilot tenant paths resolve to `islepilot`.
- Exact `eragamingvn.net` resolves to `era`.
- Exact `therealservervn.com` and `www.therealservervn.com` resolve to `titan`.
- Lookalike domains, insecure HTTP URLs, URLs containing user information, and arbitrary ports are rejected.
- Other HTTPS origins resolve to `unsupported`; the app explains that an adapter is required.

Provider detection is declarative. Adding another supported platform later should require a registry entry, an authentication adapter, a polling adapter, a normalizer, and contract tests; it must not require changes to the map renderer.

### Provider adapters

Each adapter implements the same lifecycle:

1. detect or accept its configured origin;
2. open the provider-specific login window;
3. validate and persist the resulting credential or website session;
4. return connection state;
5. poll the player's own data at the provider-supported cadence;
6. normalize the response into a shared snapshot;
7. log out and delete only that provider's saved credential.

IslePilot preserves the existing central token flow and legacy mode. Era uses a website session restricted to the Era origin and validates it against `/api/theisle/map`. Titan uses a website session restricted to the Titan origin. After login, it loads `/nguoi-choi`, obtains the active opaque server ID supplied by the page, then requests `/api/nguoi-choi/dino?sv=<id>` on the same origin.

The provider webviews have no Tauri IPC capability. HTTP clients must not forward cookies or authorization headers across origins during redirects. Credentials are stored separately and encrypted with the existing Windows DPAPI mechanism. No browser password, cookie, or session is copied from Chrome or the Codex browser.

### Shared snapshot

All adapters produce one normalized snapshot containing:

- provider ID and source origin;
- connection state and player state;
- server identity and display name when supplied;
- received-at time and source timestamp when genuinely supplied;
- optional world position in centimetres;
- optional species, sex, growth, health, stamina, hunger, thirst, mutations, nutrition, and Prime conditions;
- capability flags that describe which fields the current provider response actually supplied.

Missing data remains `null` or absent. It must never become zero, a fabricated maximum, or data retained from another server. A receive timestamp records when the overlay received a response; it must not be presented as proof that the server generated fresh data at that moment.

Era coordinates require the verified axis conversion: `player.location.y` becomes overlay `xCm`, and `player.location.x` becomes overlay `yCm`. Titan and IslePilot adapters each keep their provider-specific conversion inside their normalizer.

Percentage-only vitals are represented explicitly. If a provider gives health as 74%, the app displays 74%; it does not invent `74/100`. Current/max values are accepted only when valid and consistent with any reported percentage.

## Data routing

Only one provider generation is active at a time. Selecting, logging out of, or switching a provider increments a generation counter. Poll responses carry the generation they started with and are discarded if it is no longer current.

On a provider switch, authentication expiry, or player-offline transition, the app clears the previous player's snapshot and map marker. It also starts a new trail segment so coordinates from different servers are never joined. Clipboard position remains an explicit manual fallback; it cannot overwrite an active automatic provider unless the user selects manual mode.

The common position pipeline continues to feed both the full map and minimap. The common Dino event feeds the main Dino view and minimap HUD. Garage, live POI, skin, or other provider-specific features are enabled only when the active adapter advertises them. Era or Titan must not call IslePilot Garage endpoints.

## Startup and connection user experience

On launch, the app first checks stored provider configuration:

- No configured provider: show the mandatory connection screen.
- Credential validates: enter the main application even if the dinosaur is offline.
- HTTP 401/expired session: show the connection screen with a re-login action.
- Temporary network failure: show retry and offline status without deleting a valid saved credential.
- Unsupported saved origin: show the connection screen and preserve non-sensitive settings.

The connection screen contains a website field, provider detection result, the correct login action, status, and a concise explanation for unsupported sites. It does not show the normal navigation until a provider has authenticated successfully. Existing IslePilot users are migrated by detecting their saved token or valid legacy configuration.

After connection, the Dino tab displays player data and provider status. Login controls move out of the Dino content into the connection screen. A Change connection action remains available from the Dino tab or settings. Logging out returns to the mandatory connection screen.

## Provider-specific behavior

### IslePilot

- Preserve central Steam token login and cross-IslePilot-server behavior.
- Preserve legacy per-server cookie mode as a fallback.
- Preserve supported Garage, map layers, nutrition, and Prime data.
- Stop and clear its poller when another provider becomes active.

### Era

- Use only `https://eragamingvn.net` and its verified login/API paths.
- Map position, species, growth and available vitals into the shared snapshot.
- Show Prime and nutrition as unavailable until an authenticated live contract is verified.
- Respect the provider cadence; the known website cadence is about 12 seconds, not sub-second realtime.

### Titan / The Real Server VN

- Use only the exact Titan origins.
- Login with the Titan panel account and preserve its own same-origin session.
- Discover the server ID supplied by the authenticated player page rather than asking the user to enter or guess the opaque UUID.
- Poll the documented player endpoint at approximately the website's 20-second cadence.
- Map the fields the response actually supplies, including Prime and mutations when present.
- Treat the server ID as provider-owned opaque data and rediscover it when the account changes servers.

## Error handling

The shared connection states are: unconfigured, detecting, login-required, validating, authenticated-online, authenticated-offline, temporary-error, unsupported, and logged-out.

Parser failures do not retain the last player's values indefinitely. The UI shows the last successful update age only while the snapshot is explicitly marked cached; after the stale threshold it clears player values and reports the source problem.

Errors shown to the user name the provider and action. Logs may include provider ID, HTTP status, and a redacted origin, but never cookies, authorization tokens, Steam identifiers, email addresses, full response bodies, or precise coordinates.

## Testing

Unit and integration tests cover:

- exact provider detection, normalized URLs, lookalike domains, HTTP rejection, user-info rejection, and redirect credential isolation;
- Era axis conversion, zero versus missing values, non-finite coordinates, exact-vital consistency, offline/server-offline/401 responses;
- Titan extraction of the opaque server ID and normalization of percentages, current/max health, mutations, position, and ten Prime flags;
- IslePilot regression coverage for token and legacy modes;
- generation changes discarding late responses and preventing clipboard/provider mixing;
- startup routing for no credentials, valid offline player, expired login, and temporary network failure;
- provider-specific feature gating and clearing old stats/position when switching.

Frontend type checks and production builds must pass. Rust tests and the Windows Tauri/NSIS build must pass using the locally installed build toolchain. After a successful installer exists, back up the user's existing settings and data, uninstall the old overlay, install the custom build, and validate startup/login/minimap behavior. Any provider login that requires credentials remains a user action in the app's provider webview.

## Versioning and installation

The build is labeled as a custom version based on public 1.5.2; it must not claim to be upstream 2.3.2. Upstream updater configuration and update prompts are disabled. The Windows identifier and installer behavior are chosen so the explicitly authorized old-app replacement is predictable, while user data is backed up before uninstall.

The old installation is not removed until the custom installer has built and passed static/runtime checks. The supplied Titan application and its data remain untouched; it is a research reference and test source.
