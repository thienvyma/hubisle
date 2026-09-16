# Third-party notices

The application uses open source dependencies distributed under their own
licenses. JavaScript dependency license metadata is recorded in the npm lock
files, Rust dependency metadata is recorded in `src-tauri/Cargo.lock`, and the
Npcap sidecar notices are in
`sidecars/local-telemetry/THIRD_PARTY_NOTICES.md`.

Map imagery, server POIs, dinosaur models and textures are fetched at runtime
from the service selected by the user. They are not included in the source
repository or installer and remain subject to the terms and copyrights of
Vulnona, IsleMaps, myislemap, IslePilot, Afterthought LLC and the relevant game
server operators. This project makes no ownership claim over that content.

Npcap is installed separately from its official distribution when the user
accepts its installer. Npcap is not redistributed inside this application's
installer.

## Mutation localization references

The **VIỆT HOÁ THE ISLE MUTATIONS** feature uses independently implemented
code in this repository. Its understanding of Unreal Engine `.locres` binary
layout was cross-checked against the MIT-licensed `akintos/UnrealLocres`
project. Exact public English Mutation description wording and Vietnamese
meaning were cross-checked against the MIT-licensed `klong-dev/IsleLiveMap`
Mutation catalog.

Current EVRIMA `pakchunk` IoStore containers are encrypted, so this feature does
not recover, request, or use The Isle's container encryption key. Instead, when
no loose English localization exists, the hub downloads the public Vietnamese
localization package published for `klong-dev/IsleLiveMap` from
`https://isle.klong.dev`, verifies the package and selected `Game.locres` with
SHA-256 metadata supplied by that service, and keeps only entries whose values
match the reviewed Mutation-description catalog. Mutation-name translations
and unrelated UI translations are discarded before the hub writes its own
minimal local resource.

The Isle game files and localization keys remain property of their respective
copyright holders. The hub does not redistribute original encrypted game
containers and does not modify or bypass EAC.