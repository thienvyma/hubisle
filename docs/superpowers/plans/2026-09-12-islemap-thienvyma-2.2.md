# islemap-thienvyma 2.2 implementation plan

**Spec:** `docs/superpowers/specs/2026-09-12-islemap-thienvyma-2.2-design.md`

## Global constraints

- Version every release-facing manifest as 2.2.0.
- Preserve the existing GitHub updater endpoint and signing key.
- Preserve legacy deep-link and data paths only as migration compatibility.
- Do not copy, decompile, impersonate, or bypass the closed IsleVOIP/Pro service.
- Test behavior before implementation for new logic; configuration and prose are
  validated by builds and end-to-end consistency checks.

## Task 1: Brand, storage migration, installer, and README

Update package/Cargo/Tauri/sidecar/workflow metadata and all active strings to
`islemap-thienvyma`. Add tested one-time data-directory migration. Extend NSIS
hooks to stop and clean old/new processes and prevent a side-by-side stale app.
Remove README image embeds and unreferenced README image assets. Update changelog
and release notes for 2.2.0.

Verification: migration unit tests, version checker, sidecar build, frontend
build, NSIS bundle, and inspection of output file names.

## Task 2: Community core and voice UI

Create a testable community core for payload validation, server matching,
distance/volume attenuation, and room-code normalization. Implement a singleton
Trystero room manager with encrypted room password, muted microphone startup,
presence exchange, audio lifecycle, master/per-peer volume, and error state.
Add kept-alive Friends and Voice tabs with room join/leave and peer controls.
Make these two tabs reachable even while the provider connection gate is active.

Verification: red-green unit tests for the pure core, TypeScript/Svelte checks,
and production frontend build.

## Task 3: Community map projection and marker integration

Add a tested Rust command that projects arbitrary validated world coordinates
through the active calibration. Convert peer positions to SharedFriend-compatible
markers in the main window and emit `community://friends`. Merge and deduplicate
provider/community markers in FullMap and minimap; refresh after basemap changes
and clear markers on leave.

Verification: Rust projection tests, frontend core tests, Svelte check, and a
production frontend build.

## Task 4: Integrated verification and release

Review the complete diff for spec compliance, privacy, lifecycle leaks, upgrade
compatibility, and regressions. Run all repository test/check/build gates. Build
the signed updater and NSIS installer, copy the installer to the Desktop, install
over the previous product for a smoke test, confirm new processes/files and old
binary cleanup, commit, push `HEAD:main`, tag `v2.2.0`, wait for GitHub release,
and verify `latest.json` plus downloadable assets.
