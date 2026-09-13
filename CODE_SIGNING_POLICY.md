# Code signing policy

`islemap-thienvyma` is owned and maintained by
[thienvyma](https://github.com/thienvyma). The source, build scripts and release
workflow are published at [github.com/thienvyma/hubisle](https://github.com/thienvyma/hubisle).

Free code signing provided by SignPath.io, certificate by SignPath Foundation

- Committers and reviewers: [thienvyma](https://github.com/thienvyma)
- Approvers: [thienvyma](https://github.com/thienvyma)

Every Windows release is built from a version tag by GitHub Actions. The
release workflow runs the automated checks and always creates a Tauri updater
signature over the final installer. While the open source SignPath application
is pending, transition releases may be published without Authenticode and are
identified as such in their release notes. Once SignPath provisions the
project, the repository variable `SIGNPATH_ENFORCE_AUTHENTICODE=true` makes the
workflow require valid, timestamped Authenticode signatures on the application
executable, bundled Npcap telemetry sidecar and NSIS installer before it can
publish. Every SignPath signing request will require manual approval.

The product name, company metadata and repository owner are `thienvyma`. Under
the free open source program the certificate subject shown by Windows is
`SignPath Foundation`. A separate Tauri updater signature protects the exact
installer bytes referenced by `latest.json` after Authenticode signing.

Unsigned local and pull-request builds are development artifacts only. A
transition release built by the protected GitHub workflow may be published
while the SignPath application is pending, but Windows can still display a
SmartScreen warning for it.

See the [privacy policy](PRIVACY.md), [GPL-3.0 license](LICENSE), and
[release page](https://github.com/thienvyma/hubisle/releases).
