# Architecture

## Runtime Shape

The app is a lightweight local service bound to `127.0.0.1`.

Initial modules:

- `server`: local HTTP API and OBS overlay.
- `storage`: config, cache, logs and secret-provider abstraction.
- `auth_spotify`: Spotify OAuth PKCE and token refresh.
- `auth_twitch`: Twitch OAuth PKCE and token refresh.
- `twitch_chat`: chat connection and command dispatch.
- `song_requests`: queue, cooldowns, validation and permissions.
- `spotify_player`: Spotify search, metadata, playback and queue.
- `youtube_search`: YouTube URL parsing, Data API lookup, metadata and cache.
- `youtube_player`: initial external player integration; future MPRIS/Pear support.
- `overlay`: browser source state and WebSocket updates.
- `diagnostics`: system checks and user-facing troubleshooting.

## Filesystem Layout

- Config: `~/.config/song-request-linux/`
- Cache: `~/.cache/song-request-linux/`
- Logs: `~/.local/state/song-request-linux/logs/`
- Secrets: Linux keyring provider

## OBS

OBS should use a Browser Source pointed at:

```text
http://127.0.0.1:7384/overlay
```

The overlay must not require internet access to render basic state.

