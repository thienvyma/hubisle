# The Isle Mutation Description Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the failed Mutation `.locres` installer with a safe, screen-based overlay that visually replaces the selected Mutation description in The Isle while leaving the game process and files untouched.

**Architecture:** A separate transparent Tauri window is anchored to a normalized rectangle inside the The Isle client area. A low-cadence supervisor captures only that rectangle into RAM, hashes the frame, invokes Windows OCR only when pixels changed, matches recognized English text against the existing Mutation catalog, and emits a small payload containing the canonical English name plus the Vietnamese `descriptionVi`. Manual selection and calibration remain available when capture/OCR fails.

**Tech Stack:** Tauri 2, Rust 2021, `windows` crate Win32 + WinRT APIs, Svelte 5, TypeScript, Vite, Node test runner.

**Spec:** `docs/superpowers/specs/2026-09-16-the-isle-mutation-description-overlay-design.md`

## Global Constraints

- `src/lib/mutations.ts` is the only hand-written Vietnamese translation source of truth.
- Canonical Mutation names remain English; only the visible description is replaced.
- Never use `OpenProcess`, `ReadProcessMemory`, `WriteProcessMemory`, DLL injection, executable patching, EAC bypass/tampering, IoStore/Pak decryption, or synthetic game network traffic.
- Capture stays in RAM; no screenshots are written to disk.
- Default recognition cadence is 2 attempts/second maximum, and OCR only runs when the captured frame changes materially.
- No capture/OCR while the feature is disabled or The Isle is not foreground.
- Manual selection must remain usable if automatic recognition is unavailable.
- Existing Mutation Library and every unrelated Hub feature must remain intact.

---

### Task 1: Replace installer UI contract with overlay settings and typed API

**Files:**
- Modify: `src-tauri/src/settings.rs`
- Modify: `src/lib/api.ts`
- Modify: `src/lib/components/MutationGameLocalization.svelte`
- Modify: `src/main/settings/Settings.svelte`
- Modify: `scripts/mutation-localization-ui.test.mjs`
- Modify: `package.json`

**Interfaces:**
- Produces settings key:
  ```ts
  mutation_overlay: {
    enabled: boolean;
    auto_detect: boolean;
    rect: { x: number; y: number; w: number; h: number };
    confidence_threshold: number;
  }
  ```
- Produces frontend API types `MutationOverlayStatus`, `MutationOverlayPayload` and command wrappers for status/manual/calibration/preview.

- [ ] **Step 1: Rewrite the UI test to describe overlay semantics and fail against the current installer UI**

```js
assert.match(card, /VIỆT HOÁ THE ISLE MUTATIONS/);
assert.match(card, /BẬT VIỆT HOÁ MUTATIONS/);
assert.match(card, /CĂN CHỈNH VỊ TRÍ/);
assert.match(card, /HIỆN THỬ/);
assert.doesNotMatch(card, /CÀI VIỆT HOÁ|GỠ VIỆT HOÁ|GÓI DỊCH|v1\./);
assert.match(api, /mutation_overlay_status/);
assert.match(api, /mutation_overlay_set_manual/);
assert.match(api, /mutation_overlay_begin_calibration/);
```

- [ ] **Step 2: Run the targeted test and confirm it fails**

Run: `npm.cmd run test:mutation-overlay-ui`

Expected: FAIL because the current component still contains installer semantics and the overlay API does not exist.

- [ ] **Step 3: Add default settings**

Add to `default_settings()`:

```rust
"mutation_overlay": {
    "enabled": false,
    "auto_detect": true,
    "rect": { "x": 0.61, "y": 0.28, "w": 0.28, "h": 0.22 },
    "confidence_threshold": 0.82,
},
```

- [ ] **Step 4: Extend `Settings` and narrow IPC types in `src/lib/api.ts`**

```ts
export interface NormalizedRect { x: number; y: number; w: number; h: number }
export interface MutationOverlayStatus {
  state: "disabled" | "waiting-game" | "recognizing" | "recognized" | "manual" | "needs-calibration" | "capture-unavailable" | "ocr-unavailable";
  nameEn: string | null;
  confidence: number | null;
  message: string | null;
}
export interface MutationOverlayPayload {
  nameEn: string;
  descriptionVi: string;
  confidence: number | null;
  source: "auto" | "manual" | "preview";
}
```

Command wrappers call:

```ts
invoke("mutation_overlay_status")
invoke("mutation_overlay_set_manual", { nameEn })
invoke("mutation_overlay_clear_manual")
invoke("mutation_overlay_begin_calibration")
invoke("mutation_overlay_save_calibration")
invoke("mutation_overlay_cancel_calibration")
invoke("mutation_overlay_preview", { nameEn })
```

- [ ] **Step 5: Rewrite `MutationGameLocalization.svelte` as the settings controller**

Required visible controls/copy:

```text
VIỆT HOÁ THE ISLE MUTATIONS
BẬT VIỆT HOÁ MUTATIONS
TỰ ĐỘNG NHẬN DIỆN
CĂN CHỈNH VỊ TRÍ
HIỆN THỬ
```

Use `searchMutations()` from `src/lib/mutations.ts` for manual search. Enabling/disabling flows through `patchSettings({ mutation_overlay: { enabled } })`.

- [ ] **Step 6: Rename the package test script**

```json
"test:mutation-overlay-ui": "node --test scripts/mutation-localization-ui.test.mjs"
```

Remove the obsolete `test:mutation-locale-ui` script name.

- [ ] **Step 7: Run UI checks**

Run:

```powershell
npm.cmd run test:mutation-overlay-ui
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/settings.rs src/lib/api.ts src/lib/components/MutationGameLocalization.svelte src/main/settings/Settings.svelte scripts/mutation-localization-ui.test.mjs package.json
git commit -m "feat: replace mutation installer settings with overlay controls"
```

---

### Task 2: Make Mutation catalog reproducible for native detection

**Files:**
- Create: `scripts/generate-mutation-overlay-catalog.mjs`
- Create: `src-tauri/src/mutation_overlay/catalog.generated.json`
- Create: `scripts/mutation-overlay-catalog.test.mjs`
- Modify: `package.json`
- Modify: `src-tauri/src/mutation_overlay/mod.rs`

**Interfaces:**
- Produces generated records:
  ```json
  { "nameEn": "Cellular Regeneration", "descriptionVi": "Hồi phục máu nhanh hơn 15%.", "matchTexts": ["Recover health 15% faster."] }
  ```
- `matchTexts` may include observed English aliases, but `descriptionVi` is copied only from `src/lib/mutations.ts`.

- [ ] **Step 1: Add a failing drift test**

The test regenerates the catalog in memory and byte-compares it with the committed JSON. It also asserts exactly 42 unique `nameEn` entries and that no Vietnamese string is hand-written in Rust source.

- [ ] **Step 2: Run the drift test and confirm failure because no generator/artifact exists**

Run: `npm.cmd run test:mutation-overlay-catalog`

- [ ] **Step 3: Implement the Node generator using `MUTATION_CATALOG` from `src/lib/mutations.ts`**

The generator imports the TypeScript module under Node's strip-types support, sorts by `nameEn`, and writes deterministic two-space-indented JSON with a trailing newline.

- [ ] **Step 4: Seed observed current-game English aliases**

At minimum include aliases already verified from the user's binary scan, e.g.:

```js
const GAME_ALIASES = {
  "Cellular Regeneration": ["Recovers health slightly faster"],
  "Advanced Gestation": ["Faster Egg Gestation/Incubation/Cooldown Rate"],
  "Sustained Hydration": ["Your water drains more slowly"],
  "Efficient Digestion": ["Your food drains more slowly"],
  "Featherweight": ["Your footprints fade much faster"],
  "Osteosclerosis": ["Resist or Reduce Fracture damage"],
  "Wader": ["Less hindered when wading through shallow water"],
  "Epidermal Fibrosis": ["Increase bleed resistance"],
  "Congenital Hypoalgesia": ["Reduce incoming damage when fighting larger species"],
  "Photosynthetic Tissue": ["Faster health / locked health recovery during the day"],
  "Nocturnal": ["Faster health / locked health recovery during the night"],
  "Hydro-regenerative": ["Recover health faster during rain"],
  "Increased Inspiratory Capacity": ["Increased O2 capacity"],
  "Hydrodynamic": ["Increased swimming speed"],
  "Submerged Optical Retention": ["Increased underwater vision range"],
  "Reabsorption": ["Recover a small amount of water during the rainy weather or while swimming in drinkable water"],
  "Enhanced Digestion": ["Decrease nutrition decay rate"],
  "Reinforced Tendons": ["Jumping costs less stamina"],
  "Reniculate Kidneys": ["Can drink saltwater, no value"],
  "Multichambered Lungs": ["Reduce stamina regeneration threshold"],
  "Infrasound Communication": ["Make significantly less noise when talking in chat"]
};
```

Every catalog entry also includes its existing `descriptionEn` as a match text.

- [ ] **Step 5: Generate and validate the artifact**

Run:

```powershell
node --experimental-strip-types scripts/generate-mutation-overlay-catalog.mjs
npm.cmd run test:mutation-overlay-catalog
```

Expected: PASS with 42 entries.

- [ ] **Step 6: Commit**

```bash
git add scripts/generate-mutation-overlay-catalog.mjs scripts/mutation-overlay-catalog.test.mjs src-tauri/src/mutation_overlay/catalog.generated.json src-tauri/src/mutation_overlay/mod.rs package.json
git commit -m "feat: generate native mutation overlay catalog from TypeScript"
```

---

### Task 3: Implement OCR-tolerant detector and confidence gate

**Files:**
- Replace: `src-tauri/src/mutation_overlay/detect.rs`
- Create: `src-tauri/src/mutation_overlay/catalog.rs`
- Modify: `src-tauri/src/mutation_overlay/mod.rs`

**Interfaces:**
- Produces:
  ```rust
  pub struct DetectedMutation { pub name_en: String, pub description_vi: String, pub confidence: f32 }
  pub fn detect_mutation(ocr_text: &str, threshold: f32) -> Option<DetectedMutation>
  ```

- [ ] **Step 1: Keep the existing detector tests and add ambiguity/threshold tests**

Add tests for `rn` vs `m`, `0` vs `O`, several list names without detail, unrelated UI text, and ambiguous short text returning `None`.

- [ ] **Step 2: Run Rust detector tests and confirm failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml mutation_overlay::detect -- --nocapture`

- [ ] **Step 3: Implement normalization**

Normalize to lowercase ASCII-ish text, collapse punctuation/whitespace, and normalize common OCR confusions only for scoring (`rn`/`m`, `0`/`o`, `1`/`l`).

- [ ] **Step 4: Implement weighted scoring**

Require selected-detail evidence: a name-only match cannot exceed the threshold. Score name similarity plus the best description/alias similarity; reject ties within 0.03 confidence.

- [ ] **Step 5: Run detector tests**

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/mutation_overlay/detect.rs src-tauri/src/mutation_overlay/catalog.rs src-tauri/src/mutation_overlay/mod.rs
git commit -m "feat: detect selected mutation from noisy OCR text"
```

---

### Task 4: Add normalized geometry and RAM-only bounded capture

**Files:**
- Create: `src-tauri/src/mutation_overlay/capture.rs`
- Modify: `src-tauri/src/mutation_overlay/mod.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `scripts/installer-safety.test.mjs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Clone, Copy, Serialize, Deserialize)]
  pub struct NormalizedRect { pub x: f64, pub y: f64, pub w: f64, pub h: f64 }
  pub struct GrayFrame { pub width: u32, pub height: u32, pub pixels: Vec<u8> }
  pub trait MutationFrameSource {
      fn capture(&self, hwnd: isize, game_rect: (i32,i32,i32,i32), rect: NormalizedRect) -> Result<GrayFrame, CaptureError>;
  }
  ```

- [ ] **Step 1: Write unit tests for 1920x1080, 2560x1440, 3440x1440 and moved-window coordinate conversion**

Also assert out-of-range normalized values clamp to the client rectangle.

- [ ] **Step 2: Add black-frame detection tests**

A frame with >99% pixels within a tiny luminance range is rejected as `CaptureError::BlankFrame`.

- [ ] **Step 3: Implement geometry conversion and GDI screen capture**

Use only ordinary desktop/window geometry + GDI (`GetDC`/compatible DC/DIB + `BitBlt`) against the configured screen rectangle. Convert pixels directly to grayscale in memory and release every GDI object/DC on all paths.

- [ ] **Step 4: Extend safety test**

Assert the new module contains none of:

```text
OpenProcess
ReadProcessMemory
WriteProcessMemory
VirtualAllocEx
CreateRemoteThread
SetWindowsHookEx
```

- [ ] **Step 5: Run capture/unit/safety tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml mutation_overlay::capture
npm.cmd run test:installer
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/mutation_overlay/capture.rs src-tauri/src/mutation_overlay/mod.rs src-tauri/Cargo.toml scripts/installer-safety.test.mjs
git commit -m "feat: capture bounded mutation panel in memory"
```

---

### Task 5: Add Windows OCR adapter with frame-change throttling

**Files:**
- Create: `src-tauri/src/mutation_overlay/ocr.rs`
- Create: `src-tauri/src/mutation_overlay/frame_gate.rs`
- Modify: `src-tauri/src/mutation_overlay/mod.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Produces:
  ```rust
  pub trait MutationOcr { fn recognize(&self, frame: &GrayFrame) -> Result<String, OcrError>; }
  pub struct FrameGate { /* last perceptual hash + timing */ }
  pub fn should_ocr(&mut self, frame: &GrayFrame, now: Instant) -> bool;
  ```

- [ ] **Step 1: Write frame-gate tests**

Prove identical frames do not trigger OCR twice, materially changed frames do, and attempts are capped to 2Hz by default.

- [ ] **Step 2: Implement an in-memory PNG encoder or equivalent WinRT bitmap bridge**

No filesystem path is created. The encoded bytes live only long enough to construct a WinRT `SoftwareBitmap`.

- [ ] **Step 3: Implement Windows OCR**

Use `Windows.Media.Ocr.OcrEngine` with English (`en-US`) where available. Return `OcrError::Unavailable` when the language/OCR runtime is unavailable so the caller can fall back to manual mode.

- [ ] **Step 4: Run frame-gate and OCR construction tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml mutation_overlay::`

Expected: PASS; OCR-dependent tests may exercise error mapping without requiring a visible game window.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/mutation_overlay/ocr.rs src-tauri/src/mutation_overlay/frame_gate.rs src-tauri/src/mutation_overlay/mod.rs src-tauri/Cargo.toml
git commit -m "feat: add throttled Windows OCR for mutation overlay"
```

---

### Task 6: Build the dedicated transparent Mutation overlay window and calibration lifecycle

**Files:**
- Create: `src-tauri/src/mutation_overlay/window.rs`
- Create: `src/mutation-overlay/main.ts`
- Create: `src/mutation-overlay/MutationOverlay.svelte`
- Create: `mutation-overlay.html`
- Modify: `vite.config.ts`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/mutation_overlay/mod.rs`

**Interfaces:**
- Window label: `mutation-overlay`
- Events: `mutation-overlay://state`, `mutation-overlay://payload`
- Normal mode: transparent, borderless, non-focusable, click-through.
- Calibration mode: pointer input enabled, window resizable/drag-enabled, visible outline/instructions.

- [ ] **Step 1: Add static UI tests asserting the new Vite entry and overlay markup**

The overlay page must have no Hub navigation/buttons in normal mode and must render only `nameEn` + `descriptionVi` from trusted payloads.

- [ ] **Step 2: Implement the Vite entry and Svelte renderer**

Normal render structure:

```svelte
<div class="mask">
  <div class="name">{payload.nameEn}</div>
  <div class="description">{payload.descriptionVi}</div>
</div>
```

The page background stays transparent; the mask uses a restrained semi-opaque dark backing only behind the replaced text.

- [ ] **Step 3: Implement native window creation**

Mirror minimap's proven WebView2 isolation/topmost handling but use its own data directory `mutation-overlay-webview2` and its own supervisor/state.

- [ ] **Step 4: Implement calibration commands**

`begin` saves the current rect and makes the overlay interactive; `save` reads window position/size, converts to normalized coordinates against the current The Isle client rect, patches settings, and returns to click-through; `cancel` restores the saved rect.

- [ ] **Step 5: Wire startup and Tauri command registration**

Remove `mutation_locale::*` commands from `generate_handler!`, register the new `mutation_overlay_*` commands, and call `mutation_overlay::create(app.handle())?` during setup.

- [ ] **Step 6: Run frontend + Rust checks**

```powershell
npm.cmd run build
npm.cmd run check
cargo test --manifest-path src-tauri/Cargo.toml --workspace
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add mutation-overlay.html src/mutation-overlay vite.config.ts src-tauri/src/mutation_overlay src-tauri/src/lib.rs
git commit -m "feat: add calibrated mutation description overlay window"
```

---

### Task 7: Implement runtime supervisor, manual fallback, preview, and automatic detection

**Files:**
- Modify: `src-tauri/src/mutation_overlay/mod.rs`
- Modify: `src/lib/components/MutationGameLocalization.svelte`
- Modify: `src/lib/api.ts`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Supervisor state machine:
  ```text
  disabled -> waiting-game -> recognizing -> recognized
                         \-> manual
                         \-> capture-unavailable
                         \-> ocr-unavailable
  ```
- Manual/preview always bypass OCR but never alter the game.

- [ ] **Step 1: Add state-machine unit tests with fake frame source/OCR**

Verify no capture while disabled/not foreground, 2Hz gate, unchanged-frame OCR suppression, threshold gating, manual override, and hide-on-focus-loss.

- [ ] **Step 2: Implement supervisor loop**

Use `GAME_PROCESS_NAME`, `find_game_window`, `is_foreground`, and `client_rect_on_screen`. Poll geometry/foreground at a low cadence; only capture when enabled + foreground + auto-detect + not calibrating.

- [ ] **Step 3: Cache the last successful Mutation and payload**

Do not re-emit identical payloads. If OCR becomes ambiguous, retain the previous payload briefly, then hide it rather than guessing indefinitely.

- [ ] **Step 4: Implement manual selection and preview commands**

Manual selection looks up `nameEn` in the generated catalog and immediately emits the exact generated `descriptionVi`. Preview uses the same path with `source: "preview"` and works without a running game.

- [ ] **Step 5: Surface concise status text in Settings**

Map runtime states to the approved copy: `Đang chờ The Isle`, `Đang nhận diện`, `Đã nhận diện`, `Chọn thủ công`, plus one concise capture/OCR error.

- [ ] **Step 6: Run all targeted tests**

```powershell
npm.cmd run test:mutation-overlay-ui
npm.cmd run test:mutation-overlay-catalog
cargo test --manifest-path src-tauri/Cargo.toml mutation_overlay:: -- --nocapture
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/mutation_overlay src/lib/components/MutationGameLocalization.svelte src/lib/api.ts src-tauri/src/lib.rs
git commit -m "feat: run automatic and manual mutation translation overlay"
```

---

### Task 8: Remove failed `.locres` Mutation installer path and verify release safety

**Files:**
- Delete: `src-tauri/src/mutation_locale/**`
- Delete: `src/lib/mutation-locale-api.ts`
- Modify: `src-tauri/src/lib.rs`
- Modify: `scripts/mutation-localization-ui.test.mjs`
- Modify: `package.json`
- Review: `.github/workflows/**` if test script names are hard-coded

**Interfaces:**
- No `mutation_locale_install`, `mutation_locale_uninstall`, pack version, culture mutation, public locres downloader, `retoc`, or IoStore key logic remains reachable from the app.

- [ ] **Step 1: Extend tests to fail if obsolete installer symbols remain reachable**

Search frontend/native command registration for:

```text
mutation_locale_install
mutation_locale_uninstall
CÀI VIỆT HOÁ
GỠ VIỆT HOÁ
PUBLIC_ARTIFACT_PREFIX
retoc_cli
```

- [ ] **Step 2: Delete the old installer module/API and remove registrations/imports**

Do not touch third-party localization files already installed on the user's disk.

- [ ] **Step 3: Run the complete project verification suite**

```powershell
npm.cmd run check
npm.cmd run build
npm.cmd run test:mutations
npm.cmd run test:mutation-overlay-ui
npm.cmd run test:mutation-overlay-catalog
npm.cmd run test:installer
npm.cmd run test:release-trust
cargo test --manifest-path src-tauri/Cargo.toml --workspace
```

Expected: all PASS, zero safety regressions.

- [ ] **Step 4: Inspect CI workflow and update only renamed test invocation if necessary**

No workflow may silently skip the Mutation overlay tests.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: retire locres mutation installer"
```

---

### Task 9: Real-machine dev validation before any release

**Files:**
- No release/tag changes.

**Interfaces:**
- User test commands:
  ```powershell
  taskkill /F /IM islemap-thienvyma.exe 2>$null
  cd D:\hubisle-viet-hoa-test
  git pull --ff-only origin feature/the-isle-mutations-localization
  npm.cmd run tauri dev
  ```

- [ ] **Step 1: Verify Settings preview without launching The Isle**

Enable the feature, select `Cellular Regeneration`, press `HIỆN THỬ`, and confirm the overlay renders English name + `Hồi phục máu nhanh hơn 15%.`.

- [ ] **Step 2: Calibrate against the real Mutation screen**

Open The Isle Mutation UI, choose `CĂN CHỈNH VỊ TRÍ`, move/resize over the native name+description block, save, and confirm the normalized rectangle survives window movement/resolution changes.

- [ ] **Step 3: Validate automatic detection performance**

Switch across several Mutations. Expected response is normally <1 second, with no visible disk I/O and low idle CPU because identical frames skip OCR.

- [ ] **Step 4: Validate safety/focus behavior**

Alt-Tab away: overlay hides. Return: it restores. Disable feature: capture/OCR stops and overlay disappears immediately.

- [ ] **Step 5: Only after real-machine validation choose the next Hub release version/tag**

Do not create a GitHub Release automatically from this implementation branch.
