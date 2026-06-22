# Song Request Linux

Linux-first Twitch song request app for Spotify and YouTube.

This project is inspired by the streamer workflow of Songify, but it is not a
fork and is not affiliated with Songify or Songify.Rocks. The goal is to build a
native Linux app that works cleanly on Wayland, OBS Flatpak, Spotify, YouTube
and Twitch without Wine, WebView2, WPF or Windows APIs.

## Goals

- Twitch chat commands such as `!sr`, `!song` and `!skip`.
- Spotify support through OAuth PKCE.
- YouTube support through YouTube Data API v3.
- Local OBS overlay through `http://127.0.0.1:<port>/overlay`.
- Secure token storage using the Linux keyring when available.
- Clear diagnostics for ports, tokens, permissions and API quota.
- Low CPU/RAM usage while idle.

## Current Status

Early scaffold. The first milestone is a local HTTP server with:

- health endpoint
- JSON status endpoint
- OBS browser overlay
- storage layout
- security rules for secrets

## Development

Install dependencies on CachyOS/Arch:

```bash
sudo pacman -S --needed rust cargo github-cli git pkgconf openssl
```

Run locally:

```bash
cd ~/songify-linux
cargo run
```

Then open:

- App status: `http://127.0.0.1:7384/api/status`
- Diagnostics: `http://127.0.0.1:7384/api/diagnostics`
- Queue: `http://127.0.0.1:7384/api/queue`
- OBS overlay: `http://127.0.0.1:7384/overlay`
- Health check: `http://127.0.0.1:7384/health`

Simulate a song request:

```bash
curl -X POST http://127.0.0.1:7384/api/song-requests \
  -H 'content-type: application/json' \
  -d '{"requester":"bruno","query":"https://youtu.be/dQw4w9WgXcQ"}'
```

Simulate a Twitch chat command:

```bash
curl -X POST http://127.0.0.1:7384/api/chat-command \
  -H 'content-type: application/json' \
  -d '{"requester":"viewer","message":"!sr daft punk one more time"}'
```

Run checks:

```bash
cargo fmt
cargo test
cargo check
```

## Security

Never commit:

- Twitch tokens
- Spotify tokens
- YouTube API keys
- OAuth authorization codes
- client secrets
- exported user configs with secrets

Use `.env.example` only for public development defaults.

## License

GPL-3.0-or-later.
