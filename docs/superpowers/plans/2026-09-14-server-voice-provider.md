# Server Voice Provider Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the single-client Voice bridge with a server-neutral provider framework that is safe on a clean Windows account and ready for future server adapters.

**Architecture:** A Rust Voice registry selects a compiled adapter by exact server origin. A manager owns one-time login state, provider-scoped DPAPI sessions, and normalized Voice grants; Svelte renders only the shared status contract. The initial registry is intentionally empty, so unsupported servers fail closed without depending on another desktop application.

**Tech Stack:** Rust, Tauri 2, Windows DPAPI, Serde, reqwest URL parsing, Svelte 5, TypeScript, Node test runner.

**Spec:** `docs/superpowers/specs/2026-09-14-server-voice-provider-design.md`

## Global Constraints

- Voice code and UI must not name or depend on a specific server or external Voice application.
- The initial provider registry contains no production adapter and returns `not-configured` safely.
- Provider matching uses normalized, exact HTTPS origins; suffix and lookalike matches are rejected.
- No private server key, reusable service credential, copied token, or client-attestation emulation may be added.
- Persisted provider sessions are scoped by provider ID and exact origin and protected with Windows DPAPI.
- Voice grants remain in memory and are excluded from settings, logs, crash reports, and frontend localStorage.
- Audio transport, microphone capture, push-to-talk, proximity processing, embedding, and background-client integration remain out of scope.
- Release version is `2.5.0`; distribution is through GitHub Release and the in-app updater, with no Desktop copy.

---

### Task 1: Provider contracts and exact-origin registry

**Files:**
- Create: `src-tauri/src/voice/model.rs`
- Create: `src-tauri/src/voice/provider.rs`
- Create: `src-tauri/src/voice/registry.rs`
- Create: `src-tauri/src/voice/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] Write registry tests for exact HTTPS origin matching, path normalization, lookalike rejection, duplicate origin rejection, and provider-independent string IDs.
- [ ] Run the focused Rust tests and confirm they fail because the Voice framework is absent.
- [ ] Implement serialized models, normalized errors, the `VoiceProvider` trait, and the empty production registry.
- [ ] Run the focused Rust tests and confirm they pass.
- [ ] Commit the provider contract and registry.

### Task 2: Login lifecycle and protected session boundary

**Files:**
- Create: `src-tauri/src/voice/manager.rs`
- Create: `src-tauri/src/voice/session_store.rs`
- Modify: `src-tauri/src/voice/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] Write manager tests with a synthetic adapter and in-memory session store for `not-configured`, login start, callback success, wrong state, callback expiry, callback single use, provider mismatch, session expiry, grant normalization, and logout.
- [ ] Write store tests for provider-and-origin isolation and confirm Voice secrets never enter summary serialization.
- [ ] Run the focused tests and confirm the new lifecycle tests fail.
- [ ] Implement the manager, one-time callback validation, redacted errors, DPAPI session store, and Tauri commands.
- [ ] Route only `islemap-thienvyma://voice/callback` to the Voice callback handler; preserve existing deep links.
- [ ] Run the focused Rust tests and confirm they pass.
- [ ] Commit login lifecycle and secure storage.

### Task 3: Generic Voice UI and dependency removal

**Files:**
- Create: `src/main/voice/VoiceTab.svelte`
- Delete: `src/main/voice/DinoVoiceTab.svelte`
- Delete: `src-tauri/src/dinovoice.rs`
- Modify: `src/main/App.svelte`
- Modify: `src/lib/api.ts`
- Modify: `src/lib/i18n/en.ts`
- Modify: `src/lib/i18n/vi.ts`
- Create: `scripts/voice-provider.test.mjs`
- Modify: `package.json`
- Modify: `.github/workflows/check.yml`
- Modify: `.github/workflows/release.yml`
- Delete: `docs/dinovietnam-voice/`

- [ ] Write a Node source-contract test that rejects external-client discovery/launch code, server-specific Voice copy, service-specific endpoints, and hard-coded Voice service credentials.
- [ ] Run it first and confirm it fails against the current bridge.
- [ ] Replace the frontend types and commands with generic Voice status, login, grant-readiness, logout, and refresh APIs.
- [ ] Replace the old component with a generic status UI for the active server and provider adapter.
- [ ] Remove the old native client detector/launcher and obsolete service-specific research documents.
- [ ] Add the Voice contract test to local scripts and both GitHub workflows.
- [ ] Run the Voice test, Svelte check, frontend build, and focused Rust tests.
- [ ] Commit the generic UI and removed dependency.

### Task 4: Version, release notes, and complete verification

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `CHANGELOG.md`

- [ ] Add the `2.5.0` changelog entry describing the server-neutral Voice provider framework and clean-machine behavior.
- [ ] Update all release manifests and lockfiles to `2.5.0`.
- [ ] Run version consistency, updater, Voice, heading, skin, Garage, installer, release-trust, minimap-friends, provider-defaults, frontend, Worker, forbidden-API, Rust formatting, Clippy, and Rust workspace tests.
- [ ] Review the complete diff against the approved spec and scan for stale server-specific Voice names, external executable paths, copied credentials, placeholders, and accidental secrets.
- [ ] Commit the verified `2.5.0` release change.

### Task 5: Publish and verify the in-app update

**Files:** No source changes expected.

- [ ] Push the branch commit to `origin/main` and create/push tag `v2.5.0`.
- [ ] Wait for the GitHub release workflow to finish successfully.
- [ ] Verify the public release is latest and contains `latest.json`, the NSIS installer, and its updater signature.
- [ ] Download only the small public `latest.json`, run the updater checker against it, and confirm its Windows URL targets the `v2.5.0` installer.
- [ ] Confirm the local working tree is clean and `origin/main`, the release tag, and local `HEAD` resolve to the same commit.
- [ ] Schedule Windows shutdown after reporting the verified release to the user.
