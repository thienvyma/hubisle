# The Isle Mutation Description Overlay — Design

Date: 2026-09-16

## Context

The earlier `Game.locres` installer approach is retired for Mutation descriptions.

Static analysis of the current The Isle EVRIMA client shows that Mutation names and descriptions are compiled into the native client and represented through Mutation metadata such as `MutationName`, `MutationDescription`, `MutationsData`, and `LifecycleMutationsList`. They are not present in the loose `Game.locres` used by ordinary UI localization. The existing DinoVietNam localization mechanism therefore cannot translate this specific Mutation text.

The hub must not patch `TheIsleClient-Win64-Shipping.exe`, inject a DLL, read game memory, bypass Easy Anti-Cheat, or decrypt/modify protected game packages.

The replacement design presents Vietnamese Mutation descriptions in a transparent companion overlay positioned directly over the game's Mutation detail text. To the player it should visually read like the Mutation detail itself is localized, while the game process and game files remain untouched.

## Goal

Add an optional feature named **VIỆT HOÁ THE ISLE MUTATIONS** that:

- preserves the canonical Mutation name in English;
- displays the Vietnamese description from the hub's existing `src/lib/mutations.ts` catalog;
- visually replaces/covers only the Mutation detail text area while the Mutation screen is in use;
- follows the The Isle window across supported resolutions/window positions;
- never patches or injects into the game process;
- can be disabled instantly from Hub Settings;
- still works through a manual selection fallback when automatic text detection is unavailable.

## Source of truth

`src/lib/mutations.ts` is the only translation source of truth.

The overlay implementation must not maintain a second hand-written Vietnamese Mutation catalog in Rust. Build/test tooling may generate a compact Rust/JSON artifact from `src/lib/mutations.ts`, but generated data must be reproducible and CI must fail when it diverges from the TypeScript source.

Rules:

- `nameEn` remains English.
- `descriptionVi` is rendered by the overlay.
- Gameplay effect values, requirements, Mutation slots, and behavior are never modified.
- Existing Mutation Library/Details screens in the hub remain intact.

## User experience

The Settings screen keeps a dedicated section titled exactly:

**VIỆT HOÁ THE ISLE MUTATIONS**

The previous installer-oriented status (`pack version`, `installed`, `uninstall`) is removed. The section instead exposes:

- toggle: `BẬT VIỆT HOÁ MUTATIONS`;
- status: `Đang chờ The Isle`, `Đang nhận diện`, `Đã nhận diện`, `Chọn thủ công`, or an actionable error;
- button: `CĂN CHỈNH VỊ TRÍ`;
- manual fallback selector/search for the Mutation currently highlighted in game;
- optional `HIỆN THỬ` action that renders a known Mutation without needing to enter a server.

When enabled and The Isle is foreground:

1. Hub anchors a transparent click-through window to The Isle.
2. Hub observes only a configured screen rectangle containing the selected Mutation detail.
3. If automatic recognition identifies a known Mutation with sufficient confidence, the overlay covers the configured text block and renders:
   - English Mutation name;
   - Vietnamese description from `src/lib/mutations.ts`.
4. If recognition is uncertain, the overlay does not guess. The Settings card reports `Chọn thủ công`; selecting a Mutation manually immediately renders its Vietnamese description.
5. When The Isle loses foreground, closes, or the feature is disabled, the overlay hides.

## Visual treatment

The overlay is not a floating popup outside the game UI. It is a transparent borderless Tauri window positioned on top of the Mutation detail text region.

The rendered block should imitate The Isle's understated HUD typography rather than the Hub's card styling:

- transparent page/window background;
- a small semi-opaque backing only where needed to mask the original English description;
- English Mutation name retained;
- Vietnamese description below it;
- no title bar, shadow, Hub logo, buttons, or interactive chrome during normal gameplay;
- mouse click-through and non-focusable;
- no animation that distracts from gameplay.

The exact rectangle is stored as normalized coordinates relative to the The Isle client area so it survives resolution changes.

Example normalized configuration:

```json
{
  "mutation_overlay": {
    "enabled": true,
    "rect": { "x": 0.61, "y": 0.28, "w": 0.28, "h": 0.22 },
    "auto_detect": true,
    "confidence_threshold": 0.82
  }
}
```

## Calibration

Because The Isle UI scale, resolution, aspect ratio, and future updates can move the Mutation panel, position is user-calibrated rather than hard-coded to one monitor layout.

`CĂN CHỈNH VỊ TRÍ` enters a temporary interactive calibration mode:

- The Isle must be running and its Mutation screen visible.
- The overlay becomes focusable only during calibration.
- A visible rectangle can be moved/resized over the Mutation name + description area.
- Saving converts the rectangle into normalized client coordinates.
- Cancelling restores the previous rectangle.
- Calibration never captures or writes game memory.

A reasonable default rectangle may be shipped for common 16:9 layouts, but the product must not depend on it being correct.

## Automatic detection

Automatic detection is deliberately screen-based.

### Capture boundary

Only the configured Mutation detail rectangle is captured. The implementation must not perform process memory reads, DLL injection, executable patching, or game-package decryption.

The capture component first attempts a Windows-supported window/screen capture path for the The Isle client rectangle. The code is isolated behind:

```rust
pub trait MutationFrameSource {
    fn capture(&self, game: GameWindow, rect: NormalizedRect) -> Result<GrayFrame, CaptureError>;
}
```

This makes the capture mechanism replaceable if one Windows API behaves poorly with fullscreen rendering.

### Recognition

The recognition pipeline is:

```text
capture configured rectangle
        -> grayscale / upscale / contrast normalization
        -> OCR text
        -> normalize OCR text
        -> match against known English Mutation name + current in-game description aliases
        -> confidence gate
        -> selected Mutation ID
```

Recognition does not need to translate text. It only identifies which known Mutation is selected.

The detector already has initial tests for exact, normalized, and one-word-damaged OCR input. Those tests become the base of the production matcher rather than remaining test-only scaffolding.

The matching catalog must include observed current-game English aliases separately from the Vietnamese translation source. This is necessary because the strings compiled in the game (for example `Recovers health slightly faster`) are sometimes shorter/different from the hub's explanatory `descriptionEn`. Alias entries identify a Mutation but never provide the Vietnamese output.

### OCR implementation

Prefer the Windows built-in OCR stack when available so the Hub does not ship a large OCR model. OCR is invoked only while:

- the feature is enabled;
- The Isle is foreground;
- the capture rectangle appears to contain UI text;
- and the previous result needs refresh.

Target cadence is low (approximately 2–4 recognition attempts per second while the Mutation screen is active), not frame-rate scanning.

If Windows OCR is unavailable or capture fails, automatic recognition degrades to manual mode. This must not break the rest of the Hub.

## Manual fallback

Manual mode is a required feature, not an error-only afterthought.

From the Settings card, the user can search/select one of the Mutation names already in `src/lib/mutations.ts`. The selected English name and Vietnamese description are pushed to the overlay immediately.

Manual selection remains active until:

- auto-detection confidently finds a different Mutation;
- the user clears it;
- the overlay is disabled;
- or The Isle exits.

This guarantees the feature remains usable even if a game update changes fonts/layout or Windows capture/OCR is unavailable.

## Native overlay architecture

Reuse the existing minimap overlay principles, but create a separate window with a separate WebView2 data directory and supervisor.

Suggested module boundary:

```text
src-tauri/src/mutation_overlay/
  mod.rs          orchestration + lifecycle
  detect.rs       normalized text matcher + confidence
  capture.rs      safe Windows screen/window capture
  ocr.rs          OCR adapter
  window.rs       Tauri overlay creation/anchoring/click-through
  catalog.rs      generated/validated recognition aliases only
```

Frontend overlay entry:

```text
src/mutation-overlay/
  main.ts
  MutationOverlay.svelte
mutation-overlay.html
```

The native supervisor owns show/hide/position. The Svelte overlay renderer only receives a small event payload:

```ts
interface MutationOverlayPayload {
  nameEn: string;
  descriptionVi: string;
  confidence: number | null;
  source: "auto" | "manual" | "preview";
}
```

No arbitrary HTML or OCR text is injected into the overlay.

## Settings and API

Remove installer semantics from the Settings card and replace them with overlay semantics.

Settings additions:

```ts
mutationOverlay: {
  enabled: boolean;
  autoDetect: boolean;
  rect: { x: number; y: number; w: number; h: number };
  confidenceThreshold: number;
}
```

Narrow Tauri commands/events:

- `mutation_overlay_status`
- `mutation_overlay_set_manual`
- `mutation_overlay_clear_manual`
- `mutation_overlay_begin_calibration`
- `mutation_overlay_save_calibration`
- `mutation_overlay_cancel_calibration`
- `mutation_overlay_preview`
- `mutation-overlay://state`
- `mutation-overlay://payload`

Normal enable/disable state should flow through the existing settings patch mechanism when practical.

## Safety boundary

This feature must remain outside the game process.

Forbidden implementation techniques:

- `OpenProcess` against The Isle for data extraction;
- `ReadProcessMemory` / `WriteProcessMemory`;
- DLL injection;
- API hooks inside the game;
- executable patching;
- Easy Anti-Cheat bypass/tampering;
- decrypting or rewriting protected IoStore/Pak content;
- synthetic game network traffic.

Allowed interaction is limited to ordinary OS window discovery, foreground state, geometry, bounded screen capture, transparent overlay rendering, and user input directed at the Hub itself.

Existing safety CI must continue to reject forbidden APIs.

## Performance

The mutation overlay must be dormant when unused.

- No capture/OCR while feature is disabled.
- No capture/OCR while The Isle is not foreground.
- Overlay supervisor may poll geometry/foreground at a low cadence comparable to minimap supervision.
- OCR runs only when necessary and must be rate-limited.
- Cache the last successful Mutation and avoid re-rendering identical payloads.
- If OCR repeatedly fails, back off rather than burn CPU continuously.

## Error handling

Expected states:

- The Isle not running: hide overlay, Settings says `Đang chờ The Isle`.
- Game running but not foreground: hide overlay.
- No calibration: prompt `Căn chỉnh vị trí` and allow manual preview.
- Capture unavailable/black frame: switch to manual fallback and surface one concise explanation.
- OCR unavailable: manual fallback remains available.
- OCR result ambiguous: do not guess; keep previous result briefly, then hide translated text and request manual selection.
- Unsupported resolution/layout after game update: calibration fixes positioning without requiring a Hub release.

## Removal of failed installer path

The following behavior is removed from the product UI:

- localization pack version (`v1.x`);
- `CÀI VIỆT HOÁ` file installer;
- `CẬP NHẬT GÓI DỊCH`;
- `GỠ VIỆT HOÁ`;
- Steam localization-file installation state;
- culture mutation for the purpose of Mutation descriptions;
- public `Game.locres` download as a Mutation-description source.

Old code may be deleted once the overlay replacement has tests covering the new flow. No previously installed loose localization files from third-party tools are touched by this feature.

## Tests

### Detector tests

- Exact current-game name + description maps to the right Mutation.
- Case/punctuation changes map correctly.
- Common OCR confusion (`rn` vs `m`, `0` vs `O`, dropped punctuation) remains within threshold.
- A screen containing several Mutation names but no selected-detail text does not guess.
- Ambiguous text returns no result.
- Every output description equals the corresponding `descriptionVi` from `src/lib/mutations.ts`.

### Capture tests

- Normalized rectangle converts correctly for 16:9, ultrawide, and moved window fixtures.
- Bounds are clamped to the game client rect.
- Empty/black frame is detected.
- No capture occurs when feature is disabled or game is not foreground.

### Window tests

- Overlay is borderless, transparent, non-focusable, click-through outside calibration.
- Overlay follows game window movement/resizing.
- Overlay hides when the game loses foreground/exits.
- Calibration is the only mode that temporarily accepts pointer input.

### UI tests

- Settings title is exactly `VIỆT HOÁ THE ISLE MUTATIONS`.
- Installer copy/buttons no longer exist.
- Toggle, calibration, auto/manual status, manual search, and preview are present.
- Existing Mutation Library and other v2.6.0 hub features remain intact.

### Safety tests

CI scans the new module and dependencies for forbidden process-memory/injection APIs and confirms the release does not patch or ship a modified The Isle executable.

## Acceptance criteria

The feature is complete when:

1. User enables **VIỆT HOÁ THE ISLE MUTATIONS** in Settings.
2. The Isle remains completely unmodified on disk and in memory by the Hub.
3. With The Isle foreground and the Mutation screen visible, the Hub can render a translated detail block directly over the configured Mutation text area.
4. Canonical Mutation name remains English.
5. Vietnamese description is exactly the corresponding `descriptionVi` from `src/lib/mutations.ts`.
6. Automatic recognition never renders a translation below the confidence threshold.
7. Manual selection works when automatic detection is unavailable.
8. Overlay follows the game window and hides outside the appropriate context.
9. Calibration works across common resolutions and UI scales.
10. Existing Hub features remain unchanged.
11. Full frontend/Rust/Windows CI passes, including anti-cheat safety checks.

## Release strategy

Continue implementation on `feature/the-isle-mutations-localization` but treat this document as superseding the earlier localization-installer design for Mutation descriptions.

Do not release or merge the old `.locres` installer path. After the overlay passes automated tests, test it against the user's real The Isle installation in dev mode before merging. Only then choose the next Hub version/release tag.
