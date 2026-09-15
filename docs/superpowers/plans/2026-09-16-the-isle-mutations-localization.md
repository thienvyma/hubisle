# The Isle Mutations Localization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an in-hub **VIỆT HOÁ THE ISLE MUTATIONS** installer that keeps Mutation names in English and translates only in-game Mutation descriptions to Vietnamese.

**Architecture:** Keep all v2.6.0 hub Mutation UI unchanged. Add a native Rust localization service that detects the Steam install, reads the installed English Unreal `.locres`, replaces only exact known Mutation description strings, writes a Vietnamese `.locres` copy under a hub-owned localization target, and updates the user's The Isle culture setting to `vi` while preserving/restoring the previous value. Expose narrow Tauri commands and one Svelte card in the Dino tab.

**Tech Stack:** Tauri 2, Rust 2021, Svelte 5, TypeScript, Unreal Engine LocRes binary format, Steam VDF/ACF metadata, Node test runner, GitHub Actions Windows CI.

**Spec:** `docs/superpowers/specs/2026-09-16-the-isle-mutations-localization-design.md`

## Global Constraints

- Keep every existing v2.6.0 Mutation feature in the hub.
- UI title must be exactly **VIỆT HOÁ THE ISLE MUTATIONS**.
- Canonical Mutation names remain English everywhere in the installed game pack.
- Translate only Mutation description/detail strings.
- No EXE patching, memory modification, DLL injection, EAC bypass, server-traffic modification, or original game asset overwrite.
- Installer writes only to a validated The Isle client installation and `%LOCALAPPDATA%\\TheIsle\\Saved\\Config\\WindowsClient\\GameUserSettings.ini`.
- Uninstall removes only files created by this feature and restores the previous culture value when it is still safe to do so.
- Do not rewrite or retag `v2.6.0`; ship later as a new version.

---

## File Structure

- Create `src-tauri/src/mutation_locale/locres.rs` — minimal Unreal LocRes reader/writer for versions 1–3; exposes string-table replacement without translating keys/names.
- Create `src-tauri/src/mutation_locale/catalog.rs` — exact English Mutation description → Vietnamese description mapping; no Vietnamese Mutation-name field.
- Create `src-tauri/src/mutation_locale/steam.rs` — Steam library/appmanifest resolution for app id `376210` and strict The Isle path validation.
- Create `src-tauri/src/mutation_locale/mod.rs` — install/status/uninstall orchestration, ownership manifest, culture config patching, public Tauri commands.
- Create `src/lib/components/MutationGameLocalization.svelte` — hub card and install/update/uninstall UX.
- Modify `src-tauri/src/lib.rs` — register module + three commands.
- Modify `src/lib/api.ts` — typed IPC contract.
- Modify `src/main/dino/DinoTab.svelte` — mount the new card below the existing Mutation library.
- Modify `src/lib/i18n/vi.ts` and `src/lib/i18n/en.ts` — button/status/error copy; title remains exactly Vietnamese in both locales.
- Create `scripts/mutation-localization-ui.test.mjs` — static contract test for exact title, API surface, and keeping existing Mutation components.
- Modify `package.json`, `.github/workflows/check.yml`, `.github/workflows/release.yml` — run the new UI contract test.
- Modify `THIRD_PARTY_NOTICES.md` — credit MIT reference implementations/data sources used to derive the LocRes format and source description catalog.

---

### Task 1: Unreal LocRes transformation engine

**Files:**
- Create: `src-tauri/src/mutation_locale/locres.rs`
- Create: `src-tauri/src/mutation_locale/catalog.rs`

**Interfaces:**
- Produces `pub fn patch_locres(input: &[u8], replacements: &HashMap<&str, &str>) -> Result<PatchResult, String>`.
- Produces `pub struct PatchResult { pub bytes: Vec<u8>, pub replaced: usize, pub matched_sources: Vec<String> }`.
- Produces `pub fn description_replacements() -> HashMap<&'static str, &'static str>` containing description translations only.

- [ ] **Step 1: Write failing Rust tests in `locres.rs`**

Tests construct a tiny version-2/3 LocRes fixture with entries `Cellular Regeneration` and `Recovers health slightly faster`, patch only the description, then assert:

```rust
assert_eq!(roundtrip.values()[0], "Cellular Regeneration");
assert_eq!(roundtrip.values()[1], "Hồi máu nhanh hơn một chút.");
assert_eq!(result.replaced, 1);
```

Also assert malformed magic/version and truncated data are rejected.

- [ ] **Step 2: Run the focused Rust tests and verify RED**

Run in CI/workspace:

```powershell
cd src-tauri
cargo test mutation_locale::locres --locked
```

Expected: FAIL because the module/functions do not exist yet.

- [ ] **Step 3: Implement minimal LocRes read/write support**

Implement Unreal string helpers and LocRes v1–v3 parsing/writing based on the public format:

```rust
const MAGIC: [u8; 16] = [0x0E,0x14,0x74,0x75,0x67,0x4A,0x03,0xFC,0x4A,0x15,0x90,0x9D,0xC3,0x37,0x7F,0x1B];

pub fn patch_locres(input: &[u8], replacements: &HashMap<&str, &str>) -> Result<PatchResult, String> {
    let mut file = LocresFile::parse(input)?;
    let mut replaced = 0;
    for namespace in &mut file.namespaces {
        for entry in &mut namespace.entries {
            if let Some(next) = replacements.get(entry.value.as_str()) {
                entry.value = (*next).to_owned();
                replaced += 1;
            }
        }
    }
    Ok(PatchResult { bytes: file.write()?, replaced, matched_sources: /* exact matches */ })
}
```

Do not alter namespace/key/source-hash fields. For version 3 preserve version 3 and recompute CityHash-derived namespace/key hashes when writing; for version 2 use CRC32 hashes; version 1 has no pre-hashed keys. Add `crc32fast` only if the existing dependency set cannot provide CRC32 cleanly.

- [ ] **Step 4: Add the description-only catalog**

Use the current in-game English source-description strings as exact match keys. Store only fields needed by the pack:

```rust
pub struct MutationDescription {
    pub name_en: &'static str,
    pub source: &'static str,
    pub vi: &'static str,
}
```

`name_en` is diagnostics only and is never passed to `patch_locres` as a replacement key. Exclude Vietnamese name fields entirely.

- [ ] **Step 5: Run focused + workspace tests and verify GREEN**

```powershell
cd src-tauri
cargo test mutation_locale::locres --locked
cargo test --workspace --locked
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/mutation_locale/locres.rs src-tauri/src/mutation_locale/catalog.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add mutation locres transformer"
```

---

### Task 2: Steam detection, install state, culture ownership, and safe uninstall

**Files:**
- Create: `src-tauri/src/mutation_locale/steam.rs`
- Create/modify: `src-tauri/src/mutation_locale/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces `pub enum MutationLocaleState { GameNotFound, NotInstalled, Installed, UpdateAvailable, Incompatible, Corrupt, GameRunning }`.
- Produces serializable `pub struct MutationLocaleStatus { state, game_path, pack_version, matched, total, message }`.
- Produces Tauri commands `mutation_locale_status`, `mutation_locale_install`, `mutation_locale_uninstall`.

- [ ] **Step 1: Write failing path/manifest/config tests**

Create fixture tests for:

```rust
assert_eq!(parse_steam_library_paths(vdf).len(), 2);
assert_eq!(parse_install_dir(acf), Some("The Isle".into()));
assert!(validate_game_root(&fixture_root).is_ok());
assert!(validate_game_root(&fixture_root.join(".." )).is_err());
```

Add tests that culture patching adds/replaces exactly one `[Internationalization]` `Culture=vi`, stores the prior value in the ownership manifest, and uninstall restores only that value if it has not been changed by the user afterward.

- [ ] **Step 2: Verify RED with focused Rust tests**

```powershell
cd src-tauri
cargo test mutation_locale --locked
```

Expected: FAIL before the orchestration exists.

- [ ] **Step 3: Implement Steam discovery**

Detection order:

1. Steam install roots from Windows registry when available, plus default `%ProgramFiles(x86)%\\Steam` / `%ProgramFiles%\\Steam` candidates.
2. Parse `steamapps/libraryfolders.vdf`.
3. For every library, parse `steamapps/appmanifest_376210.acf` and its `installdir`.
4. Validate `<library>\\steamapps\\common\\<installdir>` by requiring The Isle client directory markers and at least one English `.locres` under the game localization tree.

No path accepted from metadata may escape the candidate Steam library root after canonicalization.

- [ ] **Step 4: Implement generated pack installation**

At install time:

```text
validated game root
  -> enumerate English .locres files under The Isle localization roots
  -> patch exact description values using catalog.rs
  -> require a minimum verified match count (>0; production target records exact count)
  -> write patched files to corresponding vi culture paths using temp + atomic rename
  -> verify output by parsing it again and confirming Vietnamese descriptions exist
  -> write ownership.json under %LOCALAPPDATA%/islemap-thienvyma-data/mutation-locale/
  -> set GameUserSettings.ini [Internationalization] Culture=vi
```

Never overwrite the English `.locres` source files.

- [ ] **Step 5: Implement status and uninstall**

Status derives from ownership manifest + file hashes + current game/source fingerprint. Uninstall removes only paths recorded in `ownership.json`, then removes empty hub-created directories and conditionally restores the previous culture value.

Before install/uninstall, refuse while `win::game_window::find_game_window(settings::GAME_PROCESS_NAME)` returns a window.

- [ ] **Step 6: Register commands and verify GREEN**

Add:

```rust
pub mod mutation_locale;
```

and handlers:

```rust
mutation_locale::mutation_locale_status,
mutation_locale::mutation_locale_install,
mutation_locale::mutation_locale_uninstall,
```

Run:

```powershell
cd src-tauri
cargo test mutation_locale --locked
cargo test --workspace --locked
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/mutation_locale src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: install mutation localization into The Isle"
```

---

### Task 3: Typed API and in-hub installer card

**Files:**
- Create: `src/lib/components/MutationGameLocalization.svelte`
- Create: `scripts/mutation-localization-ui.test.mjs`
- Modify: `src/lib/api.ts`
- Modify: `src/main/dino/DinoTab.svelte`
- Modify: `src/lib/i18n/vi.ts`
- Modify: `src/lib/i18n/en.ts`
- Modify: `package.json`

**Interfaces:**
- Frontend `MutationLocaleStatus` mirrors the Rust serialized shape.
- `mutationLocaleStatus()`, `mutationLocaleInstall()`, `mutationLocaleUninstall()` call the three native commands.

- [ ] **Step 1: Write failing Node contract test**

Test exact title and preservation of current hub Mutation components:

```js
assert.match(card, /VIỆT HOÁ THE ISLE MUTATIONS/);
assert.match(dinoTab, /<MutationLibrary\s*\/>/);
assert.match(dinoTab, /<MutationGameLocalization\s*\/>/);
assert.match(api, /mutation_locale_install/);
assert.match(api, /mutation_locale_uninstall/);
```

- [ ] **Step 2: Run and verify RED**

```powershell
npm run test:mutation-locale-ui
```

Expected: FAIL because the component/API are absent.

- [ ] **Step 3: Add typed IPC API**

```ts
export type MutationLocaleState =
  | "game-not-found" | "not-installed" | "installed" | "update-available"
  | "incompatible" | "corrupt" | "game-running";

export interface MutationLocaleStatus {
  state: MutationLocaleState;
  gamePath: string | null;
  packVersion: string;
  matched: number;
  total: number;
  message: string | null;
}

export const mutationLocaleStatus = () => invoke<MutationLocaleStatus>("mutation_locale_status");
export const mutationLocaleInstall = () => invoke<MutationLocaleStatus>("mutation_locale_install");
export const mutationLocaleUninstall = () => invoke<MutationLocaleStatus>("mutation_locale_uninstall");
```

- [ ] **Step 4: Build the card**

Component behavior:

- `onMount` loads status.
- Primary button shows `CÀI VIỆT HOÁ`, `CẬP NHẬT`, or disabled installed state based on status.
- Installed state exposes `GỠ VIỆT HOÁ`.
- `game-running` tells the user to close The Isle and retry.
- Show detected game path and `matched/total` coverage.
- Errors stay within the card; no unhandled rejection.

Title is literal exact copy:

```svelte
<h3>VIỆT HOÁ THE ISLE MUTATIONS</h3>
```

- [ ] **Step 5: Mount card without removing current Mutation UI**

In `DinoTab.svelte`, import `MutationGameLocalization` and render it immediately after `<MutationLibrary />`.

- [ ] **Step 6: Run frontend checks and verify GREEN**

```powershell
npm run test:mutation-locale-ui
npm run check
npm run build
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/lib/api.ts src/lib/components/MutationGameLocalization.svelte src/main/dino/DinoTab.svelte src/lib/i18n/vi.ts src/lib/i18n/en.ts scripts/mutation-localization-ui.test.mjs package.json
git commit -m "feat: add in-hub mutation localization installer"
```

---

### Task 4: CI, notices, and release guardrails

**Files:**
- Modify: `.github/workflows/check.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `THIRD_PARTY_NOTICES.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Wire the new test into CI/release**

Add after `test:mutations`:

```yaml
- run: npm run test:mutation-locale-ui
```

and equivalent named release step.

- [ ] **Step 2: Add attribution**

Document that LocRes format understanding was cross-checked against `akintos/UnrealLocres` (MIT) and exact English source-description wording was cross-checked against `klong-dev/IsleLiveMap` (MIT). Do not copy or ship Vietnamese Mutation names.

- [ ] **Step 3: Add Unreleased changelog entry**

State that the hub now offers **VIỆT HOÁ THE ISLE MUTATIONS**, installs a data-only localization pack, preserves English Mutation names, and can uninstall cleanly.

- [ ] **Step 4: Run full CI commands**

```powershell
npm ci
npm run check
npm run test:heading
npm run test:skin
npm run test:garage
npm run test:installer
npm run test:release-trust
npm run test:minimap-friends
npm run test:provider-defaults
npm run test:voice
npm run test:mutations
npm run test:mutation-locale-ui
npm run test:updater
npm run build
cd src-tauri
cargo test --workspace --locked
```

Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/check.yml .github/workflows/release.yml THIRD_PARTY_NOTICES.md CHANGELOG.md
git commit -m "chore: validate mutation localization release"
```

---

### Task 5: Pull request and verification

**Files:** none beyond prior tasks.

- [ ] **Step 1: Open PR from `feature/the-isle-mutations-localization` to `main`**

PR summary must explicitly state that v2.6.0 hub Mutation UI is retained.

- [ ] **Step 2: Wait for Windows CI and inspect every failed job if any**

Do not merge while CI is red.

- [ ] **Step 3: Verify PR diff scope**

Confirm no changes patch `TheIsleClient-Win64-Shipping.exe`, no DLL/proxy/injection code, no EAC bypass, and no deletion of `MutationDetails.svelte`, `MutationLibrary.svelte`, or `src/lib/mutations.ts`.

- [ ] **Step 4: Only after all checks are green, present merge/release options**

Do not create or move a release tag until the feature is merged and a new version is intentionally chosen.
