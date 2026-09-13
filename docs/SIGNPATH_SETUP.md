# SignPath release setup

The SignPath Foundation open source application for `islemap-thienvyma` was
submitted on 2026-09-14 and is awaiting review.

After approval, configure the following values in the GitHub repository. Keep
the API token in **Actions secrets** and put the other values in **Actions
variables**.

| Name | Location | Purpose |
| --- | --- | --- |
| `SIGNPATH_API_TOKEN` | Secret | Authenticates signing requests |
| `SIGNPATH_ORGANIZATION_ID` | Variable | SignPath organization identifier |
| `SIGNPATH_PROJECT_SLUG` | Variable | SignPath project slug |
| `SIGNPATH_SIGNING_POLICY_SLUG` | Variable | Release signing policy |
| `SIGNPATH_APP_ARTIFACT_CONFIGURATION_SLUG` | Variable | App and telemetry sidecar artifact layout |
| `SIGNPATH_INSTALLER_ARTIFACT_CONFIGURATION_SLUG` | Variable | NSIS installer artifact layout |
| `SIGNPATH_ENFORCE_AUTHENTICODE` | Variable | Set to `true` only after the signing stages are connected and tested |

The final workflow must sign the application executable and telemetry sidecar
before NSIS packaging, then sign the NSIS installer. Authenticode changes the
installer bytes, so the Tauri updater `.sig` and `latest.json` must be generated
from the Authenticode-signed installer. The public release remains a draft
until all signatures pass `scripts/check-authenticode.ps1` and
`scripts/verify-signed-installer.mjs`.

SignPath signing requests require manual approval. Do not store the API token
in source files, issue comments, release notes or chat messages.
