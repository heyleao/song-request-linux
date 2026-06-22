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

## Milestone 2: Twitch Chat

- Twitch OAuth PKCE.
- Chat connection.
- `!sr`, `!song`, `!skip`.
- Cooldowns and permissions.

## Milestone 3: Spotify

- Spotify OAuth PKCE.
- Search.
- Current song.
- Add to queue.
- Duration and explicit filters.

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
