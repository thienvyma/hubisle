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
Mutation catalog. This project does not ship that project's Vietnamese
Mutation-name fields; canonical Mutation names remain English.

The Isle game files and localization keys remain property of their respective
copyright holders. The hub generates only user-local localization files from
the user's installed game data and does not redistribute original game assets.
