# The Isle Mutations Vietnamese Overlay — Design

Date: 2026-09-16

## Goal

Replace the failed file-localization installer with a safe runtime overlay named **VIỆT HOÁ THE ISLE MUTATIONS**.

The overlay keeps the native The Isle Mutation name in English and shows only the Vietnamese description from the hub's existing `src/lib/mutations.ts` catalog. It must not patch the game executable, decrypt IoStore, inject DLLs, read game process memory, or bypass EAC.

## Why the architecture changed

Binary inspection of the current EVRIMA client showed that Mutation names and descriptions are compiled into `TheIsleClient-Win64-Shipping.exe`, and reflection metadata identifies `MutationName` and `MutationDescription` as native `FName` fields rather than localizable `FText` entries. Therefore `.locres` cannot translate the Mutation descriptions. The previous installer path is retired.

## User experience

The existing Settings card remains titled exactly **VIỆT HOÁ THE ISLE MUTATIONS**.

It becomes a runtime toggle rather than an installer:

- Status: `ĐANG TẮT`, `ĐANG BẬT`, `THE ISLE CHƯA CHẠY`, or `OCR KHÔNG KHẢ DỤNG`.
- Primary action: `BẬT VIỆT HOÁ` / `TẮT VIỆT HOÁ`.
- Explanation: the hub reads only pixels from the game window and renders a separate click-through overlay; no game files are modified.

When enabled and The Isle is the foreground application:

1. The native worker captures the visible game client area with read-only Win32 screen capture.
2. Windows on-device OCR recognizes the visible English UI text.
3. Detection signatures match the current in-game Mutation description wording.
4. The worker emits only the detected canonical English Mutation name and confidence to the overlay webview.
5. The overlay webview imports `src/lib/mutations.ts` and renders `descriptionVi` for that name. This keeps the hub's existing catalog as the single source of truth for Vietnamese text.
6. The native Mutation name remains visible in the game and is never translated or covered intentionally.

## Detection strategy

Prefer description matching over name-only matching because a Mutation screen can show multiple names simultaneously while the selected Mutation has one detail description.

The Rust detector contains only current English detection signatures:

```text
canonical English name -> current in-game English description
```

It does not contain Vietnamese copy.

OCR text is normalized to lowercase ASCII-ish alphanumeric words. A candidate can match by:

- exact normalized description containment, or
- high token overlap for OCR with minor punctuation/character errors.

A result is emitted only above a conservative confidence threshold. Ambiguous/low-confidence screens hide the translation rather than displaying the wrong Mutation.

## Capture and OCR

Windows implementation:

- identify the game by process name through the existing `win::game_window` code;
- operate only while the game window is foreground and not minimized;
- capture the game client rectangle from the desktop with GDI `BitBlt`/`GetDIBits` (no game process handle);
- downscale the capture to a bounded OCR width before recognition to cap CPU cost;
- use `Windows.Media.Ocr.OcrEngine` with the English language when available, otherwise user-profile OCR languages;
- poll around every 700–1000 ms while enabled and foreground, and sleep much longer while disabled/backgrounded.

No image is uploaded, persisted, or sent over the network.

## Overlay window

Create a dedicated Tauri webview window `mutation-overlay`:

- transparent;
- decorationless;
- always on top;
- non-focusable;
- click-through;
- taskbar-hidden;
- created hidden;
- shown only while there is a valid detected Mutation and The Isle is foreground.

Initial placement is bottom-center of the game client with a compact width (~520 px) and enough height for two or three lines of Vietnamese copy. It follows the game client when the window moves/resizes.

The webview page is intentionally tiny and independent of the full main UI/minimap bundle.

## Translation source of truth

`src/lib/mutations.ts` remains authoritative for:

- canonical English Mutation name;
- hub English description/reference;
- Vietnamese description shown to the user.

The native OCR detector never stores a second Vietnamese catalog. Its signature table exists only to identify the currently displayed Mutation from the current EVRIMA English strings.

## Settings

Add:

```json
{
  "mutation_overlay": {
    "enabled": false,
    "ocr_interval_ms": 850,
    "opacity": 0.94
  }
}
```

The feature defaults OFF. Existing installs gain these defaults through the normal nested settings merge.

## Failure behavior

- The Isle closed/backgrounded: hide overlay, do not OCR.
- OCR engine unavailable: hide overlay and expose status in Settings.
- Capture fails/black frame: skip the tick and retry later.
- No confident Mutation match: hide overlay.
- OCR produces multiple plausible matches: require a score margin; otherwise hide.
- Windows language pack missing: try user profile languages; if no OCR engine exists, report unavailable.

The feature must never block the main hub or minimap.

## Security / anti-cheat boundary

Allowed:

- process enumeration already used by the hub;
- window geometry/foreground checks;
- desktop pixel capture of the visible game window;
- OS-native OCR;
- separate transparent overlay rendering.

Forbidden:

- `OpenProcess` on The Isle;
- process memory reads/writes;
- EXE patching;
- DLL injection;
- hooks inside the game;
- IoStore decryption;
- EAC manipulation.

## Tests

### Pure Rust detector tests

- exact description identifies the expected canonical name;
- punctuation/case differences still match;
- a small OCR typo still matches when token overlap remains high;
- name-only lists with no detail description do not produce a false positive;
- ambiguous candidates return no match.

### Frontend contract tests

- Settings title remains exactly `VIỆT HOÁ THE ISLE MUTATIONS`;
- component exposes `BẬT VIỆT HOÁ` / `TẮT VIỆT HOÁ` instead of install/uninstall copy;
- overlay imports `MUTATION_CATALOG`/`mutationDisplay` from the hub catalog;
- old installer IPC calls are no longer used by the Settings component.

### Build/CI

- Svelte check/build;
- Rust workspace tests;
- existing forbidden-API safety test must remain green;
- add a guard ensuring the new OCR module does not use `OpenProcess`, `ReadProcessMemory`, `WriteProcessMemory`, DLL injection, or EAC paths.

## Acceptance criteria

1. The Settings card enables/disables the feature without modifying game files.
2. With The Isle foreground on the Mutation detail screen, a confidently recognized Mutation produces a Vietnamese description overlay.
3. The in-game Mutation name remains English.
4. Vietnamese copy comes from `src/lib/mutations.ts`.
5. Overlay hides on Alt-Tab, when no Mutation detail is detected, or when the feature is disabled.
6. No executable patching, memory access, injection, IoStore decryption, or EAC bypass is introduced.
7. Existing minimap/hub functionality remains unchanged.
