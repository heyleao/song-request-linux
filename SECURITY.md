# Security Policy

## Secret Handling

The project must never log or commit:

- access tokens
- refresh tokens
- OAuth authorization codes
- client secrets
- YouTube API keys
- Twitch/Spotify private app credentials

Secrets should be stored in Secret Service, KWallet, libsecret or another Linux
keyring provider when available. Plain-text fallback storage is not acceptable
for production builds.

## OAuth Rules

- Use PKCE for Spotify and Twitch.
- Bind callback listeners to `127.0.0.1` only.
- Validate OAuth `state` on every callback.
- Detect port conflicts and show actionable errors.
- Redact sensitive query parameters in logs.

## Reporting

During early development, report issues privately to the repository owner before
opening public issues that include logs.

