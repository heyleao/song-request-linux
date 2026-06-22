# UI/UX Direction

Song Request Linux uses a local operational dashboard, not a landing page.

The main user is a streamer who may already have OBS, Twitch chat and a game
open. The UI must be fast to scan, explicit about state and careful with
secrets.

## Principles

- First screen is the real control panel.
- Keep controls direct and visible.
- Prefer short labels over long instructions.
- Show clear states: configured, connected, waiting, error.
- Never reveal tokens, API keys, client secrets or OAuth codes.
- Keep secret fields masked and report only whether they are configured.
- Errors must say what failed and what action fixes it.
- Use local browser UI served on `127.0.0.1`.
- Keep OBS overlay separate from the dashboard.
- Optimize for 1366x768, 1920x1080 and narrow mobile widths.

## Dashboard Sections

- Status bar: Twitch, provider, queue count and overlay.
- Setup: bot account, channel and provider.
- Connections/config: provider, Spotify Client ID, Twitch bot username/channel and masked token.
- Test command: simulate `!sr`, `!song` and `!skip`.
- Now playing: current song and requester.
- Queue: upcoming requests.
- Diagnostics: config, storage, token presence and local endpoints.

## Security UX

- Do not display secret values after entry.
- Do not save secrets in files intended for version control.
- Make configured-but-hidden states explicit.
- Avoid logs in the UI unless redacted.
- Use copy buttons only for safe values such as overlay URLs.

## MVP Interaction

1. User opens the dashboard.
2. User sees whether Twitch bot settings are configured.
3. User tests `!sr` locally.
4. User opens the OBS overlay.
5. User watches current song and queue update.
6. User can skip when needed.
