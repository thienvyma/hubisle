# Implementation status

Version 2.4.1 ships a small companion tab for the official DinoVietnam client.
It safely:

- locates `DinoVietNam.exe` from the registered `dinovietnam:` protocol or the
  usual Desktop locations;
- reports whether the official process is running;
- checks only whether the official settings contain a SteamID and encrypted
  session, without reading, decrypting, returning or copying their values;
- opens the installed client for Steam sign-in and the original Voice flow;
- links to the official release when the client is absent.

The Hub does not call the private Voice token endpoint, copy a reusable app
credential, or embed a second LiveKit connection. DinoVietnam's client keeps
ownership of room authorization, microphone, noise suppression and PTT.

A future `DinoVietnamVoiceProvider` can be implemented without changing the
rest of the Hub once DinoVietnam supplies the public OAuth/PKCE client and
player-scoped Voice broker described in `INTEGRATION_OPTIONS.md`.

The Vietnamese translation feature remains independent of Voice. It opens the
official translation installer already present on the user's machine or links
to DinoVietnam's public translation release. It does not copy, decrypt or
bundle the encrypted translation archive.
