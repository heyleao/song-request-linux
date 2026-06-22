# Song Request Linux

Linux-first Twitch song request app for Spotify and YouTube.

This project is inspired by the streamer workflow of Songify, but it is not a
fork and is not affiliated with Songify or Songify.Rocks. The goal is to build a
native Linux app that works cleanly on Wayland, OBS Flatpak, Spotify, YouTube
and Twitch without Wine, WebView2, WPF or Windows APIs.

## Goals

- Twitch chat commands such as `!sr`, `!song`, `!fila`, `!vol` and `!skip`.
- Spotify support through OAuth PKCE.
- YouTube support through YouTube Data API v3.
- Local OBS overlay through `http://127.0.0.1:<port>/overlay`.
- Secure token storage using the Linux keyring when available.
- Clear diagnostics for ports, tokens, permissions and API quota.
- Low CPU/RAM usage while idle.

## Security

See [SECURITY.md](SECURITY.md).

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

Run like a desktop app:

```bash
cd ~/songify-linux
./scripts/install-desktop-entry
./scripts/song-request-linux-open
```

The launcher starts one instance, opens `http://127.0.0.1:7384/`, and reuses the
running app if it is already open. Use the dashboard `Encerrar` button or run:

```bash
./scripts/song-request-linux-stop
```

This avoids orphaned CLI processes holding ports `7384` and `7443`.

Run with a Twitch bot for `!sr`:

```bash
export TWITCH_BOT_USERNAME="your_bot_account"
export TWITCH_BOT_OAUTH_TOKEN="oauth_token_without_oauth_prefix"
export TWITCH_CHANNEL="your_channel"
cargo run
```

The Twitch bot currently supports:

- `!sr <spotify search or youtube link>`
- `!song`
- `!fila`, `!queue` or `!q`
- `!vol` to read Spotify volume
- `!vol <0-100>` for moderators/broadcaster badges
- `!play`, `!resume` for moderators/broadcaster badges
- `!pause`, `!stop`, `!parar` for moderators/broadcaster badges
- `!next`, `!pular` for moderators/broadcaster badges
- `!comandos`, `!commands` or `!help`
- `!skip` for moderators/broadcaster badges

Command access is intentionally simple:

- Everyone: `!sr`, `!song`, `!fila`, `!queue`, `!q`, `!vol`, `!comandos`.
- Moderators/broadcaster: `!skip`, `!vol <0-100>`, `!play`, `!pause`, `!stop`, `!next`.

When Spotify is connected, queue and now-playing status come from Spotify first.
The app does not restore old runtime queue state after restart, so played songs
do not come back as pending app requests.

Hybrid routing:

- Plain song names/search terms use the default provider, usually Spotify.
- YouTube links are detected and kept as YouTube requests in the app queue.
- Spotify cannot play YouTube URLs directly; YouTube requests are shown as
  pending app items until the dedicated YouTube player integration is added.

Run with Spotify queue control:

```bash
export SONG_REQUEST_PROVIDER="spotify"
export SPOTIFY_CLIENT_ID="your_spotify_app_client_id"
cargo run
```

Open `http://127.0.0.1:7384/connections`, generate the Spotify login link and
open it in a private window if you need to avoid the wrong browser account.
Register this redirect URI in your Spotify developer app:

```text
http://127.0.0.1:7384/auth/spotify/callback
```

The connections page also lets you choose a Spotify fallback playlist. Reconnect
Spotify if playlist loading asks for updated scopes.

Most development settings can be saved from `http://127.0.0.1:7384/connections`:

- default provider
- Spotify Client ID
- Twitch Client ID
- Twitch bot username
- Twitch channel
- Twitch bot OAuth through a private-window login flow

Public config is stored in `~/.config/song-request-linux/config.json`.
Temporary local secrets are stored in `~/.local/state/song-request-linux/secrets.json`
with restricted file permissions. Secret Service/KWallet is planned.

For Twitch bot OAuth, register this redirect URI in your Twitch app:

```text
https://localhost:7443/auth/twitch/callback
```

Use the connections page to generate the bot login link and open it in a private
window when you need to avoid reusing the streamer's browser session.
The app keeps the dashboard on HTTP and starts a local HTTPS callback listener
for Twitch. The certificate is self-signed and stored under
`~/.local/state/song-request-linux/tls/`, so your browser may ask you to confirm
the local certificate the first time.

Then open:

- Dashboard: `http://127.0.0.1:7384/`
- Live events: `http://127.0.0.1:7384/api/events`
- App status: `http://127.0.0.1:7384/api/status`
- Diagnostics: `http://127.0.0.1:7384/api/diagnostics`
- Queue: `http://127.0.0.1:7384/api/queue`
- OBS overlay: `http://127.0.0.1:7384/overlay`
- Health check: `http://127.0.0.1:7384/health`

The dashboard is organized as a local web app with tabs for overview, queue,
commands, player controls, live logs and setup. The live log is in memory only
and is intended for operational feedback during a stream.

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

For the development Twitch bot, keep `TWITCH_BOT_OAUTH_TOKEN` in your shell or
keyring. Do not save a real token in `.env.example`, docs, commits or logs.

## License

GPL-3.0-or-later.
