# Multi-provider Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the public 1.5.2 overlay require an authenticated data provider at startup and support IslePilot, Era Gaming VN, and The Real Server VN through one normalized live-data pipeline.

**Architecture:** Add a Rust provider registry and orchestrator around the existing IslePilot integration. Era and Titan get isolated same-origin session adapters that normalize their responses into shared player/position snapshots; Svelte renders one connection gate and one provider-neutral Dino HUD while provider-only features remain gated.

**Tech Stack:** Tauri 2, Rust 2021, reqwest blocking client, serde/serde_json, scraper, Windows DPAPI, Svelte 5, TypeScript, Vite 7, NSIS.

**Spec:** `docs/superpowers/specs/2026-09-08-multi-provider-overlay-design.md`

## Global Constraints

- Exact supported web origins are `https://eragamingvn.net`, `https://therealservervn.com`, `https://www.therealservervn.com`, and verified IslePilot origins/tenants.
- Reject HTTP, URL user information, arbitrary ports, and provider lookalike domains before handling credentials.
- Store every provider credential separately with Windows DPAPI and never log cookies, tokens, Steam IDs, email addresses, response bodies, or precise coordinates.
- A player being offline or in the game menu is authenticated state; HTTP 401 is login-required; a network error is temporary and must not erase credentials.
- Missing values remain absent. Never invent maximum values, convert missing to zero, or retain player values from another provider/server.
- Only one automatic source generation may publish. Late requests from older generations are discarded; manual clipboard input cannot overwrite an active automatic source.
- Preserve existing IslePilot token/legacy behavior and provider-specific features only when IslePilot is active.
- Keep the Donate tab removed. Label the build as a custom version based on public 1.5.2 and disable the upstream updater.
- Do not read or modify game memory/files, inject into the game, synthesize game input, or obtain credentials from external browsers.

---

### Task 1: Provider registry and shared contracts

**Files:**
- Create: `src-tauri/src/providers/mod.rs`
- Create: `src-tauri/src/providers/registry.rs`
- Create: `src-tauri/src/providers/model.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/settings.rs`
- Modify: `src/lib/api.ts`

**Interfaces:**
- Produces: `ProviderId::{IslePilot, Era, Titan}`, `ProviderConfig`, `ConnectionStatus`, `ProviderState`, `ProviderSnapshot`, `SharedPlayer`, `SharedStatBar`, `detect_provider(input: &str) -> Result<DetectedProvider, String>`.
- Produces settings object `provider: { id: string|null, website: string|null, automaticPosition: boolean }`.
- Consumes: existing `islepilot::parser::{Nutrition, QuestStatus}` and settings merge/persist behavior.

- [ ] **Step 1: Write provider detection and serialization tests**

Add Rust tests in `providers/registry.rs` that assert exact host matching:

```rust
#[test]
fn detects_verified_origins_and_rejects_lookalikes() {
    assert_eq!(detect_provider("eragamingvn.net/live-map").unwrap().id, ProviderId::Era);
    assert_eq!(detect_provider("https://www.therealservervn.com/nguoi-choi").unwrap().id, ProviderId::Titan);
    assert_eq!(detect_provider("https://mixi.islepilot.eu").unwrap().id, ProviderId::IslePilot);
    assert!(detect_provider("https://eragamingvn.net.example.org").is_err());
    assert!(detect_provider("http://eragamingvn.net").is_err());
    assert!(detect_provider("https://user:pass@eragamingvn.net").is_err());
    assert!(detect_provider("https://eragamingvn.net:8443").is_err());
}
```

Add model tests proving `None` serializes as absent/null while numeric zero survives, and `SharedStatBar::from_percent(0.0)` has no fabricated current/max.

- [ ] **Step 2: Run the focused tests and verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml providers::`

Expected: compilation fails because `providers`, `ProviderId`, and `detect_provider` do not exist.

- [ ] **Step 3: Implement the registry and shared model**

Use these stable shapes:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderId { IslePilot, Era, Titan }

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionStatus {
    Unconfigured, Detecting, LoginRequired, Validating,
    AuthenticatedOnline, AuthenticatedOffline, TemporaryError,
    Unsupported, LoggedOut,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SharedStatBar {
    pub percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")] pub current: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")] pub max: Option<f64>,
    pub raw: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SharedPlayer {
    pub name: Option<String>,
    pub dino_name: Option<String>,
    pub female: Option<bool>,
    pub growth_pct: Option<f64>,
    pub health: Option<SharedStatBar>,
    pub stamina: Option<SharedStatBar>,
    pub hunger: Option<SharedStatBar>,
    pub thirst: Option<SharedStatBar>,
    pub mutations: Vec<String>,
    pub nutrition: Option<Nutrition>,
    pub prime_quests: Vec<QuestStatus>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSnapshot {
    pub provider: ProviderId,
    pub status: ConnectionStatus,
    pub server_id: Option<String>,
    pub server_name: Option<String>,
    pub received_at_ms: i64,
    pub source_timestamp_ms: Option<i64>,
    pub player: Option<SharedPlayer>,
    pub position_cm: Option<(f64, f64, f64)>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderState {
    pub provider: Option<ProviderId>,
    pub website: Option<String>,
    pub status: ConnectionStatus,
    pub message: Option<String>,
    pub last_received_at_ms: Option<i64>,
}
```

Parse input by prepending `https://` only when no scheme is present. Require HTTPS, no username/password, no explicit port, then compare lowercase `host_str()` exactly. Canonicalize Titan `www` and non-`www` to the origin the user entered; do not transfer a cookie between them.

Add provider settings defaults without marking an old user configured. Migration recognizes an existing valid IslePilot token/legacy configuration at runtime rather than rewriting credentials during settings load.

- [ ] **Step 4: Extend frontend API contracts**

Add matching TypeScript unions and optional fields:

```ts
export type ProviderId = "isle-pilot" | "era" | "titan";
export type ConnectionStatus = "unconfigured" | "detecting" | "login-required" |
  "validating" | "authenticated-online" | "authenticated-offline" |
  "temporary-error" | "unsupported" | "logged-out";
export interface ProviderState { provider: ProviderId | null; website: string | null;
  status: ConnectionStatus; message: string | null; lastReceivedAtMs: number | null; }
```

- [ ] **Step 5: Re-run checks and commit**

Run `cargo test --manifest-path src-tauri/Cargo.toml providers::` and `npm run check`.

Expected: all focused Rust tests and Svelte/TypeScript checks pass.

Commit only Task 1 files with message `feat: add live-data provider contracts`.

---

### Task 2: Era session adapter and response normalizer

**Files:**
- Create: `src-tauri/src/providers/session_store.rs`
- Create: `src-tauri/src/providers/era.rs`
- Create: `src-tauri/fixtures/providers/era_online.json`
- Create: `src-tauri/fixtures/providers/era_offline.json`
- Modify: `src-tauri/src/providers/mod.rs`
- Modify: `src-tauri/src/islepilot/cookies.rs`

**Interfaces:**
- Consumes: Task 1 `ProviderSnapshot`, `SharedPlayer`, `SharedStatBar`, `ProviderId`; existing `cookies::{dpapi_protect, dpapi_unprotect}`.
- Produces: `era::normalize(value: &Value, received_at_ms: i64) -> Result<ProviderSnapshot, EraError>`, `era::validate(cookie: &str)`, `era::poll(cookie: &str)` and separate `ProviderSessionStore` records keyed by provider plus exact origin.

- [ ] **Step 1: Add synthetic fixture and failing normalizer tests**

State in fixture comments/tests that values are synthetic and field names come from public Era JavaScript. Cover:

```rust
#[test]
fn era_swaps_axes_and_preserves_percent_only_vitals() {
    let v = json!({"success":true,"serverOnline":true,"playerOnline":true,"player":{
      "class":"Stegosaurus","location":{"x":45100.0,"y":317900.0,"z":20900.0},
      "growthPercent":54.0,"healthPercent":0.0,"staminaPercent":96.0}});
    let s = normalize(&v, 1000).unwrap();
    assert_eq!(s.position_cm, Some((317900.0, 45100.0, 20900.0)));
    assert_eq!(s.player.as_ref().unwrap().health.as_ref().unwrap().percent, 0.0);
    assert_eq!(s.player.as_ref().unwrap().health.as_ref().unwrap().max, None);
}
```

Also test missing/empty/non-finite coordinates, server offline, player offline, malformed success, and exact vitals accepted only when current/max produce a percentage within five points of the percentage field.

- [ ] **Step 2: Run focused test and verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml providers::era::tests`

Expected: compilation fails because the Era adapter does not exist.

- [ ] **Step 3: Implement Era normalization**

Deserialize defensively from `serde_json::Value`. Accept only finite numbers. Convert location `(x,y,z)` into `(y,x,z)`. Convert percentage fields only within 0–100. Build exact stat bars with:

```rust
fn exact_bar(percent: Option<f64>, current: Option<f64>, max: Option<f64>) -> Option<SharedStatBar> {
    let pct = percent.filter(|v| v.is_finite() && (0.0..=100.0).contains(v))?;
    match (current, max) {
        (Some(cur), Some(cap)) if cur.is_finite() && cap.is_finite() && cur >= 0.0 && cap > 0.0
          && ((cur / cap * 100.0) - pct).abs() <= 5.0 => SharedStatBar::from_values(cur, cap),
        _ => Some(SharedStatBar::from_percent(pct)),
    }
}
```

Era Prime/nutrition remain absent until a verified contract exists.

- [ ] **Step 4: Implement origin-bound encrypted session storage and HTTP client**

Store `{provider, origin, cookie}` in a separate DPAPI file under the existing local settings directory. Build the blocking client with redirect policy `none`, timeout 15 seconds, `Accept: application/json`, and send `Cookie` only to exact `https://eragamingvn.net/api/theisle/map`.

Map HTTP 401 to `LoginRequired`, connection/timeout to `Temporary`, valid offline JSON to `AuthenticatedOffline`, and valid online JSON to `AuthenticatedOnline`.

- [ ] **Step 5: Run tests and commit**

Run `cargo test --manifest-path src-tauri/Cargo.toml providers::era::tests` and `cargo test --manifest-path src-tauri/Cargo.toml providers::session_store::tests`.

Expected: all tests pass; no fixture or assertion contains real user data.

Commit Task 2 files with message `feat: add Era live-data adapter`.

---

### Task 3: Titan session adapter and response normalizer

**Files:**
- Create: `src-tauri/src/providers/titan.rs`
- Create: `src-tauri/fixtures/providers/titan_online.json`
- Create: `src-tauri/fixtures/providers/titan_offline.json`
- Modify: `src-tauri/src/providers/mod.rs`

**Interfaces:**
- Consumes: Task 1 models and Task 2 `ProviderSessionStore`.
- Produces: `titan::discover_server_id(html: &str) -> Result<String, TitanError>`, `titan::normalize(value: &Value, received_at_ms: i64) -> Result<ProviderSnapshot, TitanError>`, `titan::validate(cookie, origin)`, and `titan::poll(cookie, origin, server_id)`.

- [ ] **Step 1: Write failing Titan contract tests**

Use synthetic data based on the observed public `player.js` contract:

```rust
#[test]
fn titan_extracts_server_id_and_full_player_fields() {
    assert_eq!(discover_server_id(r#"<div id="dino" data-sv="server-opaque-id"></div>"#).unwrap(), "server-opaque-id");
    let v = json!({"ok":true,"online":true,"server":"Synthetic TRS","dino":{"species":"Tyrannosaurus","growth":0.27,
      "stamina":1.0,"hunger":0.66,"thirst":0.89,"health":0.74,
      "hp":[44.0,59.0],"hp_that":true,"mutations":["SyntheticMutation"],
      "loc":[-238000.0,85000.0],"prime_dk":[false,false,false,false,false,false,true,true,true,true]}});
    let s = normalize(&v, 1000).unwrap();
    let p = s.player.unwrap();
    assert_eq!(p.growth_pct, Some(27.0));
    assert_eq!(p.mutations, vec!["SyntheticMutation"]);
    assert_eq!(p.prime_quests.len(), 10);
}
```

Read downloaded public `work/research-titan/player.js` rendering code before finalizing field names. Do not copy user-account values into fixtures or logs. Tests cover missing `data-sv`, malformed arrays, fractions versus percentages, missing health max, and offline state.

- [ ] **Step 2: Run focused test and verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml providers::titan::tests`

Expected: compilation fails because `providers::titan` does not exist.

- [ ] **Step 3: Implement server discovery and normalization**

Parse `/nguoi-choi` with `scraper`, selecting `#dino[data-sv]`. Treat the value as opaque and reject empty or excessively long values. Fetch `/api/nguoi-choi/dino?sv=<URL-encoded-id>` only on the exact saved Titan origin. Rediscover the identifier after offline/server-change responses rather than guessing it.

Normalize 0–1 fields to 0–100 only when the provider contract uses fractions. Preserve mutations as strings. Convert ten boolean `prime_dk` values into ten `QuestStatus` rows with stable Vietnamese/English labels held locally; set completed from booleans and do not infer completion from the displayed count.

- [ ] **Step 4: Implement Titan session validation**

Reuse origin-bound DPAPI storage and a no-redirect HTTP client. `GET /nguoi-choi` returning the login page/401 is `LoginRequired`; an authenticated page with no active dino is `AuthenticatedOffline`. Never submit account credentials from Rust; the user types them in the provider webview.

- [ ] **Step 5: Run tests and commit**

Run `cargo test --manifest-path src-tauri/Cargo.toml providers::titan::tests` and `cargo test --manifest-path src-tauri/Cargo.toml providers::session_store::tests`.

Expected: all tests pass and logs contain no session or player identifiers.

Commit Task 3 files with message `feat: add Titan live-data adapter`.

---

### Task 4: Provider orchestrator, login webviews, and stale-data reset

**Files:**
- Create: `src-tauri/src/providers/orchestrator.rs`
- Modify: `src-tauri/src/providers/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/pipeline.rs`
- Modify: `src-tauri/src/clipboard.rs`
- Modify: `src-tauri/src/islepilot/mod.rs`
- Modify: `src-tauri/src/events.rs`

**Interfaces:**
- Consumes: all provider adapters and existing IslePilot poller.
- Produces Tauri commands `provider_detect`, `provider_start_login`, `provider_cancel_login`, `provider_state`, `provider_logout`, `provider_select_manual`.
- Produces events `provider://state`, compatible `dino://update`, `position://update`, and new `position://cleared`.

- [ ] **Step 1: Write generation and state-transition tests**

Factor pure state transitions into a testable struct:

```rust
#[test]
fn late_generation_cannot_publish_after_switch() {
    let mut gate = PublishGate::default();
    let era = gate.activate(ProviderId::Era);
    let titan = gate.activate(ProviderId::Titan);
    assert!(!gate.accepts(era, ProviderId::Era));
    assert!(gate.accepts(titan, ProviderId::Titan));
}
```

Test `AuthenticatedOffline` passes startup gate, `LoginRequired` does not, temporary errors preserve selected provider, and switching/logout returns a reset action.

- [ ] **Step 2: Run focused tests and verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml providers::orchestrator::tests`

Expected: compilation fails because orchestrator types do not exist.

- [ ] **Step 3: Implement orchestrator and event clearing**

Maintain an atomic generation plus selected provider in `AppState`. Each worker captures its generation and checks immediately before every publish. Provider switches stop the IslePilot worker, clear the last Dino update, clear tracker current/previous data, begin a trail break, emit `position://cleared`, and then start only the chosen worker.

Gate `clipboard.rs`: automatic provider mode ignores clipboard samples; manual mode uses the existing sequence-number parser. Preserve manual fallback as an explicit selection.

- [ ] **Step 4: Implement provider login windows**

Follow the existing async IslePilot login-window pattern. Use one label `provider-login`, size approximately 620×760, and provider login URLs:

```rust
match provider {
  ProviderId::Era => "https://eragamingvn.net/api/auth/steam",
  ProviderId::Titan => "https://www.therealservervn.com/login?next=/nguoi-choi",
  ProviderId::IslePilot => return islepilot::start_token_login(app).await,
}
```

For Era/Titan, periodically inspect cookies only for the selected exact provider origin, construct the same-origin header, validate it, persist only after validation, and close the login window on success. Closing/canceling increments generation and ends the worker. Remote pages receive no command allowlist.

- [ ] **Step 5: Register commands and route IslePilot compatibility**

Add commands to `generate_handler!` and TypeScript wrappers. Existing IslePilot login functions set active provider to IslePilot after successful validation. Existing `dino://update` stays available during migration, but its payload is generated from the selected provider snapshot so main/minimap consumers do not fork.

- [ ] **Step 6: Run tests and commit**

Run `cargo test --manifest-path src-tauri/Cargo.toml providers::orchestrator::tests`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `npm run check`.

Expected: all Rust unit/workspace tests and frontend checks pass.

Commit Task 4 files with message `feat: route active live-data provider`.

---

### Task 5: Mandatory connection screen and provider-neutral Dino HUD

**Files:**
- Create: `src/main/connection/ConnectionGate.svelte`
- Modify: `src/main/App.svelte`
- Modify: `src/main/dino/DinoTab.svelte`
- Modify: `src/main/garage/GarageTab.svelte`
- Modify: `src/main/fullmap/LayerPanel.svelte`
- Modify: `src/minimap/main.ts`
- Modify: `src/minimap/render.ts`
- Modify: `src/lib/api.ts`
- Modify: `src/lib/i18n/vi.ts`
- Modify: `src/lib/i18n/en.ts`

**Interfaces:**
- Consumes: Task 4 commands/events and normalized player snapshot.
- Produces: startup connection gate, change-connection flow, percentage-capable stat rendering, and feature gating.

- [ ] **Step 1: Add pure frontend helpers**

Create `src/lib/provider-ui.ts` with:

```ts
export function providerAllowsMain(status: ConnectionStatus): boolean {
  return status === "authenticated-online" || status === "authenticated-offline" || status === "temporary-error";
}
export function formatStat(stat: SharedStatBar | null): string {
  if (!stat) return "—";
  return stat.current != null && stat.max != null ? `${stat.current} / ${stat.max}` : `${stat.percent}%`;
}
```

Provider routing decisions remain covered by Rust tests; `svelte-check` verifies component integration.

- [ ] **Step 2: Implement the mandatory gate**

On App mount, call `providerState()` before mounting normal navigation. Render `ConnectionGate` for unconfigured/login-required/logged-out/unsupported. Render a validating state while persisted credentials are checked. Allow main UI for authenticated-online, authenticated-offline, and temporary-error with a visible retry/status message.

The gate workflow is website input → Detect → provider name → Login. Unsupported websites explain that an adapter must be added. Never show a generic success solely because a URL parses.

- [ ] **Step 3: Make Dino view provider-neutral**

Remove IslePilot-specific login controls from normal stats content. Show active provider/server, update age, online/offline status, normalized bars, Prime/mutations when supplied, and `—` with an explanatory capability message when unavailable. Add Change connection, which logs out only after the user invokes it and returns to the gate.

- [ ] **Step 4: Gate provider-only features and clear stale state**

Hide or disable Garage and IslePilot live POI controls for Era/Titan with clear text. On `position://cleared` and player-null provider updates, minimap removes the marker, stats, quests, and previous-provider label immediately. A temporary error may show a bounded cached state only until the backend stale threshold expires.

- [ ] **Step 5: Remove upstream update UI from App**

Delete updater imports/state/check/download banner and launch check from `App.svelte`. The custom build must never offer upstream 2.x as an update.

- [ ] **Step 6: Run frontend verification and commit**

Run `npm run check` and `npm run build`.

Expected: zero Svelte/TypeScript errors and a successful Vite production build. The existing Three.js chunk warning is informational.

Commit Task 5 files with message `feat: require provider connection on startup`.

---

### Task 6: Custom version, Windows build, backup, replacement, and live validation

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock` only if Cargo updates package metadata
- Modify: `src-tauri/tauri.conf.json`
- Modify: `README.md`
- Create: final installer, backup manifest, and validation report in the task `outputs` directory

**Interfaces:**
- Consumes: completed application from Tasks 1–5.
- Produces: an honestly labeled custom NSIS installer and a preserved backup of the old user's settings/data.

- [ ] **Step 1: Configure custom version and updater removal**

Set all package/config versions to `1.6.0-eratitan.1`. Remove updater/process plugins only if no longer used elsewhere, remove updater endpoint/pubkey, and set `createUpdaterArtifacts` to `false`. Keep the existing Windows identifier only if uninstall inspection proves replacement behavior is safe; otherwise use a custom identifier and explicitly remove the prior app through its registered uninstaller.

- [ ] **Step 2: Run full verification**

With local Rust environment variables pointing to `work/toolchains`, run:

```powershell
npm ci
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run tauri build
```

Expected: all checks pass and NSIS installer exists under the configured local cargo target directory. Inspect Authenticode/hash and record unsigned status honestly.

- [ ] **Step 3: Review the complete source diff**

Compare merge base `5c49bfc2265534b242b24b7dd3946f3e570e56db` through HEAD. Verify credential origin isolation, no secret logging, source generation checks, offline/401 distinction, updater removal, donation removal, and feature gating. Fix all critical/important findings and rerun affected tests.

- [ ] **Step 4: Back up user data before uninstall**

Resolve exact installed paths and registered uninstall command through read-only registry checks. Copy `%APPDATA%/TheIsleOverlay`, relevant `%LOCALAPPDATA%/TheIsleOverlay` settings/waypoints/trails, and other confirmed application-owned data into a timestamped directory under task `outputs`. Hash/list copied files in `install-backup-manifest.txt`. Do not include unrelated browser profiles or provider cookies from Chrome/Codex.

- [ ] **Step 5: Replace the old app only after installer success**

Close the old overlay normally, invoke its exact registered uninstaller silently/current-user where supported, confirm the old executable is absent, install the custom NSIS package, and confirm the new executable/version and shortcut. The user already authorized old-app replacement after a successful fix/build; do not uninstall earlier.

- [ ] **Step 6: Validate startup and provider login**

Launch the custom overlay. Confirm it opens the mandatory connection gate with no validated provider, detects exact Era/Titan/IslePilot URLs, rejects a lookalike URL, and opens the correct provider login webview. User performs any credential entry. After login, confirm authenticated-offline reaches the app and authenticated-online updates the Dino/minimap state at the provider cadence. Test provider switching clears old data. Record observed limitations without calling 12/20-second polling sub-second realtime.

- [ ] **Step 7: Commit packaging/docs and deliver artifacts**

Commit repository files with message `build: package multi-provider overlay`. Copy the final installer and a concise test report into the task `outputs` directory, with SHA-256 hashes and backup location. Do not commit local backups, credentials, build outputs, or user-specific identifiers.
