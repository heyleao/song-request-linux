# Setup Guide

This app runs locally at `http://127.0.0.1:7384/`.

## 1. Spotify

Create an app at:

```text
https://developer.spotify.com/dashboard
```

Use this redirect URI:

```text
http://127.0.0.1:7384/auth/spotify/callback
```

Copy the Spotify Client ID into `Connections -> Spotify Client ID`, save, then
use `Gerar link de login`.

Required Spotify behavior:

- The account must be Premium.
- A Spotify device must be open/available.
- If no active device exists, open Spotify and press play/pause once.

## 2. Twitch Bot

Create an app at:

```text
https://dev.twitch.tv/console/apps
```

Use this redirect URI:

```text
https://localhost:7443/auth/twitch/callback
```

Client type:

```text
Public
```

In `Connections`, fill:

- Twitch Client ID
- Twitch Bot Username
- Twitch Channel

Then use `Conectar bot` in a private browser window logged into the bot account.

## 3. YouTube

Create or reuse a Google Cloud project, enable YouTube Data API v3, then create
an API key at:

```text
https://console.cloud.google.com/apis/credentials
```

In `Connections`, fill:

- YouTube API Key
- Maximo YouTube, default `360` seconds
- Accept non-music videos only if you want to allow manual exceptions

YouTube links are validated with video metadata:

- duration must be under the configured limit;
- category must be Music unless non-music is allowed.

## 4. Commands

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
!stop
!next
```

## 5. Start And Stop

Start:

```bash
./scripts/song-request-linux-open
```

Stop:

```bash
./scripts/song-request-linux-stop
```

Or use the dashboard `Encerrar` button.
