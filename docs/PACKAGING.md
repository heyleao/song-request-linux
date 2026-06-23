# Packaging and Install Strategy

Goal: make Song Request Linux feel installable for new Linux users while keeping
streamer tokens, API keys, OAuth flows and local playback controls safe.

## Security Baseline

- Bind the app dashboard and APIs to `127.0.0.1` by default.
- Keep OAuth callbacks on localhost only.
- Never package user config, `.env`, `.secrets`, logs, queue state or tokens.
- Store public config in `~/.config/song-request-linux/config.json`.
- Store secrets/tokens in `~/.local/state/song-request-linux/` until a keyring
  backend is implemented.
- Keep destructive actions, like shutdown and queue clear, behind explicit UI
  controls and confirmation headers/API methods.
- Treat Pear Desktop, Spotify and Twitch as external integrations. The app should
  diagnose them and guide the user instead of silently installing or modifying
  accounts.

## New User Path

Preferred first-run flow:

1. Install/open `Song Request Linux` from the app menu.
2. Dashboard opens automatically at `http://127.0.0.1:7384/`.
3. User chooses one active provider: Spotify or YouTube/Pear.
4. User connects Twitch bot via OAuth.
5. User connects Spotify or configures YouTube/Pear.
6. Dashboard shows copyable OBS URLs for overlay/player.
7. User tests `!sr` from the dashboard before going live.

## Current Installer

For source installs, use:

```bash
./scripts/install-user-friendly --with-pear
```

Useful modes:

```bash
./scripts/install-user-friendly --all
./scripts/install-user-friendly --no-deps
./scripts/install-user-friendly --no-start
```

What it does:

- optionally installs Arch/CachyOS dependencies through `pacman`;
- builds the release binary;
- installs a desktop launcher in `~/.local/share/applications`;
- installs the SRL icon in `~/.local/share/icons/hicolor/512x512/apps`;
- opens the dashboard.

## Windows-like Release Targets

### Phase 1: Source installer

Status: implemented.

This is the safest early path because it does not hide what is installed and
keeps all runtime data in normal XDG paths.

### Phase 2: Arch/CachyOS package

Create `song-request-linux-bin` or `song-request-linux-git` packaging that:

- installs the binary to `/usr/bin/song-request-linux`;
- installs `song-request-linux-open` and `song-request-linux-stop`;
- installs `song-request-linux.desktop`;
- installs `assets/logo-srl.png` as an icon;
- declares optional dependencies: `pear-desktop`, `yt-dlp`.

### Phase 3: AppImage

Use AppImage for double-click distribution:

- bundle only this app and required runtime libraries;
- do not bundle user secrets;
- do not bundle Pear Desktop;
- launch the local backend and open the dashboard;
- write config/state to XDG paths outside the AppImage.

### Phase 4: Tauri shell

Use Tauri to make the dashboard feel like a native desktop app:

- native window instead of browser tab;
- backend process lifecycle managed by the app;
- tray/menu support;
- no remote content by default;
- strict allowlist for shell/open commands;
- same localhost backend for OBS URLs.

## Do Not Do

- Do not ask users to paste access tokens into chat or logs.
- Do not store client secrets in the repository.
- Do not expose the dashboard on `0.0.0.0` by default.
- Do not install browser extensions or modify Discord/OBS configs silently.
- Do not auto-delete user queues/configs during updates.
