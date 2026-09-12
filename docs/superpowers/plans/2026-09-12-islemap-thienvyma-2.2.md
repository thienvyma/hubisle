# islemap-thienvyma 2.2 implementation plan

**Spec:** `docs/superpowers/specs/2026-09-12-islemap-thienvyma-2.2-design.md`

## Global constraints

- Version every release-facing manifest as 2.2.0.
- Preserve the existing GitHub updater endpoint and signing key.
- Preserve legacy deep-link and data paths only as migration compatibility.
- Integrate only through the official IsleVOIP launcher and public download page;
  do not copy private protocols, credentials, or paid server capabilities.
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

## Task 2: Friends and IsleVOIP UI

Add kept-alive Friends and Voice tabs. Friends shows the current provider's
accepted friend list and explains unavailable positions. Voice shows official
IsleVOIP installation/running state, starts the launcher, links to the official
download page, and controls the auto-start preference. Make both tabs reachable
even while the provider connection gate is active.

Verification: TypeScript/Svelte checks and production frontend build.

## Task 3: Official IsleVOIP launcher bridge

Add tested Rust path discovery and process-state helpers for the official
per-user IsleVOIP install. Expose commands to read status and start the launcher,
plus optional auto-start during hub setup. Keep the official download in the
frontend/system browser so installer ownership and updates stay with IsleVOIP.

Verification: red-green Rust unit tests for path precedence and process matching,
then Rust workspace tests and an installed-launcher smoke test.

## Task 4: Integrated verification and release

Review the complete diff for spec compliance, privacy, lifecycle leaks, upgrade
compatibility, and regressions. Run all repository test/check/build gates. Build
the signed updater and NSIS installer, copy the installer to the Desktop, install
over the previous product for a smoke test, confirm new processes/files and old
binary cleanup, commit, push `HEAD:main`, tag `v2.2.0`, wait for GitHub release,
and verify `latest.json` plus downloadable assets.
