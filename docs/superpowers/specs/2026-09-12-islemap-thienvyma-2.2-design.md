# islemap-thienvyma 2.2 design

## Goal

Release version 2.2.0 as `islemap-thienvyma`, preserve the signed update path
from Isle Pulse Overlay 2.1.4, restore a useful Friends surface, and add free
community voice without depending on the closed IsleVOIP/Pro service.

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

## Friends and community voice

The Friends tab combines two sources without fabricating provider data:

1. Accepted friends returned by Era, Titan, or IslePilot.
2. Peers in a user-created community room.

Community rooms use Trystero over WebRTC. A room code supplies both a derived
discovery identifier and session password. Presence packets contain a validated
display name, provider/server identifiers, optional world position, and muted
state. No subscription or IsleVOIP account is required. Everyone in a room must
run islemap-thienvyma and enter the same code.

Microphone capture starts muted. The user explicitly unmutes, can set master and
per-peer volume, and can leave the room at any time. Leaving stops local tracks,
disconnects the room, and removes remote audio elements. When both peers have
positions on the same server, volume is full at close range and fades to silence
at long range. If a provider does not expose coordinates, room voice remains
usable without proximity attenuation and the UI explains that limitation.

The backend projects community world coordinates through the active basemap
calibration. The main window emits only normalized map markers to the minimap.
The full map and minimap merge provider and community markers and deduplicate
them by stable source identity. Remote payloads are bounded and treated as
untrusted input.

## Availability and failure handling

Friends and Voice remain available even when no provider is authenticated so
the feature works on servers without supported account APIs. Provider-specific
map, Dino, Garage, Skin, and History behavior keeps its existing connection
requirements. Voice reports microphone denial, peer discovery failure, and
autoplay failure in the tab without crashing the shell.

Trystero discovery uses public Nostr relays; direct WebRTC can fail on restrictive
networks without TURN. This release reports that state clearly and does not claim
universal connectivity.

## Documentation and validation

README.md and README.en.md contain text only and describe the real 2.2.0 feature
set. Obsolete screenshot and guide image assets are removed when no references
remain.

Validation covers pure community normalization, distance attenuation, room-code
derivation inputs, Rust coordinate projection and data migration, frontend type
checking/build, Rust workspace tests, sidecar build/probe, version consistency,
forbidden API checks, NSIS bundle generation, an installed-file smoke test, and
GitHub release artifact/update-manifest verification.
