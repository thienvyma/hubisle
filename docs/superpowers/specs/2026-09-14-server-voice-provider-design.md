# Server Voice Provider design

## Goal

Replace the current single-client Voice bridge with a server-neutral provider
framework. A server that exposes a legitimate player-login and Voice grant API
can be added through one adapter without changing the Voice UI or depending on
another desktop application.

This change prepares authentication and room-grant boundaries only. Audio
transport, microphone capture, push-to-talk, proximity processing, embedding,
and background-client integration are outside this implementation.

## Product behavior

The Voice tab uses the currently selected game-server connection as its context.
It asks the Voice registry for an adapter whose verified origin matches that
server. The tab displays the adapter name and one normalized state:

- `not-configured`: the current server has no Voice adapter;
- `login-required`: an adapter exists and supports an independent login;
- `authorizing`: a browser login is in progress;
- `ready`: an authenticated session can request a Voice grant;
- `unavailable`: the adapter is configured but its service is temporarily
  unreachable;
- `blocked`: the server requires a private or undocumented client capability;
- `error`: the adapter rejected malformed or invalid data.

The empty initial registry is valid. In that state the tab explains that the
current server has not configured Voice for this hub. It does not offer a
download, search the computer for another client, inspect another application's
state, or name a particular server.

## Architecture

Rust owns provider selection, login orchestration, callback validation, session
storage, and Voice-grant requests. The frontend receives normalized models and
never sees provider service credentials. The new `voice` module contains:

- `model.rs`: serialized provider descriptors, server context, status, login
  start data, callback data, session metadata, and short-lived Voice grants;
- `provider.rs`: the `VoiceProvider` interface and normalized provider errors;
- `registry.rs`: exact-origin adapter lookup with duplicate and lookalike-host
  rejection;
- `manager.rs`: state transitions and command orchestration;
- `session_store.rs`: provider-scoped session persistence protected with Windows
  DPAPI;
- `mod.rs`: the small Tauri command surface.

Each adapter implements these operations:

1. `matches_server` checks a normalized HTTPS origin from the active server
   connection.
2. `start_login` creates a fresh state value, nonce, callback URI, and provider
   authorization URL.
3. `finish_login` accepts a callback once, verifies its scheme, state, nonce,
   age, and provider identity, then returns an opaque player session.
4. `session_status` determines whether the saved session is usable without
   exposing it to the frontend.
5. `request_grant` exchanges that session for a short-lived room grant.
6. `logout` revokes when supported and deletes the local session.

Provider adapters are compiled into the application. Arbitrary remote JSON
cannot add login endpoints or callback schemes. This prevents an untrusted game
server or copied URL from redirecting player credentials to a new origin.

## Shared data contracts

`VoiceProviderDescriptor` contains a stable provider ID, display name, exact
server origins, authentication method, and declared capabilities. Provider IDs
are opaque strings rather than the existing map-provider enum so Voice support
can evolve independently from map, Garage, Skin, and Friends integrations.

`VoiceServerContext` contains the selected server origin and optional server ID
or display name. It never contains map cookies or another feature's bearer
token. An adapter may explicitly request an authenticated identity from a shared
provider only through a typed capability added to that provider; implicit token
reuse is forbidden.

`VoiceAuthStart` contains the system-browser authorization URL, callback scheme,
and expiry time. Secret verifier data remains in the Rust manager.

`VoiceSessionSummary` exposes only provider ID, player display name, optional
SteamID64, and expiry. The stored record contains the opaque server-issued
session plus the exact provider ID and origin that issued it.

`VoiceGrant` contains `serverUrl`, access token, room, identity, and expiry. It
exists only in memory and is never written to settings, logs, crash reports, or
frontend localStorage. A future audio engine consumes it directly and must
discard it at expiry or disconnect.

## Data flow

When the Voice tab opens, the frontend calls `voice_status`. The manager reads
the active provider state, normalizes its server origin, and selects an exact
matching Voice adapter. With no match it returns `not-configured`.

For a configured adapter, `voice_start_login` stores a one-time login attempt in
memory and opens the returned HTTPS authorization URL in the system browser.
The application's existing deep-link listener routes only the dedicated Voice
callback path to `voice_handle_callback`; unrelated login callbacks retain their
current behavior. A valid callback creates a DPAPI-protected session scoped to
the adapter and exact server origin.

`voice_request_grant` loads the scoped session and asks the selected adapter for
a grant. Provider-specific responses are validated and normalized before being
returned. The first framework release does not join a room because the requested
audio and push-to-talk work is out of scope.

## Security and privacy

All authorization starts use HTTPS. Callback acceptance requires a registered
application scheme, exact callback path, constant-time state comparison, an
unexpired pending attempt, and single use. Adapters must use Authorization Code
with PKCE when the provider supports OAuth. The framework never accepts a Steam
identity supplied only by the caller; the provider must return a server-issued
session after its own authentication flow.

No adapter may contain a private server key, reusable service credential, token
copied from another client, or mechanism intended to imitate client attestation.
If a server requires one of those, its adapter returns `blocked` and explains
that the server must expose a public login or token broker.

DPAPI binds persisted sessions to the current Windows account. Logout removes
the provider-scoped record. Diagnostic output contains state names and provider
IDs only; authorization codes, access tokens, cookies, grant JWTs, microphone
data, and room secrets are excluded.

## UI

Rename the component to `VoiceTab.svelte`. It shows the current server, adapter
name when available, normalized status, session identity summary, and actions
allowed by that status. The initial state has Refresh and a concise explanation
for server owners: provide a documented login callback and short-lived Voice
grant endpoint to enable an adapter.

All user-facing strings are generic in English and Vietnamese. No copy refers to
an official launcher, a downloadable client, a specific service, or a specific
server. Existing map-provider tabs and connection behavior remain unchanged.

## Failure handling

Malformed server origins, callbacks, and provider responses fail closed. Network
timeouts produce `unavailable` without deleting a still-valid session. An
authentication rejection clears only the matching provider session and returns
`login-required`. A provider mismatch or callback with no pending attempt is
ignored and logged without sensitive fields. A failed Voice operation cannot
crash or block Map, Dino, Friends, Garage, Skin, History, or Settings.

## Testing and acceptance

Rust unit tests use a synthetic provider and synthetic credentials. They verify
exact-origin matching, lookalike rejection, duplicate registration rejection,
provider-independent IDs, state transitions, callback single use, wrong-state
rejection, expiry rejection, origin-scoped sessions, grant normalization, and
redacted errors. Tests must also scan the active Voice module and UI for the old
executable name, external-client paths, service-specific endpoints, and
hard-coded `x-api-key` values.

Frontend validation covers all normalized states and confirms the generic empty
state renders without an installed provider. Repository checks include Svelte
type checking, production frontend build, Rust formatting, Clippy, and the Rust
workspace test suite.

Acceptance requires that a clean Windows account can install and open this hub,
visit Voice, and receive a stable `not-configured` state without another Voice
application, its files, cookies, localStorage, process, tokens, or registry
entries. Adding a future server requires one provider adapter plus registry
registration; it does not require changes to the shared Voice UI or manager.
