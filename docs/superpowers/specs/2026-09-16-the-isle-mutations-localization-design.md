# The Isle Mutations Vietnamese Localization — Design

Date: 2026-09-16

## Goal

Keep the current hub behavior and Mutation UI unchanged, and add an optional in-hub installer named **VIỆT HOÁ THE ISLE MUTATIONS**.

When installed, the add-on changes only the detailed Mutation descriptions shown inside The Isle EVRIMA to Vietnamese. Canonical Mutation names stay in English.

## User experience

Add a dedicated card/section in the hub:

- Title: **VIỆT HOÁ THE ISLE MUTATIONS**
- Description: `Dịch phần mô tả Mutation trong The Isle sang tiếng Việt. Tên Mutation vẫn giữ nguyên tiếng Anh.`
- Detected game path
- Pack version and compatibility state
- Status: `Chưa cài`, `Đã cài`, `Có bản cập nhật`, `Không tương thích`, or an actionable error
- Primary action: `CÀI VIỆT HOÁ`
- Secondary actions when installed: `CẬP NHẬT` and `GỠ VIỆT HOÁ`

The user does not download or run a separate installer manually. Pressing the hub button performs the installation.

## Scope and non-goals

### In scope

- Preserve all current v2.6.0 Mutation features in the hub.
- Detect a Steam installation of The Isle on Windows.
- Install a self-contained Vietnamese localization payload for Mutation descriptions.
- Verify the payload before and after installation.
- Detect installed/outdated/incompatible state.
- Remove only files owned by this feature.
- Keep English Mutation names untouched.

### Out of scope

- Translating the entire The Isle UI.
- Modifying gameplay, stats, server behavior, Mutation effects, or network traffic.
- Patching the game executable.
- DLL injection or anti-cheat bypasses.
- Editing/deleting original game assets.
- Removing the Mutation library/details already present in the hub.

## Architecture

Split the feature into four independent units.

### 1. Localization payload

The payload contains only files required to replace/localize Mutation **description** text in the current EVRIMA build.

Preferred implementation is Unreal localization resources (`.locres`) because they can be added as language data without changing the game executable. The pack must not include translated Mutation names.

Before shipping a payload, the build pipeline must verify the current EVRIMA assets actually expose the target description strings as localizable text with stable namespace/key pairs. If the current build does not expose those descriptions through Unreal localization, release must fail rather than silently install a non-working pack. A different safe content-localization mechanism may then be designed separately; no executable patching or injection is permitted by this feature.

The payload has a manifest, for example:

```json
{
  "schema": 1,
  "packVersion": "1.0.0",
  "gameBuild": "<verified EVRIMA build id>",
  "files": [
    { "path": "...", "sha256": "..." }
  ]
}
```

The manifest is the source of truth for compatibility, integrity, updates, and uninstall ownership.

### 2. Native installer service

Implement installation logic on the Tauri/Rust side rather than in Svelte.

Responsibilities:

- Find Steam libraries and the The Isle install directory.
- Validate that the selected directory is a real The Isle installation.
- Refuse installation while the target files cannot be safely written.
- Resolve the localization destination without modifying original game files.
- Copy through a temporary path and rename atomically.
- Verify SHA-256 after copy.
- Persist a small ownership manifest under the hub's app-data directory.
- Report status and precise errors to the frontend.
- Uninstall only files listed in the ownership manifest and only if they still belong to the pack.

No generic arbitrary-path write API is exposed to the webview. Commands accept only the localization operation and internally constrain destinations to a validated The Isle install.

### 3. Hub UI

Add one focused UI section/card named **VIỆT HOÁ THE ISLE MUTATIONS**. Do not replace or remove the existing Mutation library/details UI.

The frontend calls native commands such as:

- `mutation_locale_status`
- `mutation_locale_install`
- `mutation_locale_uninstall`
- optionally `mutation_locale_refresh`

UI states are explicit and recoverable. Installation errors never block the rest of the hub.

### 4. Distribution and update model

The localization payload is versioned separately from the hub application.

Preferred distribution:

- GitHub Release asset attached to hub releases, or a dedicated stable release asset URL.
- Manifest + payload checksum verified before installation.
- The hub can update the translation pack without replacing unrelated hub code when the distribution model permits it.

For the first implementation, bundling the manifest/payload with the app release is acceptable if it materially simplifies integrity and offline installation. The interface should still keep `packVersion` separate from `__APP_VERSION__` so it can move to remote assets later without redesigning the UI.

## Game-path detection

Detection order:

1. Read Steam's configured library folders.
2. Look for app install metadata for The Isle and resolve `steamapps/common/The Isle` from Steam's own metadata rather than assuming `C:\Program Files (x86)`.
3. Validate expected The Isle directory markers.
4. If auto-detection fails, allow the user to choose a folder; validate it before enabling installation.

A manually selected path is never trusted without validation.

## Installation behavior

Install is transactional:

1. Resolve and validate game path.
2. Resolve current pack manifest and compatibility.
3. Verify source payload hashes.
4. Stage files in a temporary directory on the same volume where practical.
5. Re-check destination constraints.
6. Copy/rename payload files.
7. Verify installed hashes.
8. Write ownership/state manifest.
9. Return `installed` only after every verification passes.

On failure, clean staging files and leave the previous valid installation intact whenever possible.

## Uninstall behavior

Uninstall reads the ownership manifest and removes only pack-owned files. It must not recursively delete localization directories and must not remove files that no longer match a known owned payload unless the user explicitly confirms a repair flow in a future design.

## Compatibility behavior

A translation pack is tied to a verified EVRIMA game build or localization-key fingerprint.

If the game updates and compatibility cannot be established:

- Status becomes `Không tương thích / cần cập nhật gói dịch`.
- The hub does not install an old pack into an unknown game build.
- Existing installed files may be left in place but the UI warns that they are not verified for the current build; update/uninstall remains available.

This prevents a normal game update from turning into silent broken localization.

## Translation rules

- `nameEn` / canonical Mutation name: never translated in the game pack.
- Description: Vietnamese only.
- Percentages, numeric effects, slot requirements, unlock requirements, and gameplay terminology must preserve the original meaning.
- The hub's current offline Mutation catalog can be reused as a translation source only after each entry is mapped to a verified in-game localization key. Matching by visible English sentence at runtime is not sufficient for production packaging.

## Security and anti-cheat boundary

The feature is deliberately data-only:

- no EXE patching
- no runtime memory modification
- no DLL injection
- no EAC bypass
- no modification of server traffic

The installer should operate while The Isle is closed when writing localization files, both for consistency and to avoid partial reads by the game.

## Tests

### Native unit tests

- Steam library parsing and path resolution.
- Reject false-positive folders.
- Destination path cannot escape the validated game root.
- Manifest validation rejects malformed paths, duplicate paths, missing hashes, and traversal entries.
- Install state detection: absent/current/outdated/corrupt/incompatible.
- Uninstall removes only owned files.
- Failed install keeps previous valid version.

### Frontend tests

- Correct copy and buttons for each state.
- Exact title **VIỆT HOÁ THE ISLE MUTATIONS**.
- Existing Mutation components remain mounted/available.
- Errors are shown without breaking other tabs.

### Release verification

- Payload manifest hashes match release assets.
- No Mutation-name translation entries are present in the game localization payload.
- Every Vietnamese Mutation description maps to a verified current-game localization key.
- Install into a clean test The Isle directory fixture, detect as installed, then uninstall and verify original fixture is unchanged.

## Release strategy

Do not rewrite or retag `v2.6.0`.

Implement on a feature branch, verify the pack against the current EVRIMA build, merge after tests pass, then release as a new patch/minor version (for example `v2.6.1` if no broader versioning change is needed).

## Acceptance criteria

The feature is complete when:

1. v2.6.0 hub features remain intact.
2. The hub visibly offers **VIỆT HOÁ THE ISLE MUTATIONS**.
3. One button installs the data-only localization pack into a detected/validated The Isle installation.
4. In game, canonical Mutation names remain English.
5. In game, targeted Mutation descriptions display Vietnamese.
6. Hub reports installation/update/compatibility state accurately.
7. Uninstall restores the game to its pre-feature state by removing only hub-owned localization files.
8. No executable patch, injection, EAC bypass, or original-asset overwrite is introduced.
