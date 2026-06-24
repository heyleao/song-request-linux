<p align="center">
  <img src="assets/logo-srl.png" alt="Song Request Linux" width="220">
</p>

<h1 align="center">Song Request Linux</h1>

<p align="center">
  Twitch song requests for Linux streamers, with Spotify, YouTube/Pear, OBS overlay, and a local dashboard.
</p>

<p align="center">
  <a href="README.md">Português</a> ·
  <a href="docs/SETUP_EN.md">Quick setup</a> ·
  <a href="#install">Install</a> ·
  <a href="#how-it-works">How it works</a> ·
  <a href="#chat-commands">Chat commands</a> ·
  <a href="#obs">OBS</a>
</p>

---

Song Request Linux is a local app for streamers who want Twitch chat song requests on Linux. It runs on `127.0.0.1`, opens a browser dashboard, controls the queue, and provides OBS overlays.

The app avoids Windows-only dependencies, WebView2, and Wine. The current flow is simple: choose one active provider, connect the Twitch bot, configure the player, and add the OBS overlay.

## Install

On CachyOS/Arch:

```bash
git clone https://github.com/heyleao/song-request-linux.git
cd song-request-linux
./scripts/install-user-friendly --with-pear
```

Open:

```bash
./scripts/song-request-linux-open
```

Stop:

```bash
./scripts/song-request-linux-stop
```

Update from GitHub:

```bash
./scripts/update-from-github --restart
```

Uninstall while keeping local config, tokens, logs, and queue:

```bash
./scripts/uninstall-user
```

Remove everything, including local data:

```bash
./scripts/uninstall-user --remove-data
```

### Portable `.tar.gz`

The Linux `.tar.gz` package includes the compiled app. Regular users do not need Rust, Cargo, or Git.

```bash
tar -xzf song-request-linux-0.1.16-linux-x86_64.tar.gz
cd song-request-linux-0.1.16-linux-x86_64
./scripts/check-runtime-prereqs
./scripts/install-desktop-entry
./scripts/song-request-linux-open
```

### Experimental Windows `.zip`

Download the `.zip`, extract it into a simple folder, and open:

```text
Start-SongRequestLinux.cmd
```

The dashboard opens at `http://127.0.0.1:7384/`. To stop it, use `Shutdown` in the dashboard or `Stop-SongRequestLinux.cmd`.

Windows local data paths:

```text
%APPDATA%\song-request-linux
%LOCALAPPDATA%\song-request-linux
```

## How It Works

![Song Request Linux dashboard](docs/images/dashboard-overview.png)

1. Open the dashboard.
2. On the `Live` screen, choose the mode in the `Active provider` card:
   - `Spotify`: text requests search Spotify.
   - `YouTube/Pear`: text requests search YouTube and play through Pear.
3. Go to `Setup`.
4. Configure only the block for the provider you chose.
5. Connect the Twitch bot.
6. Save settings.
7. Test a request in the dashboard or chat.
8. Add the OBS overlay.

Important: the app currently works best with one active provider at a time. If the provider is Spotify, text goes to Spotify. If it is YouTube/Pear, text goes to YouTube. YouTube links are still detected as YouTube.

## Spotify

Use Spotify when you want Spotify search requests and fallback playlist.

Requirements:

- Spotify Premium.
- Spotify app open on the stream PC.
- An active device on the PC before accepting requests.
- `Client ID` from the Spotify Developer Dashboard.

Spotify redirect URI:

```text
http://127.0.0.1:7384/auth/spotify/callback
```

The app avoids transferring playback to phones. If there is no valid local device, the dashboard shows an error and asks you to open/play something in Spotify on the stream PC.

## YouTube/Pear

Use YouTube/Pear when you want YouTube requests.

Requirements:

- YouTube Data API v3 enabled.
- API key saved in the dashboard.
- Pear Desktop open if Pear mode is selected.
- Pear `API Server` plugin enabled.

Recommended Pear config:

```text
Port: 26538
Authorization: No Authorization
API: http://127.0.0.1:26538/api/v1
```

The app sends requests to Pear's queue. Pear is an external app; if it is closed or the API is disabled, a request can enter SRL's queue but will not play until Pear is available again.

## Chat Commands

![Advanced command and limit setup](docs/images/advanced-setup.png)


Default commands:

```text
!sr song name
!sr youtube_link
!song
!queue / !fila / !q
!remove / !rm
!vol
!vol 30
!skip
!play
!pause / !stop
!next / !pular
```

Commands, aliases, access levels, and per-role limits can be changed in the dashboard.

## OBS

Now-playing overlay URL:

```text
http://127.0.0.1:7384/overlay?max=48&width=520&size=24&lines=2
```

Recommended Browser Source size:

```text
Width: 620
Height: 150
```

The `width=520` parameter keeps text inside the overlay. Use `lines=2` for two song-title lines. The top label can be changed in Setup or with `label=Text` in the URL.

YouTube Browser Source player, only when using Browser Source playback instead of Pear:

```text
http://127.0.0.1:7384/player
```

## License

GPL-3.0-or-later.
