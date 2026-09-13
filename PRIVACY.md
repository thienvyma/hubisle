# Privacy policy

This policy applies to `islemap-thienvyma`, maintained by
[thienvyma](https://github.com/thienvyma). The app does not sell personal data
or include advertising.

## Data stored on the computer

Settings, waypoints, trails, saved skins and combat history are stored locally.
Downloaded maps and 3D preview assets are cached locally. Provider login tokens
and cookies are encrypted with Windows DPAPI, which binds them to the current
Windows user on that computer. Uninstalling the app may leave this user data so
a later reinstall can restore settings; it can be deleted from
`%APPDATA%\islemap-thienvyma` and `%LOCALAPPDATA%\islemap-thienvyma-data`.

## Network connections

The app makes these connections only for the related feature:

- GitHub Releases (`github.com/thienvyma/hubisle`) to check for, download and
  verify app updates.
- The server website selected by the user, including Era Gaming VN, Titan and
  IslePilot servers, to sign in and retrieve the player's live data, friends,
  Garage, skins, Prime progress and server POIs. Credentials are sent only to
  the verified provider domain that issued them.
- Vulnona, IsleMaps and myislemap to download map imagery and POI data. These
  files are requested when the associated map source or layer is used.
- The public IslePilot CDN to download 3D models and textures when a supported
  dinosaur preview is opened. The files remain the property of their respective
  owners and are not included in this repository or installer.
- The `islemap-thienvyma` Cloudflare Worker for the optional friend-position
  relay. This happens only after the user enables the clearly labelled sharing
  control. The Worker forwards the IslePilot bearer token to IslePilot's
  `/api/overlay/me` and `/api/overlay/friends` endpoints to verify the caller,
  accepted friendships and current server. It never stores the token. Cloudflare
  D1 stores secret-peppered hashes of player/server identifiers plus coordinates
  for at most 210 seconds. Only accepted friends on the same server are returned.
- A configured server Voice provider when the user signs in or requests room
  access. Provider sessions are protected with Windows DPAPI and scoped to the
  exact server origin. Room grants are short-lived and never written to disk.
  The current framework does not capture or transmit microphone audio.

Unknown Prime text is left in English. It is not sent to an online translation
service.

## Npcap and local telemetry

The bundled `islemap-thienvyma-telemetry` sidecar uses Npcap to inspect the
game's outbound UDP traffic locally for position and camera heading. Raw packets
are not sent to `thienvyma` or the Cloudflare Worker. The separate anonymous
analytics/crash client is disabled in public builds because no analytics API
endpoint is configured. Its setting defaults to off. Feedback is sent only
when the user writes a message and presses the send button; while no endpoint is
configured the request fails locally without transmitting it.

## Controls

Friend relay sharing and anonymous usage data are off by default and can be
changed in the app. Provider sessions can be disconnected from the connection
screen. Local data can be removed using the paths above.

Questions can be raised at
[github.com/thienvyma/hubisle/issues](https://github.com/thienvyma/hubisle/issues).
