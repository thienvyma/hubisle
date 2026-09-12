# islemap-thienvyma 2.2 design

## Goal

Release version 2.2.0 as `islemap-thienvyma`, preserve the signed update path
from Isle Pulse Overlay 2.1.4, restore a useful Friends surface, and connect the
hub to the official IsleVOIP launcher used by supported servers without requiring
a player-side Pro upgrade.

## Brand and upgrade compatibility

All active user-facing product names, executable names, sidecar names, package
metadata, window/tray labels, release workflow text, and current documentation
use `islemap-thienvyma`. Historical changelog entries remain historical.

The NSIS pre-install hook stops both old and new processes and removes stale old
executables before copying 2.2.0. The app accepts both the new deep-link scheme
and the legacy `theisle-overlay` scheme so existing OAuth redirects continue to
work. On first launch, the app migrates roaming and local data directories from
`TheIsleOverlay` to `islemap-thienvyma`; settings, map downloads, waypoints,
history, and encrypted credentials survive the rename. Updater endpoint and
signing public key remain unchanged.

## Friends and IsleVOIP

The Friends tab exposes accepted friends already returned by Era, Titan, or
IslePilot, including online state, species and whether a live position is
available. Existing provider friend markers remain on the full map and minimap.
The hub does not invent records when a server hides or omits the friend list.

The Voice tab integrates the official IsleVOIP desktop launcher. The backend
detects the standard per-user installation and running process, starts it on
request, and can auto-start it with the hub when enabled. If it is not installed,
the UI opens the official IsleVOIP download page in the user's browser. The
official launcher performs Steam login, supported-server discovery, nearby-player
selection and voice transport; islemap-thienvyma does not copy private protocols,
credentials, or paid server capabilities.

Players do not buy Pro: the server owner chooses its IsleVOIP license. Free
servers provide the service's basic voice; 3D proximity/range behavior is
available when that server has enabled the corresponding IsleVOIP plan. The hub
shows this boundary clearly and never claims to unlock a server plan.

## Availability and failure handling

Friends and Voice remain visible even when no provider is authenticated. The
Voice tab reports missing installation and process-launch failures without
crashing the shell. Provider-specific map, Dino, Garage, Skin, and History
behavior keeps its existing connection requirements.

## Documentation and validation

README.md and README.en.md contain text only and describe the real 2.2.0 feature
set. Obsolete screenshot and guide image assets are removed when no references
remain.

Validation covers IsleVOIP path discovery/process state and data migration,
frontend type checking/build, Rust workspace tests, sidecar build/probe, version
consistency, forbidden API checks, NSIS bundle generation, an installed-file
smoke test, and GitHub release artifact/update-manifest verification.
