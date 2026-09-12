# Test results

## Clean-machine acceptance result

| Check | Result | Evidence |
|---|---|---|
| Normal Steam/IslePilot login establishes a player session | Confirmed | Deep-link/session strings and encrypted `overlayTokenEnc` lifecycle |
| Player session alone authorizes Voice `/token` | Failed | Controlled requests return HTTP 401 |
| Voice `/token` requires a private application credential | Confirmed | `DINOVN_VOICE_TOKEN` and `x-api-key` xrefs in the official binary |
| Public client registration/token broker available | Not found | No supported public endpoint or local IPC was identified |
| My Hub works without DinoVietNam.exe | Pass for all shipped v2.4 features | Voice process bridge and UI removed |
| Standalone DinoVietnam Voice works | Blocked by server authorization | Requires the backend contract documented in `INTEGRATION_OPTIONS.md` |

## Security checks

- No credential value was printed, persisted or copied.
- No official file was modified.
- No TLS validation was disabled.
- No client identity, Steam identity or attestation was forged.
- No second Voice connection or microphone publisher was created.
- v2.4 has no code path that starts or depends on DinoVietNam.exe.

The regular Hub build, unit, type, lint and packaging checks are recorded by the
release verification run. Voice transport, room, microphone/PTT and reconnect
cannot be truthfully marked as working because the legitimate server-side grant
does not exist for this client.
