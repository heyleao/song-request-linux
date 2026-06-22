# Roadmap

## Milestone 1: Local Shell

- [x] Local server.
- [x] Health endpoint.
- [x] JSON status endpoint.
- [x] OBS overlay page.
- [x] Config/cache/state directory creation.
- [ ] Log redaction helper.

## Milestone 1.5: Local Song Request Loop

- [x] In-memory queue.
- [x] `!sr`, `!song`, `!skip` parser.
- [x] API endpoint to simulate song requests.
- [x] API endpoint to simulate Twitch chat commands.
- [x] YouTube URL parsing for `youtube.com/watch` and `youtu.be`.
- [x] Overlay reads current song from app state.

## Milestone 1.6: GUI Configuration

- [x] Save default provider from GUI.
- [x] Save Spotify Client ID from GUI.
- [x] Save Twitch bot username/channel from GUI.
- [x] Save Twitch bot token in local state file with restricted permissions.
- [ ] Move secrets to Secret Service/KWallet/libsecret.

## Milestone 2: Twitch Chat

- [ ] Twitch OAuth PKCE.
- [x] Development bot config through environment variables.
- [x] Secure Twitch IRC websocket connection.
- [x] Chat connection.
- [x] `!sr`, `!song`, `!fila`, `!vol`, `!skip`.
- [ ] Cooldowns and permissions.

## Milestone 3: Spotify

- Spotify OAuth PKCE.
- Search.
- Current song.
- Add to queue.
- Duration and explicit filters.
- Choose fallback playlist for when there are no requests.
- Start/resume fallback playlist when request queue is empty.

## Milestone 4: YouTube

- Import/store YouTube API key securely.
- Parse YouTube URLs.
- Search through YouTube Data API v3.
- Cache metadata.
- Block lives, long videos and playlist abuse.

## Milestone 5: Packaging

- AppImage.
- Flatpak.
- AUR package.
- GitHub Actions release workflow.
