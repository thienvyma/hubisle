# Implementation status

No DinoVietnam Voice runtime is shipped in v2.4.

The investigation first produced a bounded background-process adapter, but that
adapter depended on DinoVietNam.exe and its existing local login state. It was
deleted after the clean-machine requirement was finalized. The application now:

- does not locate, start, minimize, restore or monitor DinoVietNam.exe;
- does not read its encrypted session or WebView storage;
- does not call the private Voice token endpoint;
- does not embed LiveKit for DinoVietnam Voice;
- does not include a Voice tab or background-start setting.

A future `DinoVietnamVoiceProvider` can be implemented without changing the
rest of the Hub once DinoVietnam supplies the public OAuth/PKCE client and
player-scoped Voice broker described in `INTEGRATION_OPTIONS.md`.

The Vietnamese translation feature is independent of Voice. It opens the
official translation installer already present on the user's machine or links
to DinoVietnam's public translation release. It does not copy, decrypt or
bundle the encrypted translation archive.
