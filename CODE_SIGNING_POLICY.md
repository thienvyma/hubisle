# Code signing policy

`islemap-thienvyma` is owned and maintained by
[thienvyma](https://github.com/thienvyma). The source, build scripts and release
workflow are published at [github.com/thienvyma/hubisle](https://github.com/thienvyma/hubisle).

Free code signing provided by SignPath.io, certificate by SignPath Foundation

- Committers and reviewers: [thienvyma](https://github.com/thienvyma)
- Approvers: [thienvyma](https://github.com/thienvyma)

Every Windows release is built from a version tag by GitHub Actions. The
release workflow runs the automated checks and requires the application
executable, bundled Npcap telemetry sidecar and NSIS installer to carry valid,
timestamped Authenticode signatures before it can publish the release. The
SignPath submission stages will be connected after the open source application
is approved; until then this gate deliberately prevents a new public release.
Every future signing request will require manual approval in SignPath.

The product name, company metadata and repository owner are `thienvyma`. Under
the free open source program the certificate subject shown by Windows is
`SignPath Foundation`. A separate Tauri updater signature protects the exact
installer bytes referenced by `latest.json` after Authenticode signing.

Unsigned local and pull-request builds are development artifacts only. They
must never be attached to a public GitHub release or updater manifest.

See the [privacy policy](PRIVACY.md), [GPL-3.0 license](LICENSE), and
[release page](https://github.com/thienvyma/hubisle/releases).
