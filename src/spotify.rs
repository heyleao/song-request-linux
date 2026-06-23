use std::{
    collections::HashSet,
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distr::Alphanumeric, Rng};
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};
use url::Url;

use crate::{
    config::{AppConfig, UiConfigView},
    song_requests::{RequestSource, SongRequest, SpotifyTrackRef},
};

const AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const API_URL: &str = "https://api.spotify.com/v1";
const SCOPES: &str =
    "user-read-private user-read-playback-state user-modify-playback-state user-read-currently-playing playlist-read-private playlist-read-collaborative";

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
    pub product: Option<String>,
    pub premium: Option<bool>,
    pub product_check_error: Option<String>,
    pub redirect_uri: String,
    pub scopes: &'static str,
    pub fallback_enabled: bool,
    pub fallback_playlist: Option<SpotifyFallbackPlaylist>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpotifyFallbackPlaylist {
    pub id: String,
    pub name: String,
    pub uri: String,
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

#[derive(Debug, Deserialize)]
struct PlaylistsResponse {
    items: Vec<SpotifyPlaylistItem>,
}

#[derive(Debug, Deserialize)]
struct SpotifyQueueResponse {
    currently_playing: Option<SpotifyPlayable>,
    queue: Vec<SpotifyPlayable>,
}

#[derive(Clone, Debug, Deserialize)]
struct SpotifyPlayable {
    name: Option<String>,
    artists: Option<Vec<SpotifyArtist>>,
    duration_ms: Option<u64>,
    #[serde(rename = "type")]
    item_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyPlaybackResponse {
    is_playing: bool,
    item: Option<SpotifyPlayable>,
    context: Option<SpotifyPlaybackContext>,
    progress_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct SpotifyPlaybackContext {
    uri: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpotifyQueueSnapshot {
    pub currently_playing: Option<String>,
    pub upcoming: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SpotifyPlayback {
    pub title: String,
    pub is_playing: bool,
    pub context_uri: Option<String>,
    pub progress_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyDevicesResponse {
    pub devices: Vec<SpotifyDevice>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpotifyDevice {
    pub id: Option<String>,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub volume_percent: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpotifyPlaylistItem {
    pub id: String,
    pub name: String,
    pub uri: String,
    pub tracks: SpotifyPlaylistTracks,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpotifyPlaylistTracks {
    pub total: u64,
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

#[derive(Debug, Deserialize)]
struct SpotifyCurrentUserProfile {
    product: Option<String>,
}

pub async fn connection_status(
    config: &AppConfig,
    token: Option<&mut SpotifyToken>,
) -> SpotifyConnectionStatus {
    let token_configured = token.is_some();
    let mut product = None;
    let mut product_check_error = None;
    if let Some(token) = token {
        match current_user_product(config, token).await {
            Ok(Some(value)) => product = Some(value),
            Ok(None) => {
                product_check_error =
                    Some("Spotify profile did not include product; reconnect with user-read-private scope".to_string());
            }
            Err(error) => product_check_error = Some(error.to_string()),
        }
    }
    let premium = product.as_ref().map(|value| value == "premium");

    SpotifyConnectionStatus {
        client_id_configured: config.spotify.client_id.is_some(),
        token_configured,
        product,
        premium,
        product_check_error,
        redirect_uri: config.spotify.redirect_uri.clone(),
        scopes: SCOPES,
        fallback_enabled: UiConfigView::load(&config.paths).spotify_fallback_enabled,
        fallback_playlist: load_fallback_playlist(config).ok().flatten(),
    }
}

pub async fn current_user_product(
    config: &AppConfig,
    token: &mut SpotifyToken,
) -> Result<Option<String>> {
    refresh_if_needed(config, token).await?;

    let response = Client::new()
        .get(format!("{API_URL}/me"))
        .bearer_auth(&token.access_token)
        .send()
        .await
        .context("failed to read Spotify profile")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            "spotify profile request failed"
        );
        bail!("spotify profile failed with {status}: {body}");
    }

    Ok(response.json::<SpotifyCurrentUserProfile>().await?.product)
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

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            "spotify token exchange failed"
        );
        bail!("spotify token exchange failed with {status}: {body}");
    }

    Ok(token_from_response(response.json().await?))
}

pub async fn search_and_queue(
    config: &AppConfig,
    token: &mut SpotifyToken,
    query: &str,
) -> Result<SongRequest> {
    refresh_if_needed(config, token).await?;

    let track = match SpotifyTrackRef::parse(query) {
        Some(track_ref) => get_track(token, &track_ref.id).await?,
        None => search_track(token, query).await?,
    };
    let title = format_track(&track);
    info!(
        query = %query,
        track = %track.name,
        uri = %track.uri,
        "spotify track resolved"
    );
    if spotify_queue_contains(token, &title).await? {
        info!(track = %title, "spotify track already present; skipping duplicate queue add");
    } else {
        add_to_queue(token, &track.uri).await?;
    }

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

pub async fn list_playlists(
    config: &AppConfig,
    token: &mut SpotifyToken,
) -> Result<Vec<SpotifyPlaylistItem>> {
    refresh_if_needed(config, token).await?;

    let response = Client::new()
        .get(format!("{API_URL}/me/playlists"))
        .bearer_auth(&token.access_token)
        .query(&[("limit", "50")])
        .send()
        .await
        .context("failed to list Spotify playlists")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            "spotify playlists request failed"
        );
        bail!("spotify playlists failed with {status}: {body}. Reconnect Spotify if playlist scopes were just added.");
    }

    Ok(response.json::<PlaylistsResponse>().await?.items)
}

pub async fn list_devices(
    config: &AppConfig,
    token: &mut SpotifyToken,
) -> Result<Vec<SpotifyDevice>> {
    refresh_if_needed(config, token).await?;
    Ok(fetch_devices(token).await?.devices)
}

pub async fn queue_snapshot(
    config: &AppConfig,
    token: &mut SpotifyToken,
) -> Result<SpotifyQueueSnapshot> {
    refresh_if_needed(config, token).await?;

    let response = Client::new()
        .get(format!("{API_URL}/me/player/queue"))
        .bearer_auth(&token.access_token)
        .send()
        .await
        .context("failed to read Spotify queue")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            "spotify queue snapshot failed"
        );
        bail!("spotify queue snapshot failed with {status}: {body}");
    }

    let queue = response.json::<SpotifyQueueResponse>().await?;
    let currently_playing = queue.currently_playing.and_then(format_playable);
    let mut seen = HashSet::new();
    if let Some(current) = &currently_playing {
        seen.insert(normalize(current));
    }
    let upcoming = queue
        .queue
        .into_iter()
        .filter_map(format_playable)
        .filter(|title| seen.insert(normalize(title)))
        .take(5)
        .collect();

    Ok(SpotifyQueueSnapshot {
        currently_playing,
        upcoming,
    })
}

pub async fn current_playback(
    config: &AppConfig,
    token: &mut SpotifyToken,
) -> Result<Option<SpotifyPlayback>> {
    refresh_if_needed(config, token).await?;

    let response = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(format!("{API_URL}/me/player"))
        .bearer_auth(&token.access_token)
        .send()
        .await
        .context("failed to read Spotify playback")?;

    let status = response.status();
    if status == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            "spotify playback snapshot failed"
        );
        bail!("spotify playback snapshot failed with {status}: {body}");
    }

    let playback = response.json::<SpotifyPlaybackResponse>().await?;
    let Some(item) = playback.item else {
        return Ok(None);
    };
    let duration_ms = item.duration_ms;
    let Some(title) = format_playable(item) else {
        return Ok(None);
    };

    Ok(Some(SpotifyPlayback {
        title,
        is_playing: playback.is_playing,
        context_uri: playback.context.and_then(|context| context.uri),
        progress_ms: playback.progress_ms,
        duration_ms,
    }))
}

pub async fn current_volume(config: &AppConfig, token: &mut SpotifyToken) -> Result<Option<u8>> {
    refresh_if_needed(config, token).await?;
    Ok(fetch_devices(token)
        .await?
        .devices
        .into_iter()
        .find(|device| device.is_active)
        .and_then(|device| device.volume_percent))
}

pub async fn set_volume(config: &AppConfig, token: &mut SpotifyToken, level: u8) -> Result<u8> {
    refresh_if_needed(config, token).await?;
    ensure_available_device(token).await?;

    let response = Client::builder()
        .http1_only()
        .build()?
        .put(format!("{API_URL}/me/player/volume"))
        .bearer_auth(&token.access_token)
        .query(&[("volume_percent", level.min(100))])
        .header(CONTENT_LENGTH, "0")
        .body(Vec::new())
        .send()
        .await
        .context("failed to set Spotify volume")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            level,
            "spotify volume change failed"
        );
        bail!("Spotify nao aceitou mudar o volume ({status}): {body}");
    }

    Ok(level.min(100))
}

pub async fn play_context(
    config: &AppConfig,
    token: &mut SpotifyToken,
    context_uri: &str,
) -> Result<()> {
    refresh_if_needed(config, token).await?;
    let device = ensure_available_device(token).await?;

    let response = Client::builder()
        .http1_only()
        .build()?
        .put(format!("{API_URL}/me/player/play"))
        .bearer_auth(&token.access_token)
        .query(&[("device_id", device.id.as_deref().unwrap_or_default())])
        .header(CONTENT_TYPE, "application/json")
        .json(&serde_json::json!({ "context_uri": context_uri }))
        .send()
        .await
        .context("failed to start Spotify context")?;

    let status = response.status();
    if status.is_success() {
        info!(context_uri, "spotify context playback started");
        return Ok(());
    }

    let body = response.text().await.unwrap_or_default();
    warn!(
        status = %status,
        response = %body,
        context_uri,
        "spotify context playback failed"
    );
    bail!("Spotify nao aceitou iniciar a playlist ({status}): {body}");
}

pub async fn resume_playback(config: &AppConfig, token: &mut SpotifyToken) -> Result<()> {
    refresh_if_needed(config, token).await?;
    let device = ensure_available_device(token).await?;
    player_empty_request(
        token,
        reqwest::Method::PUT,
        "me/player/play",
        device.id.as_deref(),
    )
    .await
}

pub async fn pause_playback(config: &AppConfig, token: &mut SpotifyToken) -> Result<()> {
    refresh_if_needed(config, token).await?;
    let device = ensure_available_device(token).await?;
    player_empty_request(
        token,
        reqwest::Method::PUT,
        "me/player/pause",
        device.id.as_deref(),
    )
    .await
}

pub async fn skip_next(config: &AppConfig, token: &mut SpotifyToken) -> Result<()> {
    refresh_if_needed(config, token).await?;
    ensure_available_device(token).await?;
    player_empty_request(token, reqwest::Method::POST, "me/player/next", None).await
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

pub fn load_fallback_playlist(config: &AppConfig) -> Result<Option<SpotifyFallbackPlaylist>> {
    let path = fallback_playlist_path(config);
    if !path.exists() {
        return Ok(None);
    }

    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(Some(serde_json::from_str(&data)?))
}

pub fn save_fallback_playlist(
    config: &AppConfig,
    playlist: &SpotifyFallbackPlaylist,
) -> Result<()> {
    fs::create_dir_all(&config.paths.config_dir)?;
    let path = fallback_playlist_path(config);
    fs::write(&path, serde_json::to_vec_pretty(playlist)?)?;
    Ok(())
}

fn token_path(config: &AppConfig) -> std::path::PathBuf {
    config.paths.state_dir.join("spotify-token.json")
}

fn fallback_playlist_path(config: &AppConfig) -> std::path::PathBuf {
    config
        .paths
        .config_dir
        .join("spotify-fallback-playlist.json")
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

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            "spotify token refresh failed"
        );
        bail!(
            "Spotify precisa ser reconectado: o token salvo nao foi aceito ({status}). Abra Configuracao > Spotify > Login Spotify. Detalhe: {body}"
        );
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
        .query(&[("type", "track"), ("limit", "10"), ("q", query)])
        .send()
        .await
        .context("failed to search Spotify")?;

    if !response.status().is_success() {
        bail!("spotify search failed with {}", response.status());
    }

    let tracks = response.json::<SearchResponse>().await?.tracks.items;
    choose_best_track(query, tracks).ok_or_else(|| anyhow!("no Spotify track found for query"))
}

async fn get_track(token: &SpotifyToken, track_id: &str) -> Result<SpotifyTrack> {
    let response = Client::new()
        .get(format!("{API_URL}/tracks/{track_id}"))
        .bearer_auth(&token.access_token)
        .send()
        .await
        .context("failed to read Spotify track")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("spotify track lookup failed with {status}: {body}");
    }

    response
        .json::<SpotifyTrack>()
        .await
        .context("failed to parse Spotify track")
}

async fn spotify_queue_contains(token: &SpotifyToken, title: &str) -> Result<bool> {
    let response = Client::new()
        .get(format!("{API_URL}/me/player/queue"))
        .bearer_auth(&token.access_token)
        .send()
        .await
        .context("failed to read Spotify queue before adding duplicate guard")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            "spotify duplicate guard queue snapshot failed"
        );
        return Ok(false);
    }

    let target = normalize(title);
    let queue = response.json::<SpotifyQueueResponse>().await?;
    let current_matches = queue
        .currently_playing
        .and_then(format_playable)
        .is_some_and(|current| normalize(&current) == target);
    if current_matches {
        return Ok(true);
    }

    Ok(queue
        .queue
        .into_iter()
        .filter_map(format_playable)
        .any(|queued| normalize(&queued) == target))
}

fn choose_best_track(query: &str, tracks: Vec<SpotifyTrack>) -> Option<SpotifyTrack> {
    let query_tokens = tokenize(query);
    let mut ranked = tracks
        .into_iter()
        .enumerate()
        .map(|(index, track)| {
            let score = score_track(&query_tokens, &track, index);
            debug!(
                score,
                track = %track.name,
                artists = %artist_names(&track).join(", "),
                "spotify search candidate"
            );
            (score, track)
        })
        .collect::<Vec<_>>();

    ranked.sort_by_key(|item| std::cmp::Reverse(item.0));
    ranked.into_iter().map(|(_, track)| track).next()
}

fn score_track(query_tokens: &[String], track: &SpotifyTrack, index: usize) -> i64 {
    let title_tokens = tokenize(&track.name);
    let artist_tokens = tokenize(&artist_names(track).join(" "));
    let combined_tokens = title_tokens
        .iter()
        .chain(artist_tokens.iter())
        .cloned()
        .collect::<HashSet<_>>();

    let title_set = title_tokens.iter().cloned().collect::<HashSet<_>>();
    let artist_set = artist_tokens.iter().cloned().collect::<HashSet<_>>();

    let mut score = 0;
    for token in query_tokens {
        if title_set.contains(token) {
            score += 12;
        }
        if artist_set.contains(token) {
            score += 10;
        }
        if combined_tokens.contains(token) {
            score += 4;
        } else {
            score -= 18;
        }
    }

    let query_joined = query_tokens.join(" ");
    let title_joined = title_tokens.join(" ");
    let artist_joined = artist_tokens.join(" ");
    let combined_joined = format!("{artist_joined} {title_joined}");

    if title_joined == query_joined {
        score += 80;
    }
    if combined_joined == query_joined {
        score += 120;
    }
    if combined_joined.contains(&query_joined) {
        score += 35;
    }
    if !query_tokens.is_empty()
        && query_tokens
            .iter()
            .all(|token| combined_tokens.contains(token))
    {
        score += 60;
    }

    score - index as i64
}

fn artist_names(track: &SpotifyTrack) -> Vec<String> {
    track
        .artists
        .iter()
        .map(|artist| artist.name.clone())
        .collect()
}

fn format_track(track: &SpotifyTrack) -> String {
    format!("{} - {}", track.name, artist_names(track).join(", "))
}

fn tokenize(value: &str) -> Vec<String> {
    normalize(value)
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect()
}

async fn add_to_queue(token: &SpotifyToken, uri: &str) -> Result<()> {
    match add_to_queue_once(token, uri).await {
        Ok(()) => Ok(()),
        Err(error) if error.no_active_device => {
            let device = transfer_to_available_device(token).await?;
            info!(
                device = %device.name,
                device_type = %device.device_type,
                "spotify playback transferred to available device"
            );
            add_to_queue_once(token, uri)
                .await
                .map_err(|error| error.error)?;
            Ok(())
        }
        Err(error) => Err(error.error),
    }
}

async fn add_to_queue_once(token: &SpotifyToken, uri: &str) -> Result<(), SpotifyQueueError> {
    debug!(uri = %uri, "spotify add-to-queue request");
    let response = Client::builder()
        .http1_only()
        .build()
        .map_err(SpotifyQueueError::technical)?
        .post(format!("{API_URL}/me/player/queue"))
        .bearer_auth(&token.access_token)
        .query(&[("uri", uri)])
        .header(CONTENT_LENGTH, "0")
        .header(CONTENT_TYPE, "application/json")
        .body(Vec::new())
        .send()
        .await
        .map_err(|error| {
            SpotifyQueueError::technical(anyhow!("failed to add Spotify track to queue: {error}"))
        })?;

    let status = response.status();
    if status.is_success() {
        info!(uri = %uri, "spotify track added to queue");
        return Ok(());
    }

    let body = response.text().await.unwrap_or_default();
    warn!(
        status = %status,
        response = %body,
        uri = %uri,
        "spotify add-to-queue failed"
    );
    Err(SpotifyQueueError {
        no_active_device: status.as_u16() == 404 && body.contains("NO_ACTIVE_DEVICE"),
        error: anyhow!(
            "Spotify nao encontrou um device de stream ativo ({status}). Abra o Spotify no PC da live; celular nao e usado para playback do app. Resposta: {body}"
        ),
    })
}

async fn fetch_devices(token: &SpotifyToken) -> Result<SpotifyDevicesResponse> {
    let response = Client::new()
        .get(format!("{API_URL}/me/player/devices"))
        .bearer_auth(&token.access_token)
        .send()
        .await
        .context("failed to list Spotify devices")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            "spotify devices request failed"
        );
        bail!("spotify devices failed with {status}: {body}");
    }

    Ok(response.json::<SpotifyDevicesResponse>().await?)
}

async fn transfer_to_available_device(token: &SpotifyToken) -> Result<SpotifyDevice> {
    let devices = fetch_devices(token).await?.devices;
    let device = devices
        .iter()
        .find(|device| is_allowed_stream_device(device) && device.is_active)
        .or_else(|| devices.iter().find(|device| is_allowed_stream_device(device)))
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "Nenhum device Spotify de stream disponivel. Abra o Spotify no PC da live; o app nao transfere playback para celular."
            )
        })?;

    let device_id = device.id.as_ref().expect("device id checked");
    let response = Client::new()
        .put(format!("{API_URL}/me/player"))
        .bearer_auth(&token.access_token)
        .json(&serde_json::json!({
            "device_ids": [device_id],
            "play": false
        }))
        .send()
        .await
        .context("failed to transfer Spotify playback")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            status = %status,
            response = %body,
            device = %device.name,
            "spotify transfer playback failed"
        );
        bail!(
            "Spotify abriu o device {}, mas falhou ao transferir playback ({status}): {body}",
            device.name
        );
    }

    Ok(device)
}

fn is_allowed_stream_device(device: &SpotifyDevice) -> bool {
    !device.is_restricted
        && device.id.is_some()
        && device.device_type.eq_ignore_ascii_case("computer")
}

async fn ensure_available_device(token: &SpotifyToken) -> Result<SpotifyDevice> {
    let devices = fetch_devices(token).await?.devices;
    if let Some(device) = devices
        .iter()
        .find(|device| is_allowed_stream_device(device) && device.is_active)
        .cloned()
    {
        return Ok(device);
    }

    transfer_to_available_device(token).await
}

async fn player_empty_request(
    token: &SpotifyToken,
    method: reqwest::Method,
    path: &str,
    device_id: Option<&str>,
) -> Result<()> {
    let mut request = Client::builder()
        .http1_only()
        .build()?
        .request(method, format!("{API_URL}/{path}"))
        .bearer_auth(&token.access_token)
        .header(CONTENT_LENGTH, "0")
        .body(Vec::new());
    if let Some(device_id) = device_id {
        request = request.query(&[("device_id", device_id)]);
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("failed Spotify player request {path}"))?;

    let status = response.status();
    if status.is_success() {
        info!(path, "spotify player command succeeded");
        return Ok(());
    }

    let body = response.text().await.unwrap_or_default();
    warn!(
        status = %status,
        response = %body,
        path,
        "spotify player command failed"
    );
    bail!("Spotify nao aceitou o comando {path} ({status}): {body}");
}

fn format_playable(item: SpotifyPlayable) -> Option<String> {
    let name = item.name?;
    let artists = item
        .artists
        .unwrap_or_default()
        .into_iter()
        .map(|artist| artist.name)
        .collect::<Vec<_>>();

    if artists.is_empty() {
        let item_type = item.item_type.unwrap_or_else(|| "item".to_string());
        Some(format!("{name} ({item_type})"))
    } else {
        Some(format!("{} - {}", name, artists.join(", ")))
    }
}

struct SpotifyQueueError {
    no_active_device: bool,
    error: anyhow::Error,
}

impl SpotifyQueueError {
    fn technical(error: impl Into<anyhow::Error>) -> Self {
        Self {
            no_active_device: false,
            error: error.into(),
        }
    }
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

    #[test]
    fn ranking_prefers_title_artist_match_over_first_result() {
        let tracks = vec![
            SpotifyTrack {
                name: "Aerials".to_string(),
                uri: "spotify:track:aerials".to_string(),
                artists: vec![SpotifyArtist {
                    name: "System Of A Down".to_string(),
                }],
            },
            SpotifyTrack {
                name: "Spiders".to_string(),
                uri: "spotify:track:spiders".to_string(),
                artists: vec![SpotifyArtist {
                    name: "System Of A Down".to_string(),
                }],
            },
        ];

        let track = choose_best_track("system of a down spiders", tracks).expect("track");

        assert_eq!(track.uri, "spotify:track:spiders");
    }
}
