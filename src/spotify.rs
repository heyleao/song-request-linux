use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distr::Alphanumeric, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::{
    config::AppConfig,
    song_requests::{RequestSource, SongRequest},
};

const AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const API_URL: &str = "https://api.spotify.com/v1";
const SCOPES: &str =
    "user-read-playback-state user-modify-playback-state user-read-currently-playing";

#[derive(Clone, Debug)]
pub struct SpotifyAuthSession {
    pub state: String,
    pub verifier: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpotifyToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SpotifyAuthStart {
    pub auth_url: String,
}

#[derive(Debug, Serialize)]
pub struct SpotifyConnectionStatus {
    pub client_id_configured: bool,
    pub token_configured: bool,
    pub redirect_uri: String,
    pub scopes: &'static str,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    tracks: SearchTracks,
}

#[derive(Debug, Deserialize)]
struct SearchTracks {
    items: Vec<SpotifyTrack>,
}

#[derive(Clone, Debug, Deserialize)]
struct SpotifyTrack {
    name: String,
    uri: String,
    artists: Vec<SpotifyArtist>,
}

#[derive(Clone, Debug, Deserialize)]
struct SpotifyArtist {
    name: String,
}

pub fn connection_status(
    config: &AppConfig,
    token: Option<&SpotifyToken>,
) -> SpotifyConnectionStatus {
    SpotifyConnectionStatus {
        client_id_configured: config.spotify.client_id.is_some(),
        token_configured: token.is_some(),
        redirect_uri: config.spotify.redirect_uri.clone(),
        scopes: SCOPES,
    }
}

pub fn start_auth(config: &AppConfig) -> Result<(SpotifyAuthStart, SpotifyAuthSession)> {
    let client_id = config
        .spotify
        .client_id
        .as_ref()
        .context("SPOTIFY_CLIENT_ID is not configured")?;
    let verifier = random_string(64);
    let state = random_string(32);
    let challenge = pkce_challenge(&verifier);

    let mut url = Url::parse(AUTH_URL)?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("scope", SCOPES)
        .append_pair("redirect_uri", &config.spotify.redirect_uri)
        .append_pair("state", &state)
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", &challenge)
        .append_pair("show_dialog", "true");

    Ok((
        SpotifyAuthStart {
            auth_url: url.to_string(),
        },
        SpotifyAuthSession { state, verifier },
    ))
}

pub async fn exchange_code(
    config: &AppConfig,
    session: SpotifyAuthSession,
    callback_state: &str,
    code: &str,
) -> Result<SpotifyToken> {
    if session.state != callback_state {
        bail!("spotify callback state mismatch");
    }

    let client_id = config
        .spotify
        .client_id
        .as_ref()
        .context("SPOTIFY_CLIENT_ID is not configured")?;

    let response = Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", config.spotify.redirect_uri.as_str()),
            ("client_id", client_id.as_str()),
            ("code_verifier", session.verifier.as_str()),
        ])
        .send()
        .await
        .context("failed to exchange Spotify authorization code")?;

    if !response.status().is_success() {
        bail!("spotify token exchange failed with {}", response.status());
    }

    Ok(token_from_response(response.json().await?))
}

pub async fn search_and_queue(
    config: &AppConfig,
    token: &mut SpotifyToken,
    query: &str,
) -> Result<SongRequest> {
    refresh_if_needed(config, token).await?;

    let track = search_track(token, query).await?;
    add_to_queue(token, &track.uri).await?;

    Ok(SongRequest {
        id: 0,
        requester: String::new(),
        query: query.to_string(),
        source: RequestSource::Spotify {
            uri: track.uri.clone(),
        },
        title: track.name,
        artist: track
            .artists
            .into_iter()
            .map(|artist| artist.name)
            .collect::<Vec<_>>()
            .join(", "),
    })
}

pub fn load_token(config: &AppConfig) -> Result<Option<SpotifyToken>> {
    let path = token_path(config);
    if !path.exists() {
        return Ok(None);
    }

    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(Some(serde_json::from_str(&data)?))
}

pub fn save_token(config: &AppConfig, token: &SpotifyToken) -> Result<()> {
    fs::create_dir_all(&config.paths.state_dir)?;
    let path = token_path(config);
    fs::write(&path, serde_json::to_vec_pretty(token)?)?;
    restrict_file_permissions(&path);
    Ok(())
}

fn token_path(config: &AppConfig) -> std::path::PathBuf {
    config.paths.state_dir.join("spotify-token.json")
}

async fn refresh_if_needed(config: &AppConfig, token: &mut SpotifyToken) -> Result<()> {
    let now = unix_now();
    if token.expires_at > now + 60 {
        return Ok(());
    }

    let refresh_token = token
        .refresh_token
        .as_ref()
        .context("Spotify refresh token is missing")?;
    let client_id = config
        .spotify
        .client_id
        .as_ref()
        .context("SPOTIFY_CLIENT_ID is not configured")?;

    let response = Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("client_id", client_id.as_str()),
        ])
        .send()
        .await
        .context("failed to refresh Spotify token")?;

    if !response.status().is_success() {
        bail!("spotify token refresh failed with {}", response.status());
    }

    let refreshed = token_from_response(response.json().await?);
    token.access_token = refreshed.access_token;
    token.expires_at = refreshed.expires_at;
    token.scope = refreshed.scope;
    if refreshed.refresh_token.is_some() {
        token.refresh_token = refreshed.refresh_token;
    }

    save_token(config, token)?;
    Ok(())
}

async fn search_track(token: &SpotifyToken, query: &str) -> Result<SpotifyTrack> {
    let response = Client::new()
        .get(format!("{API_URL}/search"))
        .bearer_auth(&token.access_token)
        .query(&[("type", "track"), ("limit", "1"), ("q", query)])
        .send()
        .await
        .context("failed to search Spotify")?;

    if !response.status().is_success() {
        bail!("spotify search failed with {}", response.status());
    }

    response
        .json::<SearchResponse>()
        .await?
        .tracks
        .items
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no Spotify track found for query"))
}

async fn add_to_queue(token: &SpotifyToken, uri: &str) -> Result<()> {
    let response = Client::new()
        .post(format!("{API_URL}/me/player/queue"))
        .bearer_auth(&token.access_token)
        .query(&[("uri", uri)])
        .send()
        .await
        .context("failed to add Spotify track to queue")?;

    if response.status().is_success() {
        return Ok(());
    }

    bail!(
        "spotify queue failed with {}. Confirm Spotify Premium and an active device.",
        response.status()
    );
}

fn token_from_response(response: TokenResponse) -> SpotifyToken {
    SpotifyToken {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        expires_at: unix_now() + response.expires_in,
        scope: response.scope,
    }
}

fn random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

#[cfg(unix)]
fn restrict_file_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &std::path::Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_is_url_safe() {
        let challenge = pkce_challenge("abc");

        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));
    }
}
