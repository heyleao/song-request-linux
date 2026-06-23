# Song Request Linux

Linux-first Twitch song request app for Spotify and YouTube.

The app runs as a local web dashboard, works well on Linux/Wayland, and avoids
Wine/WebView2/Windows-only dependencies. It is inspired by the streamer workflow
of Songify, but it is not a fork and is not affiliated with Songify or
Songify.Rocks.

## What Works

- Twitch bot commands for song requests and player control.
- Spotify OAuth, search, queue control, playback control and fallback playlist selection.
- Hybrid request routing: Spotify search by default, YouTube links as YouTube requests.
- YouTube text search validation with duration/category policy through YouTube Data API v3.
- YouTube playback through Pear Desktop API or the local OBS browser source.
- Local OBS overlay at `http://127.0.0.1:7384/overlay`.
- Local YouTube player source at `http://127.0.0.1:7384/player`.
- Local web dashboard with tabs for overview, queue, commands, player, logs, setup and guide.
- Desktop-style launcher with single-instance behavior and clean shutdown.

## Setup

Use the dashboard setup tab first:

```text
http://127.0.0.1:7384/
```

Public setup guide:

[docs/SETUP.md](docs/SETUP.md)

UI direction:

[docs/UI_REFERENCES.md](docs/UI_REFERENCES.md)

Packaging and security-minded install plan:

[docs/PACKAGING.md](docs/PACKAGING.md)

The dashboard guide links directly to:

- Spotify Developer Dashboard
- Twitch Developer Console
- Google Cloud Credentials

## Run

Install dependencies on CachyOS/Arch:

```bash
./scripts/install-cachyos-deps --with-pear
```

Use `--all` instead if you also want the OBS Browser Source fallback through
`yt-dlp`.

For the easiest local install, use the friendly installer:

```bash
./scripts/install-user-friendly --with-pear
```

Manual desktop entry install remains available:

```bash
./scripts/install-desktop-entry
./scripts/song-request-linux-open
```

Stop the app:

```bash
./scripts/song-request-linux-stop
```

You can also stop it from the dashboard with `Encerrar`.

## Distribution

The simplest supported distribution path for now is CachyOS/Arch:

```bash
git clone https://github.com/heyleao/song-request-linux.git
cd song-request-linux
./scripts/install-cachyos-deps --with-pear
./scripts/install-desktop-entry
./scripts/song-request-linux-open
```

This keeps Pear Desktop as an external player dependency instead of bundling it
inside this app. That makes updates, audio capture, and Linux desktop
integration simpler. A packaged release can later wrap these same steps in an
Arch package or AppImage.

## URLs

- Dashboard: `http://127.0.0.1:7384/`
- OBS overlay: `http://127.0.0.1:7384/overlay`
- OBS YouTube player/audio source: `http://127.0.0.1:7384/player`
- Queue API: `http://127.0.0.1:7384/api/queue`
- Events API: `http://127.0.0.1:7384/api/events`
- Diagnostics API: `http://127.0.0.1:7384/api/diagnostics`
- Health check: `http://127.0.0.1:7384/health`

## Commands

Everyone:

```text
!sr nome da musica
!sr https://youtu.be/VIDEO_ID
!song
!fila
!queue
!q
!vol
!comandos
```

Moderator/broadcaster:

```text
!skip
!vol 30
!play
!pause
!next
```

## Request Routing

- Plain song names use the default provider, usually Spotify.
- YouTube links are detected as exact YouTube requests and do not need search filtering.
- YouTube text searches are checked against the configured duration limit.
- By default, searched YouTube videos must be marked as Music by YouTube metadata.
- Non-music YouTube search results can be allowed from the setup tab when needed.

Spotify plays through your active Spotify device. For YouTube, the recommended
mode is Pear Desktop with its API Server enabled at
`http://127.0.0.1:26538/api/v1`; the app sends YouTube requests to Pear's queue.
The local browser player at `http://127.0.0.1:7384/player` remains available as
the OBS Browser Source fallback and requires `yt-dlp` on the system.

## Local Data

Public config:

```text
~/.config/song-request-linux/config.json
```

Local secrets/tokens:

```text
~/.local/state/song-request-linux/
```

Do not commit real tokens, API keys, OAuth codes, exported config, `.env` files,
or private planning/security notes.

## Development

Run checks:

```bash
cargo fmt
cargo test
cargo check
```

Run directly without the launcher:

```bash
cargo run
```

Simulate a chat command:

```bash
curl -X POST http://127.0.0.1:7384/api/chat-command \
  -H 'content-type: application/json' \
  -d '{"requester":"viewer","message":"!song"}'
```

## License

GPL-3.0-or-later.
