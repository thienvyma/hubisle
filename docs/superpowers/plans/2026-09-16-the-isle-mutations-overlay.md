# The Isle Mutations Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the failed Mutation `.locres` installer with a safe OCR-driven click-through overlay that displays Vietnamese Mutation descriptions from the hub catalog while keeping native English names unchanged.

**Architecture:** A Windows-only native worker finds the existing The Isle window, captures only its visible client pixels, runs on-device `Windows.Media.Ocr`, and matches the selected detail description against current English detection signatures. It emits only the canonical English Mutation name to a tiny dedicated Tauri overlay page; that page imports `src/lib/mutations.ts` and renders the existing Vietnamese description.

**Tech Stack:** Tauri 2, Rust 2021, `windows` crate (Win32 GDI + WinRT OCR), Svelte/TypeScript, Vite multi-page build.

**Spec:** `docs/superpowers/specs/2026-09-16-the-isle-mutations-overlay-design.md`

## Global Constraints

- Settings title is exactly **VIỆT HOÁ THE ISLE MUTATIONS**.
- Vietnamese text comes from `src/lib/mutations.ts`; Rust contains no Vietnamese translation catalog.
- Native The Isle Mutation names remain English.
- No EXE patching, `OpenProcess`, game-memory read/write, DLL injection, IoStore decryption, hooks, or EAC manipulation.
- Capture/OCR runs only when enabled and The Isle is foreground.
- Captured pixels remain local and are never persisted or uploaded.
- Existing minimap and all v2.6.0 hub behavior remain intact.

---

### Task 1: Pure OCR-text Mutation detector

**Files:**
- Create: `src-tauri/src/mutation_overlay/detect.rs`
- Create: `src-tauri/src/mutation_overlay/mod.rs`

**Interfaces:**
- `detect::Detection { name_en: &'static str, confidence: f32 }`
- `detect::detect_mutation(ocr_text: &str) -> Option<Detection>`
- The signature table stores only `name_en` plus the current in-game English detail string.

- [ ] **Step 1: Write failing detector tests**

Cover exact description, case/punctuation variation, one-word OCR damage, and false-positive rejection for a screen containing only Mutation names.

- [ ] **Step 2: Run focused Rust tests and verify RED**

```powershell
cd src-tauri
cargo test mutation_overlay::detect --locked
```

Expected: fail because the module/API does not exist.

- [ ] **Step 3: Implement normalization and conservative matching**

Normalize to lowercase alphanumeric tokens, prefer exact normalized containment, otherwise score token overlap. Require a minimum score and a margin over the runner-up; do not accept name-only matches.

- [ ] **Step 4: Re-run focused tests and verify GREEN**

```powershell
cargo test mutation_overlay::detect --locked
```

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/mutation_overlay
git commit -m "feat: detect mutation details from OCR text"
```

---

### Task 2: Windows game capture + on-device OCR

**Files:**
- Create: `src-tauri/src/mutation_overlay/capture.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- `capture::recognize_game_text(hwnd: isize) -> Result<String, String>`
- Uses existing `win::game_window::client_rect_on_screen` externally; does not open the game process.

- [ ] **Step 1: Add compile-time/unit tests for capture helpers**

Test pure helpers that bound OCR dimensions and convert BGRX rows into top-down BGRA buffers without touching a live window.

- [ ] **Step 2: Run tests and verify RED**

```powershell
cargo test mutation_overlay::capture --locked
```

Expected: fail before implementation.

- [ ] **Step 3: Add required Windows features**

Extend the existing `windows = 0.62` feature list with WinRT OCR/imaging/buffer support and GDI APIs required by capture.

- [ ] **Step 4: Implement GDI client-area capture**

Use screen DC + compatible DC/bitmap + `BitBlt` + `GetDIBits`. Capture only the already-visible client rectangle, scale large windows to a bounded width before OCR, and release every GDI object on every path.

- [ ] **Step 5: Implement Windows.Media.Ocr**

Create a `SoftwareBitmap` from the captured BGRA buffer and call `OcrEngine::TryCreateFromLanguage(en-US)` when supported, falling back to `TryCreateFromUserProfileLanguages`. Run `RecognizeAsync(...).get()` on the worker thread.

- [ ] **Step 6: Verify GREEN**

```powershell
cargo test mutation_overlay --locked
cargo check --locked
```

Expected: pass on Windows CI.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/mutation_overlay/capture.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add local Windows OCR for mutation overlay"
```

---

### Task 3: Dedicated Tauri overlay supervisor

**Files:**
- Modify: `src-tauri/src/mutation_overlay/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- `mutation_overlay::create(app: &AppHandle) -> tauri::Result<()>`
- Event payload `mutation-overlay://detected`: `{ nameEn: String, confidence: f32 }`
- Event `mutation-overlay://cleared` hides stale content.

- [ ] **Step 1: Write failing state-transition tests**

Test the pure supervisor decision helper: disabled/background/no-match => hidden; enabled+foreground+match => shown; stale match => hidden.

- [ ] **Step 2: Verify RED**

```powershell
cargo test mutation_overlay::tests --locked
```

- [ ] **Step 3: Implement `mutation-overlay` window**

Build a hidden transparent, decorationless, always-on-top, skip-taskbar, non-focusable 520x132 webview using `mutation-overlay.html`. Apply existing native overlay styles and click-through behavior.

- [ ] **Step 4: Implement worker loop**

When `mutation_overlay.enabled` is true, find the game window by existing process-name logic. OCR only while it is foreground and not minimized. Emit a detected event only on mutation changes; hide/clear when no confident detail remains.

- [ ] **Step 5: Anchor bottom-center**

Use `client_rect_on_screen` and move the overlay to centered X and a bottom margin inside the game client. Re-assert topmost while visible.

- [ ] **Step 6: Register from app setup**

Add `pub mod mutation_overlay;` and call `mutation_overlay::create(app.handle())?` independently of the minimap.

- [ ] **Step 7: Verify GREEN**

```powershell
cargo test mutation_overlay --locked
cargo test --workspace --locked
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/mutation_overlay src-tauri/src/lib.rs
git commit -m "feat: supervise mutation translation overlay"
```

---

### Task 4: Tiny overlay page using the hub translation catalog

**Files:**
- Create: `mutation-overlay.html`
- Create: `src/mutation-overlay/main.ts`
- Modify: `vite.config.ts`
- Create: `scripts/mutation-overlay-ui.test.mjs`

**Interfaces:**
- Frontend listens for `mutation-overlay://detected` and `mutation-overlay://cleared`.
- It calls `mutationDisplay(payload.nameEn, "vi")`; no Vietnamese translation table is duplicated.

- [ ] **Step 1: Write failing Node contract test**

Assert the overlay entry imports `mutationDisplay` from `$lib/mutations`, listens for the two native events, and Vite includes `mutationOverlay` as a separate entry.

- [ ] **Step 2: Verify RED**

```powershell
node scripts/mutation-overlay-ui.test.mjs
```

- [ ] **Step 3: Implement minimal overlay page**

Render a compact card with English Mutation name as a small heading and Vietnamese description as the primary text. Page background remains fully transparent; pointer events are disabled.

- [ ] **Step 4: Add Vite entry**

Add `mutationOverlay: mutation-overlay.html` beside `main` and `minimap`.

- [ ] **Step 5: Verify GREEN**

```powershell
node scripts/mutation-overlay-ui.test.mjs
npm run check
npm run build
```

- [ ] **Step 6: Commit**

```bash
git add mutation-overlay.html src/mutation-overlay/main.ts vite.config.ts scripts/mutation-overlay-ui.test.mjs
git commit -m "feat: render Vietnamese mutation overlay"
```

---

### Task 5: Replace installer Settings UI with runtime toggle

**Files:**
- Modify: `src-tauri/src/settings.rs`
- Modify: `src/lib/api.ts`
- Modify: `src/lib/components/MutationGameLocalization.svelte`
- Modify: `scripts/mutation-localization-ui.test.mjs`

**Interfaces:**
- `Settings.mutation_overlay = { enabled: boolean; ocr_interval_ms: number; opacity: number }`
- Settings component uses existing `getSettings`, `onSettingsChanged`, and `patchSettings`; it no longer invokes installer commands.

- [ ] **Step 1: Update UI contract test first**

Assert exact title, `BẬT VIỆT HOÁ`, `TẮT VIỆT HOÁ`, use of `patchSettings`, and absence of `mutation_locale_install`/`mutation_locale_uninstall` in the component.

- [ ] **Step 2: Verify RED**

```powershell
npm run test:mutation-locale-ui
```

- [ ] **Step 3: Add default settings/type**

Default OFF with `ocr_interval_ms: 850` and `opacity: 0.94`.

- [ ] **Step 4: Rewrite the existing Settings card**

The card explains that it reads visible pixels locally and renders a separate overlay. Button toggles `mutation_overlay.enabled`. Remove game path, pack version, coverage, install/uninstall/update actions.

- [ ] **Step 5: Remove old installer IPC registration from `lib.rs`**

Keep old module files temporarily only if needed for clean migration during the same branch; no old command is reachable from production UI.

- [ ] **Step 6: Verify GREEN**

```powershell
npm run test:mutation-locale-ui
npm run check
npm run build
cargo test --workspace --locked
```

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/settings.rs src/lib/api.ts src/lib/components/MutationGameLocalization.svelte scripts/mutation-localization-ui.test.mjs src-tauri/src/lib.rs
git commit -m "refactor: replace mutation installer with overlay toggle"
```

---

### Task 6: Retire obsolete localization installer and release guardrails

**Files:**
- Delete: obsolete runtime code under `src-tauri/src/mutation_locale/` after signatures have moved to `mutation_overlay/detect.rs`
- Delete: `src/lib/mutation-locale-api.ts`
- Modify: `.github/workflows/check.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `CHANGELOG.md`
- Modify: `THIRD_PARTY_NOTICES.md` if Windows OCR/reference attribution requires it

- [ ] **Step 1: Add CI test command**

Run `node scripts/mutation-overlay-ui.test.mjs` in check and release workflows.

- [ ] **Step 2: Extend safety/static guards**

Ensure source contains no `OpenProcess`, `ReadProcessMemory`, `WriteProcessMemory`, injection, EAC bypass, or IoStore decryption implementation for this feature.

- [ ] **Step 3: Remove obsolete installer code/API**

Delete unreferenced `.locres`, public-pack, install/uninstall, culture-editing implementation and frontend IPC wrapper.

- [ ] **Step 4: Update changelog**

Document that **VIỆT HOÁ THE ISLE MUTATIONS** is now a local OCR overlay and does not modify The Isle files.

- [ ] **Step 5: Run full verification**

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
node scripts/mutation-overlay-ui.test.mjs
npm run test:updater
npm run build
cd src-tauri
cargo test --workspace --locked
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "chore: retire mutation locres installer"
```

---

### Task 7: Windows smoke-test handoff

**Files:** none.

- [ ] **Step 1: Confirm GitHub Windows CI is green**
- [ ] **Step 2: Confirm PR diff contains no forbidden game-process access or executable modification**
- [ ] **Step 3: Have the user pull the branch and run `npm.cmd run tauri dev`**
- [ ] **Step 4: In Settings enable `VIỆT HOÁ THE ISLE MUTATIONS`, open the Mutation detail screen, and verify the correct Vietnamese description appears while the English name remains native**
- [ ] **Step 5: Tune OCR matching/capture region only from real-game evidence; do not weaken confidence thresholds blindly**
