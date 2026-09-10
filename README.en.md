# Isle Pulse Overlay

[Tiếng Việt](README.md) · **English**

**Isle Pulse Overlay 2.0.2** is developed by **Huỳnh Vỹ**. Its in-game minimap
reads the player's coordinates automatically from the server's account
website. Adapters currently support **Era Gaming VN**, **The Real Server VN
(Titan)**, and **IslePilot**.

On first launch, enter the server website and sign in in the separate window.
The session is encrypted on this computer. Unsupported sites require a specific
adapter; the app never guesses an API or sends cookies to an unknown domain.

![Minimap with a friend marker, stat bars, Prime quests and the voice strip over the running game — the dino wears a skin just changed in the app](docs/screenshot-ingame.jpg)

![Full map with place names and POI layers](docs/screenshot-fullmap.png)

![Your Dino tab with stats and Prime progress](docs/screenshot-dino.png)

![Garage (Gacha) tab: a parked dino in 3D with growth, stats and a Restore button](docs/screenshot-garage.png)

![Per-region skin palette applied directly to the current dino](docs/screenshot-skin.png)

![Voice tab: proximity voice over IsleVOIP, device pickers and who is in earshot](docs/screenshot-voip.png)

## Features

- **Circular minimap** pinned to a corner of the game window, click-through so it
  never blocks play. North stays up, with an arrow showing your direction of travel.
- **Full map**: smooth zoom/pan, 12 toggleable layers (fresh water, water, salt licks,
  mud wallows, sanctuaries, migration zones, AI patrol zones, food zones, animals
  with per-species icons 🐗🦌🐢, region names, landmarks, and a live **server
  POI** layer from IslePilot), with place names drawn directly on the map; a
  collapsible layer list and a clear-trail button to declutter mid-session.
  Opening the map by hotkey lands on the map tab.
- **3 basemap styles**: Vulnona captures (default) or the hand-drawn
  [IsleMaps](https://www.islemaps.com/) light/dark art — switch in Settings,
  applies to both the full map and the minimap. The IsleMaps art tracks a newer
  game build and shows the SE archipelago (Hell's Mouth).
- **Waypoints**: right-click to drop, rename/recolor, delete, quick icons
  (💀 death spot, 🏠 nest…); the minimap gets a rim arrow with bearing +
  distance to the nearest one.
- **Search & navigation**: search places/waypoints, paste coordinates to jump
  there, follow mode with an edge arrow leading back to your position.
- **Travel trail** recorded per session, with the previous session's path restored.
- **Realtime position and stats**: coordinates, growth, health, hunger, thirst,
  and stamina are normalized from the selected provider and rendered by the same
  minimap. Missing provider fields remain empty rather than being fabricated.
- **Multi-provider Garage**: IslePilot keeps its 3D cards and Park/Restore/
  Rename/Sell flow; Era and Titan get five official API slots with store,
  restore, delete, connection gating, and server job progress.
- **Skin editor in the hub**: seven current Era/Titan color regions, presets,
  local drafts, Era's 16-color/full-color permission, Titan variation, and a
  server-authoritative cooldown. IslePilot has no published overlay skin API yet.
- **Global hotkeys** rebindable in-app and a bilingual Vietnamese/English UI.

## Install

Run the NSIS installer built from this custom branch.
On first launch the app downloads the map data (~3 MB) to your machine.

Requires **Windows 10/11 64-bit**. WebView2 is already present on most Windows 11
installs; the installer fetches it if missing.

> Windows may show a SmartScreen warning because the installer is not
> code-signed. Click **More info → Run anyway**.

### Automatic updates

The app checks at startup, hourly, and when the network reconnects.
Use **Check for updates** in the footer to check manually or retry.
The notification includes release notes, download progress, and a **Later** button.

The app checks releases at [github.com/thienvyma/hubisle](https://github.com/thienvyma/hubisle).
When a newer version is available, an in-app banner can download and install it.
Every package must carry Huỳnh Vỹ's updater signature; modified packages and
packages published from another source are rejected.

## Connecting "Your Dino" (IslePilot)

The **Dino** tab reads your own dino's stats (growth, health, hunger, thirst,
stamina, nutrition, Prime progress) from the IslePilot system. Two ways to
connect:

**Method 1 — Steam login via IslePilot (recommended):** open the Dino tab →
click **Steam login** → sign in in the islepilot.eu window that opens; it
closes itself when done. Do this **once** — no server link needed, it works on
**every IslePilot server**, and switching servers in game follows automatically.
This login also unlocks the **Garage (Gacha)** tab and the **server POI** map
layer. If the window fails to catch the token, open *"Or paste the token
manually"* and paste the token (or the whole `theisle-overlay://…` link).

**Method 2 — Legacy: server link + cookie** (only when method 1 does not work;
the cookie is stored per server, so switching servers means doing it again).
Open the **"Legacy"** section of the login card, enter the server link and
click Steam login there; if that still fails, paste the cookie manually:

1. Open the server page in your browser and sign in with Steam there. Press
   **F12** (or right-click → **Inspect**) and open the **Application** tab
   (Chrome) / **Storage** (Firefox).

   ![Open DevTools and pick the Application tab](docs/guide-dino-1-devtools.png)

2. Pick **Cookies** → the server's domain → click the **`islepilot_player`**
   cookie → copy the whole **Value**.

   ![Copy the islepilot_player cookie value](docs/guide-dino-2-copy-cookie.jpg)

3. In the app: paste it into the cookie box → click **Verify & save cookie**.

   ![Enter the server link, paste the cookie and save](docs/guide-dino-3-paste-app.jpg)

If the server runs a **live map**, the app detects it and enables automatic
position — no manual coordinate copying needed; when the server has the live
map disabled the option locks itself off.

**Some servers using IslePilot** (examples — any IslePilot-powered server works):

- https://mixi.islepilot.eu
- https://hoho.islepilot.eu
- https://sdvn.islepilot.eu
- https://sdvn2.islepilot.eu
- https://khunglong.islepilot.eu
- https://islepilot.eu/p/sbtcisland

## How light is it?

Measured on a real machine: **Intel Core i5-14400F (10 cores / 16 threads), 32 GB
RAM, RTX 3060 Ti, Windows 11 Pro build 26200, 100% display scaling** — release
build v1.0.0:

| Item | Size |
|---|---|
| Installer | **4.3 MB** |
| Installed executable | 17.8 MB |
| Map data downloaded on first run | 2.9 MB (2.6 MB basemap + 0.3 MB point data) |
| **Total disk footprint** | **~21 MB** |

| At runtime | RAM (working set) | Idle CPU |
|---|---|---|
| Full map **and** minimap open | **522 MB** (8 processes) | 0.18% |
| Full map hidden with `Ctrl+Alt+F` (the while-playing scenario) | **448 MB** | 0.08% |

**CPU is essentially zero** because the app has no repaint loop — it draws only
when new data arrives.

## Things to know

1. **Game display mode**: no out-of-process overlay can draw over **Exclusive
   Fullscreen** — a Windows limitation. Use **Windowed** or **Borderless
   Fullscreen**. The app reads your game config and warns you if the mode is wrong.
2. **Position does not update by itself**: you press `Tab` → **Asset Location** in
   game whenever you want a position update. This is *deliberate* — see the
   anti-cheat section below.
3. **Heading needs two coordinate copies** at least 20 m apart; samples older than
   10 minutes expire so the arrow never points the wrong way.
4. **Only one instance can run** — global hotkeys are system-exclusive, so two
   copies would fight over them.
5. **Low-RAM machines**: hide the full map with `Ctrl+Alt+F` while playing —
   the app trims the hidden window's memory. Clicking X parks the app in the
   **system tray** (icon next to the clock), Steam/Discord-style — left-click
   the icon to bring it back, right-click → Quit to exit fully.
6. **Hotkeys taken by another app** are reported at startup; rebind them in Settings.
7. **The "Your dino" feature** supports IslePilot-based servers — see
   [Connecting "Your Dino"](#connecting-your-dino-islepilot). The recommended
   Steam-login mode reads a stable JSON API; only the legacy server + cookie
   mode parses the server's web pages, so that path **can break whenever
   IslePilot changes their markup** — the app flags it when it detects a new
   deployment. If this part fails, the map features are **unaffected**.
8. **Ask your server admins** before using it routinely — some servers have their
   own rules about third-party tools. Auto-position only turns on when the app
   detects that the server runs a live map; it locks itself off when the live map
   is disabled, and a manual choice you make is always respected.
9. **Your login token/cookie** is encrypted with Windows DPAPI and can only be
   decrypted by your Windows account on that machine.
10. **SmartScreen** warns on first install because the installer is not code-signed
   (certificates cost a yearly fee). Later auto-updates are not prompted again.

## Anti-cheat safety

The game runs kernel-level Easy Anti-Cheat. This app is safe because it
**never touches the game process**:

- Realtime position arrives over **HTTPS from the signed-in server management
  site**; the clipboard remains a manual fallback when a provider has no feed.
- Hotkeys use `RegisterHotKey` (Windows' cooperative API), **not** a keyboard
  hook.
- Stats, position, accepted friends, Garage and skins come over **HTTPS from the
  exact verified provider** (Era, Titan or IslePilot), without reading game memory.
- The `` ` `` shortcut uses `SendInput` only to send the fixed `/unstuck` chat
  command after verifying that The Isle is foreground. The app does not inject
  DLLs, hook DirectX, or capture packets.

CI greps for any forbidden API call site (`scripts/check-forbidden-apis.ps1`).
The allowed-call list lives at the top of `src-tauri/src/win/mod.rs`.

## Development

Requirements: Node 22+, Rust stable (MSVC), WebView2.

```powershell
npm install
npx tauri dev                        # run dev

# Drive the UI without the game running:
$env:THEISLE_REPLAY = "path\to\replay_sample.txt"; npx tauri dev

# Tests
npm run check                        # svelte-check
cd src-tauri; cargo test --workspace # all Rust tests
cargo clippy --workspace -- -D warnings
..\scripts\check-forbidden-apis.ps1

# After each in-game map update (run for every downloaded basemap):
cargo run --bin verify_data --features devtools -- --source vulnona
cargo run --bin verify_data --features devtools -- --source islemaps-light
cargo run --bin verify_data --features devtools -- --source islemaps-dark
cargo test -p theisle-overlay --lib -- --ignored parse_real_cache
```

Note: `.cargo/config.toml` moves the `target-dir` outside the OneDrive-synced
folder.

## Credits

Map data is **fetched on first run, never bundled** — it is a personal copy on
your machine, not a redistribution.

- Basemap: [VulnonaMAP](https://vulnona.com/game/map/) (Coco.N) — stitched
  from in-game captures. Imagery copyright Afterthought LLC (The Isle).
- IsleMaps basemap (optional, downloaded only when selected in Settings) and
  animal spawn points: [islemaps.com](https://www.islemaps.com/) (Pont & Emeara).
- POIs: [myislemap.com](https://myislemap.com/), VulnonaMAP, wiredredman's
  Steam guide.

Unaffiliated with Afterthought LLC.

## Credits

Developed by **Huỳnh Vỹ**.

- 💬 Facebook: https://www.facebook.com/thienvyma
- 💻 GitHub: https://github.com/thienvyma/hubisle
