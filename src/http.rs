use anyhow::anyhow;
use axum::{
    extract::{DefaultBodyLimit, Path as AxumPath, Query, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::map_response,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::atomic::Ordering, time::Duration};
use tokio::{process::Command, time::timeout};
use tower_http::trace::TraceLayer;

use crate::{
    commands::{parse_chat_command, ChatCommand, ChatCommandInput, ChatUserRole, PlaybackAction},
    config::{self, YoutubePlayback},
    connections, dashboard,
    diagnostics::DiagnosticsResponse,
    display, overlay, pear, player, request_flow,
    song_requests::{
        MusicProvider, QueuePersistence, QueueView, RequestSource, SongRequest, SongRequestInput,
        YoutubeRequestPlayback,
    },
    spotify,
    state::{AppState, HealthResponse, StatusResponse},
    twitch_auth,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(dashboard::page))
        .route("/connections", get(connections::page))
        .route("/auth/spotify/callback", get(spotify_callback))
        .route("/auth/twitch/callback", get(twitch_callback))
        .route("/health", get(health))
        .route("/assets/logo-srl.png", get(logo_srl))
        .route("/assets/i18n/:lang", get(i18n_asset))
        .route("/favicon.png", get(logo_srl))
        .route("/api/shutdown", post(shutdown))
        .route("/api/status", get(status))
        .route("/api/events", get(events).delete(clear_events))
        .route("/api/diagnostics", get(diagnostics))
        .route("/api/update", post(update_from_github))
        .route("/api/update/status", get(update_status))
        .route("/api/update/latest", get(update_latest))
        .route("/api/update/installed", get(update_installed))
        .route("/api/config", get(get_config).post(save_config))
        .route("/api/config/export", get(export_config))
        .route("/api/config/import", post(import_config))
        .route("/api/connections/status", get(connections_status))
        .route("/api/connections/spotify/start", post(spotify_start))
        .route("/api/connections/twitch/start", post(twitch_start))
        .route("/api/connections/twitch/token", post(twitch_token))
        .route("/api/spotify/devices", get(spotify_devices))
        .route("/api/pear/status", get(pear_status))
        .route("/api/spotify/playlists", get(spotify_playlists))
        .route(
            "/api/spotify/fallback-playlist",
            post(spotify_fallback_playlist),
        )
        .route("/api/queue", get(queue).delete(clear_queue))
        .route("/api/queue/:id", delete(remove_queue_item))
        .route("/api/song-requests", post(add_song_request))
        .route("/api/chat-command", post(chat_command))
        .route("/api/skip", post(skip))
        .route("/api/volume", get(volume_status).post(set_volume))
        .route("/overlay", get(overlay::page))
        .route("/player", get(player::page))
        .route("/api/player/youtube", get(youtube_player_current))
        .route("/api/player/youtube/audio", post(youtube_player_audio))
        .route("/api/player/youtube/start", post(youtube_player_start))
        .route("/api/player/youtube/finish", post(youtube_player_finish))
        .route("/api/player/youtube/event", post(youtube_player_event))
        .fallback(not_found)
        .layer(map_response(add_security_headers))
        .layer(DefaultBodyLimit::max(128 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn add_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; connect-src 'self' http://127.0.0.1:* https://127.0.0.1:*; img-src 'self' data:; media-src 'self' http: https: blob:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; frame-ancestors 'self'",
        ),
    );
    response
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn logo_srl() -> impl IntoResponse {
    static LOGO: &[u8] = include_bytes!("../assets/logo-srl.png");
    ([(CONTENT_TYPE, "image/png")], LOGO)
}

async fn i18n_asset(AxumPath(lang): AxumPath<String>) -> impl IntoResponse {
    let fallback = match lang.as_str() {
        "en-US.json" => Some(include_str!("../i18n/en-US.json")),
        "pt-BR.json" => Some(include_str!("../i18n/pt-BR.json")),
        _ => None,
    };
    let Some(fallback) = fallback else {
        return (
            StatusCode::NOT_FOUND,
            [(CONTENT_TYPE, "text/plain")],
            "not found".to_string(),
        );
    };

    let contents = match app_root_dir() {
        Ok(root) => tokio::fs::read_to_string(root.join("i18n").join(&lang))
            .await
            .unwrap_or_else(|_| fallback.to_string()),
        Err(_) => fallback.to_string(),
    };

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json")],
        contents,
    )
}

async fn shutdown(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ShutdownResponse>, ApiError> {
    let allowed = headers
        .get("x-song-request-action")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == "shutdown");
    if !allowed {
        return Err(ApiError::bad_request(anyhow!(
            "shutdown confirmation header missing"
        )));
    }

    state
        .record_event("system", "encerramento solicitado")
        .await;
    let _ = state.shutdown.send(());

    Ok(Json(ShutdownResponse {
        status: "shutting_down",
    }))
}

async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let queue = effective_queue_view(&state).await;
    let provider = current_provider(&state);
    let mut response = StatusResponse::from_queue(&state, queue, provider);

    if response.current_song.is_none()
        && matches!(provider, MusicProvider::Youtube)
        && matches!(current_youtube_playback(&state), YoutubePlayback::Pear)
    {
        if let Ok(song) = pear::now_playing(&state.config).await {
            if !song.is_paused {
                response.current_song = Some(crate::state::SongView {
                    title: song.title.unwrap_or_else(|| "Pear tocando".to_string()),
                    artist: song.artist.unwrap_or_else(|| "YouTube Music".to_string()),
                    requester: "Pear / A seguir".to_string(),
                });
            }
        }
    }

    Json(response)
}

async fn diagnostics(State(state): State<AppState>) -> Json<DiagnosticsResponse> {
    Json(DiagnosticsResponse::collect(&state.config))
}

async fn update_from_github(headers: HeaderMap) -> Result<Json<UpdateResponse>, ApiError> {
    let allowed = headers
        .get("x-song-request-action")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == "update");
    if !allowed {
        return Err(ApiError::bad_request(anyhow!(
            "update confirmation header missing"
        )));
    }

    let app_dir = app_root_dir().map_err(ApiError::bad_request)?;
    let script = app_dir.join("scripts/update-from-github");
    if !script.is_file() {
        return Err(ApiError::bad_request(anyhow!(
            "Esta instalacao nao tem atualizacao pelo GitHub. Use uma instalacao via Git ou baixe o tar.gz mais recente na pagina de releases."
        )));
    }

    let escaped_dir = app_dir.display().to_string().replace('\'', "'\\''");
    Command::new("setsid")
        .arg("sh")
        .arg("-c")
        .arg(format!(
            "cd '{escaped_dir}' && ./scripts/update-from-github --restart"
        ))
        .spawn()
        .map_err(|error| ApiError::bad_request(error.into()))?;

    Ok(Json(UpdateResponse {
        status: "updating",
        message: "Atualizacao iniciada. O app vai reiniciar em alguns segundos.",
    }))
}

fn app_root_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    if let Some(parent) = exe.parent() {
        if parent.file_name().and_then(|name| name.to_str()) == Some("bin") {
            if let Some(root) = parent.parent() {
                return Ok(root.to_path_buf());
            }
        }
        if parent.join("scripts").is_dir() {
            return Ok(parent.to_path_buf());
        }
    }

    Ok(std::env::current_dir()?)
}

async fn update_latest() -> Result<Json<UpdateLatestResponse>, ApiError> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let client = reqwest::Client::builder()
        .user_agent(format!("song-request-linux/{current_version}"))
        .timeout(Duration::from_secs(4))
        .build()
        .map_err(|error| ApiError::bad_request(error.into()))?;
    let response = client
        .get("https://api.github.com/repos/heyleao/song-request-linux/releases/latest")
        .send()
        .await
        .map_err(|error| ApiError::bad_request(error.into()))?;
    if !response.status().is_success() {
        return Err(ApiError::bad_request(anyhow!(
            "GitHub releases returned {}",
            response.status()
        )));
    }

    let release = response
        .json::<GithubReleaseResponse>()
        .await
        .map_err(|error| ApiError::bad_request(error.into()))?;
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let update_available = version_is_newer(&latest_version, &current_version);
    let message = if update_available {
        format!("Nova versao disponivel: v{latest_version}.")
    } else {
        format!("Voce ja esta na versao mais recente: v{current_version}.")
    };

    Ok(Json(UpdateLatestResponse {
        current_version,
        latest_version,
        latest_tag: release.tag_name,
        release_url: release.html_url,
        update_available,
        message,
        changelog: release.body.unwrap_or_default(),
    }))
}

async fn update_installed() -> Result<Json<UpdateInstalledResponse>, ApiError> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let tag = format!("v{current_version}");
    let client = reqwest::Client::builder()
        .user_agent(format!("song-request-linux/{current_version}"))
        .timeout(Duration::from_secs(4))
        .build()
        .map_err(|error| ApiError::bad_request(error.into()))?;
    let response = client
        .get(format!(
            "https://api.github.com/repos/heyleao/song-request-linux/releases/tags/{tag}"
        ))
        .send()
        .await
        .map_err(|error| ApiError::bad_request(error.into()))?;
    if !response.status().is_success() {
        return Err(ApiError::bad_request(anyhow!(
            "GitHub release {tag} returned {}",
            response.status()
        )));
    }

    let release = response
        .json::<GithubReleaseResponse>()
        .await
        .map_err(|error| ApiError::bad_request(error.into()))?;

    Ok(Json(UpdateInstalledResponse {
        current_version,
        current_tag: release.tag_name,
        release_url: release.html_url,
        changelog: release.body.unwrap_or_default(),
    }))
}

async fn update_status(State(state): State<AppState>) -> Json<UpdateStatusResponse> {
    let path = state.config.paths.state_dir.join("update-status.json");
    let log_path = state.config.paths.log_dir.join("update.log");
    let mut status = match tokio::fs::read_to_string(path).await {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|_| UpdateStatusResponse {
            status: "unknown".to_string(),
            message: "Nao foi possivel ler o status da ultima atualizacao.".to_string(),
            before: String::new(),
            after: String::new(),
            timestamp: String::new(),
            log_tail: String::new(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }),
        Err(_) => UpdateStatusResponse {
            status: "none".to_string(),
            message: "Nenhuma atualizacao executada por este painel ainda.".to_string(),
            before: String::new(),
            after: String::new(),
            timestamp: String::new(),
            log_tail: String::new(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };
    status.current_version = env!("CARGO_PKG_VERSION").to_string();
    status.log_tail = read_update_log_tail(&log_path).await;

    Json(status)
}

async fn read_update_log_tail(path: &std::path::Path) -> String {
    match tokio::fs::read_to_string(path).await {
        Ok(contents) => {
            let lines: Vec<String> = contents
                .lines()
                .rev()
                .take(80)
                .map(redact_local_paths)
                .collect();
            lines.into_iter().rev().collect::<Vec<_>>().join(
                "
",
            )
        }
        Err(_) => String::new(),
    }
}

fn redact_local_paths(line: &str) -> String {
    if line.contains("/home/")
        || line.contains("/run/media/")
        || line.contains("/mnt/")
        || line.contains("/tmp/")
    {
        return "[local path redacted]".to_string();
    }

    line.to_string()
}

async fn events(State(state): State<AppState>) -> Json<Vec<crate::state::AppEvent>> {
    Json(state.events.read().await.recent(80))
}

async fn clear_events(State(state): State<AppState>) -> Json<ClearEventsResponse> {
    state.events.write().await.clear();

    Json(ClearEventsResponse { status: "cleared" })
}

async fn get_config(State(state): State<AppState>) -> Json<config::UiConfigView> {
    Json(config::UiConfigView::load(&state.config.paths))
}

async fn export_config(State(state): State<AppState>) -> Json<config::ConfigBackup> {
    Json(config::export_user_config(&state.config.paths))
}

async fn save_config(
    State(state): State<AppState>,
    Json(input): Json<config::UiConfigInput>,
) -> Result<Json<config::UiConfigView>, ApiError> {
    let view = config::save_ui_config(&state.config.paths, input).map_err(ApiError::bad_request)?;
    if view.queue_persistence_enabled {
        save_current_queue_state(&state).await?;
    } else if let Err(error) = std::fs::remove_file(&state.config.paths.queue_file) {
        if error.kind() != std::io::ErrorKind::NotFound {
            return Err(ApiError::bad_request(anyhow::anyhow!(error)));
        }
    }

    Ok(Json(view))
}

async fn import_config(
    State(state): State<AppState>,
    Json(input): Json<config::ConfigBackup>,
) -> Result<Json<config::UiConfigView>, ApiError> {
    let view =
        config::import_user_config(&state.config.paths, input).map_err(ApiError::bad_request)?;

    Ok(Json(view))
}

async fn connections_status(State(state): State<AppState>) -> Json<ConnectionsStatus> {
    let mut spotify_token = state.spotify_token.write().await;

    Json(ConnectionsStatus {
        spotify: spotify::connection_status(&state.config, spotify_token.as_mut()).await,
    })
}

async fn spotify_playlists(
    State(state): State<AppState>,
) -> Result<Json<Vec<spotify::SpotifyPlaylistItem>>, ApiError> {
    let mut token_guard = state.spotify_token.write().await;
    let token = token_guard
        .as_mut()
        .ok_or_else(|| ApiError::bad_request(anyhow::anyhow!("Spotify is not connected")))?;
    let playlists = spotify::list_playlists(&state.config, token)
        .await
        .map_err(ApiError::bad_request)?;

    Ok(Json(playlists))
}

async fn spotify_devices(
    State(state): State<AppState>,
) -> Result<Json<Vec<spotify::SpotifyDevice>>, ApiError> {
    let mut token_guard = state.spotify_token.write().await;
    let token = token_guard
        .as_mut()
        .ok_or_else(|| ApiError::bad_request(anyhow::anyhow!("Spotify is not connected")))?;
    let devices = spotify::list_devices(&state.config, token)
        .await
        .map_err(ApiError::bad_request)?;

    Ok(Json(devices))
}

async fn pear_status(State(state): State<AppState>) -> Json<pear::PearStatus> {
    Json(pear::status(&state.config).await)
}

async fn spotify_fallback_playlist(
    State(state): State<AppState>,
    Json(input): Json<spotify::SpotifyFallbackPlaylist>,
) -> Result<Json<spotify::SpotifyFallbackPlaylist>, ApiError> {
    spotify::save_fallback_playlist(&state.config, &input).map_err(ApiError::bad_request)?;

    Ok(Json(input))
}

async fn spotify_start(
    State(state): State<AppState>,
) -> Result<Json<spotify::SpotifyAuthStart>, ApiError> {
    let (start, session) = spotify::start_auth(&state.config).map_err(ApiError::bad_request)?;
    *state.spotify_auth.write().await = Some(session);

    Ok(Json(start))
}

async fn twitch_start(
    State(state): State<AppState>,
) -> Result<Json<twitch_auth::TwitchAuthStart>, ApiError> {
    Ok(Json(
        twitch_auth::start_auth(&state.config).map_err(ApiError::bad_request)?,
    ))
}

async fn twitch_token(
    State(state): State<AppState>,
    Json(input): Json<twitch_auth::TwitchTokenInput>,
) -> Result<Json<config::UiConfigView>, ApiError> {
    let view = twitch_auth::save_bot_token(&state.config, input)
        .await
        .map_err(ApiError::bad_request)?;
    if let Some(secrets) = config::TwitchBotSecrets::from_env() {
        crate::twitch_chat::spawn_bot(state.clone(), secrets);
    }

    Ok(Json(view))
}

async fn twitch_callback() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Twitch Bot conectado</title>
  <link rel="icon" type="image/png" href="/favicon.png">
  <style>
    :root { color-scheme: dark; font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
    * { box-sizing: border-box; }
    body {
      min-height: 100vh;
      margin: 0;
      display: grid;
      place-items: center;
      padding: 24px;
      background: linear-gradient(180deg, #101720 0%, #080d13 100%);
      color: #f4f7fb;
    }
    main {
      width: min(520px, 100%);
      display: grid;
      justify-items: center;
      gap: 16px;
      text-align: center;
      border: 1px solid #2c3442;
      border-radius: 8px;
      background: #151b24;
      padding: 28px;
      box-shadow: 0 22px 48px rgba(0, 0, 0, .32);
    }
    img { width: 92px; height: 92px; border-radius: 8px; }
    h1 { margin: 0; font-size: 26px; line-height: 1.15; }
    p { margin: 0; color: #aab5c4; line-height: 1.5; }
    .actions { display: flex; flex-wrap: wrap; justify-content: center; gap: 10px; margin-top: 4px; }
    a, button {
      min-height: 40px;
      border: 1px solid #2c3442;
      border-radius: 6px;
      padding: 9px 13px;
      background: #1d2530;
      color: #f4f7fb;
      font-weight: 800;
      text-decoration: none;
      cursor: pointer;
    }
    a.primary { border-color: #4b92d8; background: #5aa9ff; color: #07111f; }
  </style>
</head>
<body>
  <main>
    <img src="/assets/logo-srl.png" alt="Song Request Linux">
    <h1 id="title">Conectando Twitch Bot...</h1>
    <p id="message">Aguarde enquanto salvamos o login do bot.</p>
    <div class="actions">
      <a class="primary" href="/">Voltar ao dashboard</a>
      <button type="button" onclick="window.close()">Fechar aba</button>
    </div>
  </main>
  <script>
    const params = new URLSearchParams(location.hash.slice(1));
    const token = params.get('access_token');
    const message = document.getElementById('message');
    const title = document.getElementById('title');
    async function save() {
      if (!token) {
        title.textContent = 'Falha ao conectar Twitch Bot';
        message.textContent = 'Token nao veio no callback. Tente novamente em janela privada.';
        return;
      }
      try {
        const response = await fetch('/api/connections/twitch/token', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ access_token: token })
        });
        const data = await response.json();
        if (!response.ok) throw new Error(data.error || 'Falha ao salvar token');
        history.replaceState(null, '', '/auth/twitch/callback');
        title.textContent = 'Twitch Bot conectado';
        message.textContent = `Bot conectado como ${data.twitch_bot_username}. Voce ja pode voltar ao dashboard e testar no chat.`;
      } catch (error) {
        title.textContent = 'Falha ao conectar Twitch Bot';
        message.textContent = error.message;
      }
    }
    save();
  </script>
</body>
</html>"#,
    )
}

async fn spotify_callback(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<&'static str>, ApiError> {
    let code = params
        .get("code")
        .ok_or_else(|| ApiError::bad_request(anyhow::anyhow!("missing Spotify code")))?;
    let callback_state = params
        .get("state")
        .ok_or_else(|| ApiError::bad_request(anyhow::anyhow!("missing Spotify state")))?;
    let session =
        state.spotify_auth.write().await.take().ok_or_else(|| {
            ApiError::bad_request(anyhow::anyhow!("Spotify login session expired"))
        })?;

    let token = spotify::exchange_code(&state.config, session, callback_state, code)
        .await
        .map_err(ApiError::bad_request)?;
    spotify::save_token(&state.config, &token).map_err(ApiError::bad_request)?;
    *state.spotify_token.write().await = Some(token);

    Ok(Html(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Spotify conectado</title>
  <link rel="icon" type="image/png" href="/favicon.png">
  <style>
    :root { color-scheme: dark; font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
    * { box-sizing: border-box; }
    body { min-height: 100vh; margin: 0; display: grid; place-items: center; padding: 24px; background: linear-gradient(180deg, #101720 0%, #080d13 100%); color: #f4f7fb; }
    main { width: min(520px, 100%); display: grid; justify-items: center; gap: 16px; text-align: center; border: 1px solid #2c3442; border-radius: 8px; background: #151b24; padding: 28px; box-shadow: 0 22px 48px rgba(0, 0, 0, .32); }
    img { width: 92px; height: 92px; border-radius: 8px; }
    h1 { margin: 0; font-size: 26px; line-height: 1.15; }
    p { margin: 0; color: #aab5c4; line-height: 1.5; }
    .actions { display: flex; flex-wrap: wrap; justify-content: center; gap: 10px; margin-top: 4px; }
    a, button { min-height: 40px; border: 1px solid #2c3442; border-radius: 6px; padding: 9px 13px; background: #1d2530; color: #f4f7fb; font-weight: 800; text-decoration: none; cursor: pointer; }
    a.primary { border-color: #4b92d8; background: #5aa9ff; color: #07111f; }
  </style>
</head>
<body>
  <main>
    <img src="/assets/logo-srl.png" alt="Song Request Linux">
    <h1>Spotify conectado</h1>
    <p>Login salvo com sucesso. Volte ao dashboard para carregar playlists, ativar fallback ou testar um pedido.</p>
    <div class="actions">
      <a class="primary" href="/">Voltar ao dashboard</a>
      <button type="button" onclick="window.close()">Fechar aba</button>
    </div>
  </main>
</body>
</html>"#,
    ))
}

async fn queue(State(state): State<AppState>) -> Json<QueueView> {
    let mut view = effective_queue_view(&state).await;
    view.persistence = Some(queue_persistence(&state).await);
    Json(view)
}

async fn clear_queue(State(state): State<AppState>) -> Result<Json<QueueView>, ApiError> {
    let ids = active_queue_item_ids(&state).await;
    {
        let mut queue = state.queue.write().await;
        for id in ids {
            queue.remove_by_id(id);
        }
        save_queue_state(&state, &queue)?;
    }
    clear_youtube_state_if_no_pending(&state).await;
    state.record_event("queue", "fila ativa zerada").await;

    let mut view = effective_queue_view(&state).await;
    view.persistence = Some(queue_persistence(&state).await);
    Ok(Json(view))
}

async fn active_queue_item_ids(state: &AppState) -> Vec<u64> {
    let view = effective_queue_view(state).await;
    view.current_song
        .into_iter()
        .chain(view.queue)
        .map(|song| song.id)
        .collect()
}

async fn remove_queue_item(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<u64>,
) -> Result<Json<QueueView>, ApiError> {
    let removed = {
        let mut queue = state.queue.write().await;
        let removed = queue.remove_by_id(id);
        save_queue_state(&state, &queue)?;
        removed
    };

    clear_youtube_state_if_no_pending(&state).await;

    match removed {
        Some(song) => {
            state
                .record_event("queue", format!("removido do painel: {}", song.title))
                .await;
        }
        None => {
            state
                .record_event("queue", format!("item {id} nao estava na fila local"))
                .await;
        }
    }

    let mut view = effective_queue_view(&state).await;
    view.persistence = Some(queue_persistence(&state).await);
    Ok(Json(view))
}

async fn youtube_player_current(State(state): State<AppState>) -> Json<YoutubePlayerResponse> {
    Json(youtube_player_response(&state).await)
}

async fn youtube_player_start(
    State(state): State<AppState>,
    Json(input): Json<YoutubePlayerSyncInput>,
) -> Result<Json<YoutubePlayerResponse>, ApiError> {
    if spotify_blocks_youtube(&state).await.is_some() {
        return Ok(Json(youtube_player_response(&state).await));
    }

    let current_song = current_youtube_song(&state).await;
    if current_song
        .as_ref()
        .is_some_and(|song| song.id == input.id)
    {
        *state.youtube_waiting_spotify_title.lock().await = None;
        pause_spotify_for_youtube(&state).await;
    }

    Ok(Json(YoutubePlayerResponse {
        current_song,
        waiting_for_spotify: None,
        paused: state.youtube_browser_paused.load(Ordering::SeqCst),
        volume: state
            .youtube_browser_volume
            .load(Ordering::SeqCst)
            .clamp(1, 100),
    }))
}

async fn youtube_player_audio(
    State(state): State<AppState>,
    Json(input): Json<YoutubePlayerSyncInput>,
) -> Result<Json<YoutubeAudioResponse>, ApiError> {
    if spotify_blocks_youtube(&state).await.is_some() {
        return Err(ApiError::bad_request(anyhow!(
            "aguardando a musica atual do Spotify terminar"
        )));
    }

    let current_song = current_youtube_song(&state)
        .await
        .ok_or_else(|| ApiError::bad_request(anyhow!("nenhum video YouTube ativo")))?;
    if current_song.id != input.id {
        return Err(ApiError::bad_request(anyhow!(
            "video YouTube atual mudou; atualize o player"
        )));
    }

    let audio_url = resolve_youtube_audio_url(&current_song.video_id)
        .await
        .map_err(ApiError::bad_request)?;

    Ok(Json(YoutubeAudioResponse { audio_url }))
}

async fn youtube_player_finish(
    State(state): State<AppState>,
    Json(input): Json<YoutubePlayerSyncInput>,
) -> Result<Json<YoutubePlayerResponse>, ApiError> {
    {
        let mut queue = state.queue.write().await;
        queue.remove_by_id(input.id);
        save_queue_state(&state, &queue)?;
    }

    let current_song = current_youtube_song(&state).await;
    if current_song.is_none() {
        *state.youtube_waiting_spotify_title.lock().await = None;
        resume_spotify_after_youtube(&state).await;
    }

    Ok(Json(YoutubePlayerResponse {
        current_song,
        waiting_for_spotify: None,
        paused: state.youtube_browser_paused.load(Ordering::SeqCst),
        volume: state
            .youtube_browser_volume
            .load(Ordering::SeqCst)
            .clamp(1, 100),
    }))
}

async fn youtube_player_event(
    State(state): State<AppState>,
    Json(input): Json<YoutubePlayerEventInput>,
) -> Json<YoutubePlayerEventResponse> {
    state.record_event("player", input.message).await;

    Json(YoutubePlayerEventResponse { ok: true })
}

async fn add_song_request(
    State(state): State<AppState>,
    Json(input): Json<SongRequestInput>,
) -> Result<Json<SongRequest>, ApiError> {
    let request = add_request_to_queue(&state, input).await?;

    Ok(Json(request))
}

async fn chat_command(
    State(state): State<AppState>,
    Json(input): Json<ChatCommandInput>,
) -> Result<Json<ChatCommandResponse>, ApiError> {
    let settings = config::command_settings(&state.config.paths);
    let command = parse_chat_command(input, &settings);
    let response = match command {
        ChatCommand::SongRequest { input, role } => {
            let request = add_request_to_queue_for_role(&state, input, role).await?;
            let display_title = display::chat_song_title(&request);
            state
                .record_event(
                    "request",
                    format!("{} pediu {}", request.requester, display_title),
                )
                .await;
            ChatCommandResponse::SongRequest { request }
        }
        ChatCommand::CurrentSong => {
            let queue = effective_queue_view(&state).await;
            ChatCommandResponse::CurrentSong {
                current_song: queue.current_song,
            }
        }
        ChatCommand::Queue => {
            let queue = effective_queue_view(&state).await;
            ChatCommandResponse::Queue { queue }
        }
        ChatCommand::RemoveLast { requester } => ChatCommandResponse::Playback {
            message: remove_last_request_message(&state, requester).await,
        },
        ChatCommand::Skip { requester } => ChatCommandResponse::Playback {
            message: skip_message(&state, requester).await,
        },
        ChatCommand::Playback { requester, action } => ChatCommandResponse::Playback {
            message: playback_message(&state, requester, action).await,
        },
        ChatCommand::Volume { requester, level } => ChatCommandResponse::Volume {
            message: volume_message(&state, requester, level).await,
        },
        ChatCommand::Help => ChatCommandResponse::Help {
            commands: help_commands(&settings),
        },
        ChatCommand::AccessDenied {
            requester,
            command,
            required,
        } => ChatCommandResponse::AccessDenied {
            message: {
                let message = access_denied_message(requester, &command, required);
                state.record_event("access", message.clone()).await;
                message
            },
        },
        ChatCommand::Ignored => ChatCommandResponse::Ignored,
    };

    Ok(Json(response))
}

async fn remove_last_request_message(state: &AppState, requester: String) -> String {
    let removed = {
        let mut queue = state.queue.write().await;
        queue.remove_last_by_requester(&requester)
    };
    if let Err(error) = save_current_queue_state(state).await {
        state.record_event("error", error.message).await;
    }

    let message = match removed {
        Some(song) => {
            let suffix = if matches!(song.source, RequestSource::Spotify { .. }) {
                " Se ela ja entrou na fila interna do Spotify, use skip para pular quando chegar."
            } else {
                ""
            };
            let title = display::chat_song_title(&song);
            format!("@{requester} removi seu ultimo pedido: {title}.{suffix}")
        }
        None => format!("@{requester} nao encontrei pedido seu pendente para remover."),
    };
    state.record_event("request", message.clone()).await;
    message
}

async fn skip_message(state: &AppState, requester: String) -> String {
    if is_youtube_browser_mode(state) {
        let message = local_browser_skip_message(state, &requester).await;
        state.record_event("player", message.clone()).await;
        return message;
    }

    if matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(current_youtube_playback(state), YoutubePlayback::Pear)
    {
        let message = pear_skip_message(state, &requester).await;
        state.record_event("player", message.clone()).await;
        return message;
    }

    if let Some(message) =
        spotify_playback_message(state, requester.clone(), PlaybackAction::Next).await
    {
        state.record_event("player", message.clone()).await;
        return message;
    }

    let current_song = state.queue.write().await.skip();
    if let Err(error) = save_current_queue_state(state).await {
        state.record_event("error", error.message).await;
    }
    let message = match current_song {
        Some(song) => format!(
            "@{requester} skip feito. Agora: {}",
            display::chat_song_title(&song)
        ),
        None => format!("@{requester} skip feito. Fila vazia."),
    };
    state.record_event("player", message.clone()).await;
    message
}

async fn pear_skip_message(state: &AppState, requester: &str) -> String {
    match pear::skip_next(&state.config).await {
        Ok(outcome) if outcome.changed => {
            let suffix = if outcome.fallback_used {
                " usando fila interna"
            } else {
                ""
            };
            match outcome.after {
                Some(after) => format!("@{requester} pulei no Pear{suffix}. Agora: {after}."),
                None => format!("@{requester} pulei no Pear{suffix}."),
            }
        }
        Ok(outcome) => {
            let current = outcome
                .after
                .or(outcome.before)
                .map(|song| format!(" Musica atual: {song}."))
                .unwrap_or_default();
            format!("@{requester} enviei skip ao Pear, mas a musica nao mudou.{current}")
        }
        Err(error) => format!("@{requester} nao consegui pular no Pear: {error}"),
    }
}

fn help_commands(settings: &crate::commands::CommandSettings) -> Vec<String> {
    let aliases = &settings.aliases;
    vec![
        format!("{} nome/link", aliases.song_request[0]),
        aliases.current_song[0].clone(),
        aliases.queue[0].clone(),
        aliases.remove[0].clone(),
        aliases.volume[0].clone(),
        format!("{} 30", aliases.volume[0]),
        aliases.play[0].clone(),
        aliases.pause[0].clone(),
        aliases.skip[0].clone(),
    ]
}

async fn playback_message(state: &AppState, requester: String, action: PlaybackAction) -> String {
    if is_youtube_browser_mode(state) {
        let message = match action {
            PlaybackAction::Next => local_browser_skip_message(state, &requester).await,
            PlaybackAction::Play | PlaybackAction::Pause => {
                local_browser_playback_message(state, &requester, action)
            }
        };
        state.record_event("player", message.clone()).await;
        return message;
    }

    if matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(current_youtube_playback(state), YoutubePlayback::Pear)
    {
        let message = match action {
            PlaybackAction::Play => match pear::play(&state.config).await {
                Ok(()) => format!("@{requester} Pear retomado."),
                Err(error) => format!("@{requester} nao consegui controlar o Pear: {error}"),
            },
            PlaybackAction::Pause => match pear::pause(&state.config).await {
                Ok(()) => format!("@{requester} Pear pausado."),
                Err(error) => format!("@{requester} nao consegui controlar o Pear: {error}"),
            },
            PlaybackAction::Next => pear_skip_message(state, &requester).await,
        };
        state.record_event("player", message.clone()).await;
        return message;
    }

    let message = spotify_playback_message(state, requester, action)
        .await
        .unwrap_or_else(|| "Spotify nao conectado.".to_string());
    state.record_event("player", message.clone()).await;
    message
}

async fn spotify_playback_message(
    state: &AppState,
    requester: String,
    action: PlaybackAction,
) -> Option<String> {
    let mut token_guard = state.spotify_token.write().await;
    let token = token_guard.as_mut()?;

    let result = match action {
        PlaybackAction::Play => spotify::resume_playback(&state.config, token).await,
        PlaybackAction::Pause => spotify::pause_playback(&state.config, token).await,
        PlaybackAction::Next => spotify::skip_next(&state.config, token).await,
    };

    Some(match result {
        Ok(()) => match action {
            PlaybackAction::Play => format!("@{requester} playback retomado."),
            PlaybackAction::Pause => format!("@{requester} playback pausado."),
            PlaybackAction::Next => format!("@{requester} pulei para a proxima."),
        },
        Err(error) => format!("@{requester} nao consegui controlar o Spotify: {error}"),
    })
}

async fn volume_message(state: &AppState, requester: String, level: Option<u8>) -> String {
    match level {
        Some(level) => {
            let level = level.clamp(1, 100);
            let mut changed = Vec::new();
            let mut errors = Vec::new();

            match (current_provider(state), current_youtube_playback(state)) {
                (MusicProvider::Youtube, YoutubePlayback::Pear) => {
                    match pear::set_volume(&state.config, level).await {
                        Ok(level) => {
                            persist_volume_setting(
                                state,
                                MusicProvider::Youtube,
                                YoutubePlayback::Pear,
                                level,
                            )
                            .await;
                            changed.push(format!("Pear/YouTube {level}%"));
                        }
                        Err(error) => errors.push(format!("Pear/YouTube: {error}")),
                    }
                }
                (MusicProvider::Youtube, YoutubePlayback::Browser) => {
                    let level = set_browser_volume(state, level).await;
                    changed.push(format!("OBS Browser {level}%"));
                }
                (MusicProvider::Spotify, _) => match set_spotify_volume(state, level).await {
                    Some(Ok(level)) => {
                        persist_volume_setting(
                            state,
                            MusicProvider::Spotify,
                            YoutubePlayback::Browser,
                            level,
                        )
                        .await;
                        changed.push(format!("Spotify {level}%"));
                    }
                    Some(Err(error)) => errors.push(format!("Spotify: {error}")),
                    None => errors.push("Spotify nao conectado".to_string()),
                },
            }

            if !changed.is_empty() {
                let message = format!("@{requester} volume ajustado: {}.", changed.join(", "));
                state.record_event("volume", message.clone()).await;
                if !errors.is_empty() {
                    state
                        .record_event(
                            "volume",
                            format!(
                                "Volume parcial: {}. Falhas: {}",
                                changed.join(", "),
                                errors.join(" | ")
                            ),
                        )
                        .await;
                }
                message
            } else {
                let message = format!(
                    "@{requester} nao consegui mudar volume: {}",
                    errors.join(" | ")
                );
                state.record_event("error", message.clone()).await;
                message
            }
        }
        None => current_volume_message(state).await,
    }
}

async fn set_spotify_volume(state: &AppState, level: u8) -> Option<anyhow::Result<u8>> {
    let mut token_guard = state.spotify_token.write().await;
    let token = token_guard.as_mut()?;
    Some(spotify::set_volume(&state.config, token, level).await)
}

async fn set_browser_volume(state: &AppState, level: u8) -> u8 {
    let level = level.clamp(1, 100);
    state.youtube_browser_volume.store(level, Ordering::SeqCst);
    persist_volume_setting(
        state,
        MusicProvider::Youtube,
        YoutubePlayback::Browser,
        level,
    )
    .await;
    level
}

async fn persist_volume_setting(
    state: &AppState,
    provider: MusicProvider,
    playback: YoutubePlayback,
    level: u8,
) {
    if let Err(error) =
        config::update_volume_setting(&state.config.paths, provider, playback, level)
    {
        state
            .record_event("error", format!("Nao consegui salvar volume: {error}"))
            .await;
    }
}

async fn current_volume_message(state: &AppState) -> String {
    let response = read_volume(state).await;
    state.record_event("volume", response.message.clone()).await;
    response.message
}

async fn read_volume(state: &AppState) -> VolumeResponse {
    if matches!(
        (current_provider(state), current_youtube_playback(state)),
        (MusicProvider::Youtube, YoutubePlayback::Pear)
    ) {
        return match pear::current_volume(&state.config).await {
            Ok(volume) => {
                let suffix = if volume.is_muted { " (mutado)" } else { "" };
                VolumeResponse {
                    target: "pear",
                    level: Some(volume.state),
                    muted: volume.is_muted,
                    message: format!("Volume atual Pear/YouTube: {}%{suffix}.", volume.state),
                }
            }
            Err(error) => VolumeResponse {
                target: "pear",
                level: None,
                muted: false,
                message: format!("Nao consegui ler o volume Pear/YouTube: {error}"),
            },
        };
    }

    if is_youtube_browser_mode(state) {
        let level = state
            .youtube_browser_volume
            .load(Ordering::SeqCst)
            .clamp(1, 100);
        return VolumeResponse {
            target: "browser",
            level: Some(level),
            muted: false,
            message: format!(
                "Volume atual OBS Browser: {level}%. O fader do OBS continua separado no mixer."
            ),
        };
    }

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return VolumeResponse {
            target: "spotify",
            level: None,
            muted: false,
            message: "Spotify nao conectado.".to_string(),
        };
    };

    match spotify::current_volume(&state.config, token).await {
        Ok(Some(level)) => VolumeResponse {
            target: "spotify",
            level: Some(level),
            muted: false,
            message: format!("Volume atual Spotify: {level}%."),
        },
        Ok(None) => VolumeResponse {
            target: "spotify",
            level: None,
            muted: false,
            message: "Nao encontrei um device Spotify ativo para ler o volume.".to_string(),
        },
        Err(error) => VolumeResponse {
            target: "spotify",
            level: None,
            muted: false,
            message: format!("Nao consegui ler o volume Spotify: {error}"),
        },
    }
}

fn access_denied_message(
    requester: String,
    command: &str,
    required: crate::commands::CommandAccess,
) -> String {
    match required {
        crate::commands::CommandAccess::Follower => {
            format!("@{requester} {command} precisa seguir o canal.")
        }
        crate::commands::CommandAccess::Subscriber => {
            format!("@{requester} {command} precisa de sub, VIP, moderador ou streamer.")
        }
        crate::commands::CommandAccess::Vip => {
            format!("@{requester} {command} precisa de VIP, moderador ou streamer.")
        }
        crate::commands::CommandAccess::Moderator => {
            format!("@{requester} {command} precisa de moderador ou streamer.")
        }
        crate::commands::CommandAccess::Streamer => {
            format!("@{requester} {command} precisa do streamer.")
        }
    }
}

async fn add_request_to_queue(
    state: &AppState,
    input: SongRequestInput,
) -> Result<SongRequest, ApiError> {
    match request_flow::add_request(state, input).await {
        Ok(request) => {
            save_current_queue_state(state).await?;
            if is_youtube_browser_mode(state)
                && matches!(request.source, RequestSource::Youtube { .. })
            {
                arm_youtube_after_current_spotify(state).await;
            }
            Ok(request)
        }
        Err(error) => {
            state.record_event("error", error.to_string()).await;
            Err(ApiError::bad_request(error))
        }
    }
}

async fn add_request_to_queue_for_role(
    state: &AppState,
    input: SongRequestInput,
    role: ChatUserRole,
) -> Result<SongRequest, ApiError> {
    match request_flow::add_request_for_role(state, input, role).await {
        Ok(request) => {
            save_current_queue_state(state).await?;
            if is_youtube_browser_mode(state)
                && matches!(request.source, RequestSource::Youtube { .. })
            {
                arm_youtube_after_current_spotify(state).await;
            }
            Ok(request)
        }
        Err(error) => {
            state.record_event("error", error.to_string()).await;
            Err(ApiError::bad_request(error))
        }
    }
}

async fn effective_queue_view(state: &AppState) -> QueueView {
    let provider = current_provider(state);
    if matches!(provider, MusicProvider::Spotify) {
        if let Some(view) = spotify_queue_view(state).await {
            return merge_spotify_and_app_queue(state, view).await;
        }
    }

    if matches!(provider, MusicProvider::Youtube)
        && matches!(current_youtube_playback(state), YoutubePlayback::Pear)
    {
        return merge_pear_and_app_queue(state).await;
    }

    filtered_app_queue_view(
        state,
        MusicProvider::Youtube,
        Some(YoutubeRequestPlayback::Browser),
    )
    .await
}

fn current_provider(state: &AppState) -> MusicProvider {
    config::UiConfigView::load(&state.config.paths).default_provider
}

fn current_youtube_playback(state: &AppState) -> YoutubePlayback {
    config::UiConfigView::load(&state.config.paths).youtube_playback
}

fn is_youtube_browser_mode(state: &AppState) -> bool {
    matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(current_youtube_playback(state), YoutubePlayback::Browser)
}

async fn local_browser_skip_message(state: &AppState, requester: &str) -> String {
    state.youtube_browser_paused.store(false, Ordering::SeqCst);
    let current_id = filtered_app_queue_view(
        state,
        MusicProvider::Youtube,
        Some(YoutubeRequestPlayback::Browser),
    )
    .await
    .current_song
    .map(|song| song.id);

    let current_song = if let Some(id) = current_id {
        let mut queue = state.queue.write().await;
        queue.remove_by_id(id);
        save_queue_state(state, &queue).ok();
        filtered_app_queue_view(
            state,
            MusicProvider::Youtube,
            Some(YoutubeRequestPlayback::Browser),
        )
        .await
        .current_song
    } else {
        None
    };

    match current_song {
        Some(song) => format!(
            "@{requester} skip feito. Agora: {}",
            display::chat_song_title(&song)
        ),
        None => format!("@{requester} skip feito. Fila vazia."),
    }
}

fn local_browser_playback_message(
    state: &AppState,
    requester: &str,
    action: PlaybackAction,
) -> String {
    match action {
        PlaybackAction::Play => {
            state.youtube_browser_paused.store(false, Ordering::SeqCst);
            format!("@{requester} player OBS retomado.")
        }
        PlaybackAction::Pause => {
            state.youtube_browser_paused.store(true, Ordering::SeqCst);
            format!("@{requester} player OBS pausado.")
        }
        PlaybackAction::Next => {
            state.youtube_browser_paused.store(false, Ordering::SeqCst);
            format!("@{requester} pulando player OBS.")
        }
    }
}

async fn merge_pear_and_app_queue(state: &AppState) -> QueueView {
    let app_view = filtered_app_queue_view(
        state,
        MusicProvider::Youtube,
        Some(YoutubeRequestPlayback::Pear),
    )
    .await;
    let Some(pear_current) = pear::now_playing_request(&state.config)
        .await
        .ok()
        .flatten()
    else {
        return app_view;
    };

    let Some(app_current) = app_view.current_song.clone() else {
        return QueueView {
            current_song: Some(pear_current),
            ..app_view
        };
    };

    if same_youtube_video(&app_current, &pear_current) {
        return QueueView {
            current_song: Some(SongRequest {
                title: pear_current.title,
                artist: pear_current.artist,
                ..app_current
            }),
            ..app_view
        };
    }

    let mut pending = app_pending_requests(app_view.clone());
    pending.retain(|song| !same_youtube_video(song, &pear_current));
    let queue_length = pending.len();

    QueueView {
        current_song: Some(pear_current),
        queue: pending,
        queue_length,
        persistence: app_view.persistence,
    }
}

fn same_youtube_video(left: &SongRequest, right: &SongRequest) -> bool {
    match (&left.source, &right.source) {
        (
            RequestSource::Youtube { video_id: left, .. },
            RequestSource::Youtube {
                video_id: right, ..
            },
        ) => left == right,
        _ => false,
    }
}

async fn filtered_app_queue_view(
    state: &AppState,
    provider: MusicProvider,
    playback: Option<YoutubeRequestPlayback>,
) -> QueueView {
    filter_queue_view(state.queue.read().await.view(), provider, playback)
}

fn filter_queue_view(
    view: QueueView,
    provider: MusicProvider,
    playback: Option<YoutubeRequestPlayback>,
) -> QueueView {
    let mut requests = app_pending_requests(view)
        .into_iter()
        .filter(|song| request_matches_stream(song, provider, playback))
        .collect::<Vec<_>>();
    let current_song = requests.first().cloned();
    if current_song.is_some() {
        requests.remove(0);
    }
    QueueView {
        current_song,
        queue_length: requests.len(),
        queue: requests,
        persistence: None,
    }
}

fn request_matches_stream(
    song: &SongRequest,
    provider: MusicProvider,
    playback: Option<YoutubeRequestPlayback>,
) -> bool {
    match provider {
        MusicProvider::Spotify => matches!(
            song.source,
            RequestSource::Spotify { .. }
                | RequestSource::Search {
                    provider: MusicProvider::Spotify
                }
        ),
        MusicProvider::Youtube => match (&song.source, playback) {
            (
                RequestSource::Youtube {
                    playback: Some(source_playback),
                    ..
                },
                Some(active_playback),
            ) => *source_playback == active_playback,
            _ => false,
        },
    }
}

async fn save_current_queue_state(state: &AppState) -> Result<(), ApiError> {
    let queue = state.queue.read().await;
    save_queue_state(state, &queue)
}

fn save_queue_state(
    state: &AppState,
    queue: &crate::song_requests::SongQueue,
) -> Result<(), ApiError> {
    if !config::queue_persistence_enabled(&state.config.paths) {
        return Ok(());
    }

    queue
        .save(&state.config.paths.queue_file)
        .map_err(ApiError::bad_request)
}

async fn active_persisted_queue_view(state: &AppState) -> QueueView {
    let ui = config::UiConfigView::load(&state.config.paths);
    match ui.default_provider {
        MusicProvider::Spotify => {
            filtered_app_queue_view(state, MusicProvider::Spotify, None).await
        }
        MusicProvider::Youtube => {
            let playback = match ui.youtube_playback {
                YoutubePlayback::Pear => YoutubeRequestPlayback::Pear,
                YoutubePlayback::Browser => YoutubeRequestPlayback::Browser,
            };
            filtered_app_queue_view(state, MusicProvider::Youtube, Some(playback)).await
        }
    }
}

async fn queue_persistence(state: &AppState) -> QueuePersistence {
    let enabled = config::queue_persistence_enabled(&state.config.paths);
    let view = active_persisted_queue_view(state).await;
    QueuePersistence {
        enabled,
        exists: state.config.paths.queue_file.exists(),
        saved_items: if enabled {
            usize::from(view.current_song.is_some()) + view.queue.len()
        } else {
            0
        },
    }
}

async fn merge_spotify_and_app_queue(state: &AppState, mut spotify_view: QueueView) -> QueueView {
    let app_view = filtered_app_queue_view(state, MusicProvider::Spotify, None).await;
    let mut app_requests = app_pending_requests(app_view);

    if let Some(current) = &spotify_view.current_song {
        let matched_requests = app_requests
            .iter()
            .filter(|song| spotify_current_matches_request(&current.title, song))
            .cloned()
            .collect::<Vec<_>>();
        let matched_ids = matched_requests
            .iter()
            .map(|song| song.id)
            .collect::<Vec<_>>();
        if let Some(matched_request) = matched_requests.into_iter().next() {
            spotify_view.current_song = Some(SongRequest {
                title: current.title.clone(),
                artist: matched_request.artist,
                ..matched_request
            });
            {
                let mut queue = state.queue.write().await;
                for id in &matched_ids {
                    queue.remove_by_id(*id);
                }
                if let Err(error) = save_queue_state(state, &queue) {
                    tracing::warn!(
                        error = %error.message,
                        "failed to save queue after Spotify request started"
                    );
                }
            }
            app_requests.retain(|song| !matched_ids.contains(&song.id));
        }
    }

    if spotify_view.current_song.is_none() && !app_requests.is_empty() {
        spotify_view.current_song = Some(app_requests.remove(0));
    }

    spotify_view.queue = app_requests;
    spotify_view.queue_length = spotify_view.queue.len();
    spotify_view
}

fn app_pending_requests(view: QueueView) -> Vec<SongRequest> {
    let mut requests = Vec::new();

    if let Some(song) = view.current_song {
        if matches!(
            song.source,
            RequestSource::Youtube { .. } | RequestSource::Spotify { .. }
        ) {
            requests.push(song);
        }
    }

    requests.extend(view.queue.into_iter().filter(|song| {
        matches!(
            song.source,
            RequestSource::Youtube { .. } | RequestSource::Spotify { .. }
        )
    }));

    requests
}

fn normalize_song_title(title: &str) -> String {
    title
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| ch.is_alphanumeric())
        .collect()
}

fn spotify_current_matches_request(current_title: &str, request: &SongRequest) -> bool {
    let current = normalize_song_title(current_title);
    let title = normalize_song_title(&request.title);
    if title.is_empty() || !current.contains(&title) {
        return false;
    }

    let artist = normalize_song_title(&request.artist);
    artist.is_empty() || request.artist.eq_ignore_ascii_case("spotify") || current.contains(&artist)
}

async fn spotify_queue_view(state: &AppState) -> Option<QueueView> {
    let mut token_guard = state.spotify_token.write().await;
    let token = token_guard.as_mut()?;
    let snapshot = spotify::queue_snapshot(&state.config, token).await.ok()?;

    let current_song = snapshot
        .currently_playing
        .map(|title| spotify_song_request(0, title));
    let queue = snapshot
        .upcoming
        .into_iter()
        .enumerate()
        .map(|(index, title)| spotify_song_request(index as u64 + 1, title))
        .collect::<Vec<_>>();

    Some(QueueView {
        current_song,
        queue_length: queue.len(),
        queue,
        persistence: None,
    })
}

fn spotify_song_request(id: u64, title: String) -> SongRequest {
    SongRequest {
        id,
        requester: "spotify".to_string(),
        query: title.clone(),
        source: RequestSource::Search {
            provider: MusicProvider::Spotify,
        },
        title,
        artist: "Spotify".to_string(),
    }
}

async fn skip(State(state): State<AppState>) -> Json<SkipResponse> {
    let current_song = state.queue.write().await.skip();
    if let Err(error) = save_current_queue_state(&state).await {
        state.record_event("error", error.message).await;
    }
    clear_youtube_state_if_no_pending(&state).await;

    Json(SkipResponse { current_song })
}

async fn volume_status(State(state): State<AppState>) -> Json<VolumeResponse> {
    Json(read_volume(&state).await)
}

async fn set_volume(
    State(state): State<AppState>,
    Json(input): Json<VolumeInput>,
) -> Result<Json<VolumeResponse>, ApiError> {
    let current = read_volume(&state).await;
    let delta = input.delta.unwrap_or(0);
    let target = input
        .level
        .unwrap_or_else(|| current.level.unwrap_or(50).saturating_add_signed(delta))
        .clamp(1, 100);

    let message = volume_message(&state, "dashboard".to_string(), Some(target)).await;
    let mut response = read_volume(&state).await;
    response.message = message;
    Ok(Json(response))
}

async fn youtube_player_response(state: &AppState) -> YoutubePlayerResponse {
    if matches!(current_youtube_playback(state), YoutubePlayback::Pear) {
        return YoutubePlayerResponse {
            current_song: None,
            waiting_for_spotify: None,
            paused: false,
            volume: 100,
        };
    }

    let current_song = current_youtube_song(state).await;
    if current_song.is_none() {
        *state.youtube_waiting_spotify_title.lock().await = None;
        return YoutubePlayerResponse {
            current_song: None,
            waiting_for_spotify: None,
            paused: state.youtube_browser_paused.load(Ordering::SeqCst),
            volume: state
                .youtube_browser_volume
                .load(Ordering::SeqCst)
                .clamp(1, 100),
        };
    }

    let waiting_for_spotify = spotify_blocks_youtube(state).await;
    YoutubePlayerResponse {
        current_song: if waiting_for_spotify.is_some() {
            None
        } else {
            current_song
        },
        waiting_for_spotify,
        paused: state.youtube_browser_paused.load(Ordering::SeqCst),
        volume: state
            .youtube_browser_volume
            .load(Ordering::SeqCst)
            .clamp(1, 100),
    }
}

async fn current_youtube_song(state: &AppState) -> Option<YoutubePlayerSong> {
    filtered_app_queue_view(
        state,
        MusicProvider::Youtube,
        Some(YoutubeRequestPlayback::Browser),
    )
    .await
    .current_song
    .and_then(|song| YoutubePlayerSong::from_request(&song))
}

async fn has_pending_youtube_request(state: &AppState) -> bool {
    filtered_app_queue_view(
        state,
        MusicProvider::Youtube,
        Some(YoutubeRequestPlayback::Browser),
    )
    .await
    .current_song
    .is_some()
}

async fn clear_youtube_state_if_no_pending(state: &AppState) {
    if has_pending_youtube_request(state).await {
        return;
    }

    *state.youtube_waiting_spotify_title.lock().await = None;
    *state.youtube_active_pear_video_id.lock().await = None;
    *state.youtube_failed_pear_video_id.lock().await = None;
    resume_spotify_after_youtube(state).await;
}

async fn pause_spotify_for_youtube(state: &AppState) {
    if state.youtube_player_paused_spotify.load(Ordering::SeqCst) {
        return;
    }

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return;
    };

    match spotify::pause_playback(&state.config, token).await {
        Ok(()) => {
            state
                .youtube_player_paused_spotify
                .store(true, Ordering::SeqCst);
            state
                .record_event("player", "Spotify pausado para tocar YouTube")
                .await;
        }
        Err(error) => {
            if error.to_string().contains("Restriction violated") {
                state
                    .youtube_player_paused_spotify
                    .store(true, Ordering::SeqCst);
                state
                    .record_event(
                        "player",
                        "Spotify recusou pausa por restricao; liberando pedido YouTube",
                    )
                    .await;
                return;
            }

            state
                .record_event("error", format!("Nao consegui pausar Spotify: {error}"))
                .await;
        }
    }
}

async fn arm_youtube_after_current_spotify(state: &AppState) {
    if !is_youtube_browser_mode(state) || state.youtube_player_paused_spotify.load(Ordering::SeqCst)
    {
        return;
    }

    {
        let waiting_title = state.youtube_waiting_spotify_title.lock().await;
        if waiting_title.is_some() {
            return;
        }
    }

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return;
    };

    match spotify::current_playback(&state.config, token).await {
        Ok(Some(playback)) if playback.is_playing => {
            let title = playback.title;
            *state.youtube_waiting_spotify_title.lock().await = Some(title.clone());
            state
                .record_event(
                    "player",
                    format!("YouTube aguardando fim do Spotify atual: {title}"),
                )
                .await;
        }
        Ok(_) => {
            *state.youtube_waiting_spotify_title.lock().await = None;
        }
        Err(error) => {
            state
                .record_event(
                    "error",
                    format!("Nao consegui marcar espera do Spotify: {error}"),
                )
                .await;
        }
    }
}

async fn spotify_blocks_youtube(state: &AppState) -> Option<String> {
    if state.youtube_player_paused_spotify.load(Ordering::SeqCst) {
        return None;
    }

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        *state.youtube_waiting_spotify_title.lock().await = None;
        return None;
    };

    let playback = match spotify::current_playback(&state.config, token).await {
        Ok(playback) => playback,
        Err(error) => {
            state
                .record_event("error", format!("Nao consegui ler Spotify atual: {error}"))
                .await;
            return None;
        }
    };

    let Some(playback) = playback else {
        *state.youtube_waiting_spotify_title.lock().await = None;
        return None;
    };
    if !playback.is_playing {
        *state.youtube_waiting_spotify_title.lock().await = None;
        return None;
    }

    let mut waiting_title = state.youtube_waiting_spotify_title.lock().await;
    match waiting_title.as_ref() {
        Some(title) if title == &playback.title => Some(playback.title),
        Some(_) => {
            *waiting_title = None;
            None
        }
        None => None,
    }
}

async fn resume_spotify_after_youtube(state: &AppState) {
    if !state
        .youtube_player_paused_spotify
        .swap(false, Ordering::SeqCst)
    {
        return;
    }

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return;
    };

    match spotify::resume_playback(&state.config, token).await {
        Ok(()) => {
            state
                .record_event("player", "Spotify retomado apos fila YouTube")
                .await;
        }
        Err(error) => {
            state
                .record_event("error", format!("Nao consegui retomar Spotify: {error}"))
                .await;
        }
    }
}

async fn resolve_youtube_audio_url(video_id: &str) -> anyhow::Result<String> {
    let output = timeout(
        Duration::from_secs(20),
        Command::new("yt-dlp")
            .args([
                "--no-playlist",
                "--no-warnings",
                "-f",
                "bestaudio[ext=m4a]/bestaudio/best",
                "-g",
                &format!("https://www.youtube.com/watch?v={video_id}"),
            ])
            .output(),
    )
    .await
    .map_err(|_| anyhow!("yt-dlp demorou demais para resolver o audio"))?
    .map_err(|error| anyhow!("yt-dlp nao executou: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "yt-dlp falhou ao resolver audio: {}",
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("http://") || line.starts_with("https://"))
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("yt-dlp nao retornou uma URL de audio"))
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    let parse = |version: &str| -> Vec<u64> {
        version
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let latest_parts = parse(latest);
    let current_parts = parse(current);
    let len = latest_parts.len().max(current_parts.len());

    for index in 0..len {
        let latest_part = *latest_parts.get(index).unwrap_or(&0);
        let current_part = *current_parts.get(index).unwrap_or(&0);
        if latest_part != current_part {
            return latest_part > current_part;
        }
    }

    false
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
}

#[derive(Debug, Deserialize)]
struct GithubReleaseResponse {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

#[derive(Debug, Serialize)]
struct UpdateInstalledResponse {
    current_version: String,
    current_tag: String,
    release_url: String,
    changelog: String,
}

#[derive(Debug, Serialize)]
struct UpdateLatestResponse {
    current_version: String,
    latest_version: String,
    latest_tag: String,
    release_url: String,
    update_available: bool,
    message: String,
    changelog: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ChatCommandResponse {
    SongRequest { request: SongRequest },
    CurrentSong { current_song: Option<SongRequest> },
    Queue { queue: QueueView },
    Volume { message: String },
    Playback { message: String },
    Help { commands: Vec<String> },
    AccessDenied { message: String },
    Ignored,
}

#[derive(Debug, Serialize)]
struct SkipResponse {
    current_song: Option<SongRequest>,
}

#[derive(Debug, Deserialize)]
struct VolumeInput {
    level: Option<u8>,
    delta: Option<i8>,
}

#[derive(Debug, Serialize)]
struct VolumeResponse {
    target: &'static str,
    level: Option<u8>,
    muted: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct YoutubePlayerResponse {
    current_song: Option<YoutubePlayerSong>,
    waiting_for_spotify: Option<String>,
    paused: bool,
    volume: u8,
}

#[derive(Debug, Deserialize)]
struct YoutubePlayerSyncInput {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct YoutubePlayerEventInput {
    message: String,
}

#[derive(Debug, Serialize)]
struct YoutubePlayerEventResponse {
    ok: bool,
}

#[derive(Debug, Serialize)]
struct YoutubeAudioResponse {
    audio_url: String,
}

#[derive(Debug, Serialize)]
struct YoutubePlayerSong {
    id: u64,
    video_id: String,
    title: String,
    artist: String,
    requester: String,
}

impl YoutubePlayerSong {
    fn from_request(song: &SongRequest) -> Option<Self> {
        let RequestSource::Youtube { video_id, .. } = &song.source else {
            return None;
        };

        Some(Self {
            id: song.id,
            video_id: video_id.clone(),
            title: song.title.clone(),
            artist: song.artist.clone(),
            requester: song.requester.clone(),
        })
    }
}

#[derive(Debug, Serialize)]
struct ShutdownResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ClearEventsResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct UpdateResponse {
    status: &'static str,
    message: &'static str,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct UpdateStatusResponse {
    status: String,
    message: String,
    before: String,
    after: String,
    timestamp: String,
    log_tail: String,
    current_version: String,
}

#[derive(Debug, Serialize)]
struct ConnectionsStatus {
    spotify: spotify::SpotifyConnectionStatus,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(error: anyhow::Error) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: error.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::*;
    use crate::config::AppConfig;

    fn isolated_config(
        name: &str,
        provider: MusicProvider,
        playback: YoutubePlayback,
    ) -> AppConfig {
        let mut config = AppConfig::from_env().expect("config");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("song-request-linux-http-{name}-{unique}"));
        config.paths.config_dir = root.join("config");
        config.paths.state_dir = root.join("state");
        config.paths.queue_file = config.paths.state_dir.join("queue.json");
        config.default_provider = provider;
        config.youtube.playback = playback;
        crate::config::save_ui_config(
            &config.paths,
            crate::config::UiConfigInput {
                default_provider: Some(provider),
                youtube_playback: Some(playback),
                pear_base_url: None,
                spotify_client_id: None,
                spotify_fallback_enabled: Some(false),
                queue_persistence_enabled: Some(false),
                twitch_client_id: None,
                twitch_bot_username: None,
                twitch_channel: None,
                twitch_bot_oauth_token: None,
                youtube_api_key: None,
                youtube_max_duration_seconds: Some(360),
                youtube_allow_non_music: Some(false),
                command_settings: None,
                queue_limits: Some(crate::config::QueueLimitConfig::default()),
                overlay: None,
            },
        )
        .expect("save config");
        config
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let config = AppConfig::from_env().expect("config");
        let app = router(AppState::new(config));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn player_page_returns_ok() {
        let config = AppConfig::from_env().expect("config");
        let app = router(AppState::new(config));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/player")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn song_request_endpoint_adds_current_song() {
        let config = isolated_config(
            "song-request-endpoint-adds-current-song",
            MusicProvider::Youtube,
            YoutubePlayback::Browser,
        );
        let app = router(AppState::new(config));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/song-requests")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"requester":"bruno","query":"https://youtu.be/dQw4w9WgXcQ"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn youtube_player_returns_current_youtube_song() {
        let config = isolated_config(
            "youtube-player-current",
            MusicProvider::Youtube,
            YoutubePlayback::Browser,
        );
        let state = AppState::new(config);
        state.queue.write().await.clear();
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            source: RequestSource::Youtube {
                video_id: "dQw4w9WgXcQ".to_string(),
                playback: Some(YoutubeRequestPlayback::Browser),
            },
            title: "Never Gonna Give You Up".to_string(),
            artist: "Rick Astley".to_string(),
        });

        let response = youtube_player_response(&state).await;

        assert_eq!(
            response.current_song.map(|song| song.video_id),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(response.waiting_for_spotify, None);
    }

    #[tokio::test]
    async fn youtube_player_uses_browser_stream_only() {
        let config = isolated_config(
            "youtube-player-stream",
            MusicProvider::Youtube,
            YoutubePlayback::Browser,
        );
        let state = AppState::new(config);
        state.queue.write().await.clear();
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "system of a down spiders".to_string(),
            source: RequestSource::Spotify {
                uri: "spotify:track:3njyXewwWLp3B0p6rSMeyw".to_string(),
            },
            title: "Spiders".to_string(),
            artist: "System Of A Down".to_string(),
        });
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            source: RequestSource::Youtube {
                video_id: "dQw4w9WgXcQ".to_string(),
                playback: Some(YoutubeRequestPlayback::Browser),
            },
            title: "Never Gonna Give You Up".to_string(),
            artist: "Rick Astley".to_string(),
        });

        let response = youtube_player_response(&state).await;

        assert_eq!(
            response.current_song.map(|song| song.video_id),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(response.waiting_for_spotify, None);
    }

    #[tokio::test]
    async fn spotify_queue_hides_youtube_requests() {
        let config = isolated_config(
            "spotify-queue-filter",
            MusicProvider::Spotify,
            YoutubePlayback::Browser,
        );
        let state = AppState::new(config);
        state.queue.write().await.clear();
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            source: RequestSource::Youtube {
                video_id: "dQw4w9WgXcQ".to_string(),
                playback: Some(YoutubeRequestPlayback::Browser),
            },
            title: "Never Gonna Give You Up".to_string(),
            artist: "Rick Astley".to_string(),
        });
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "system of a down spiders".to_string(),
            source: RequestSource::Spotify {
                uri: "spotify:track:3njyXewwWLp3B0p6rSMeyw".to_string(),
            },
            title: "Spiders".to_string(),
            artist: "System Of A Down".to_string(),
        });
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/9bZkp7q19f0".to_string(),
            source: RequestSource::Youtube {
                video_id: "9bZkp7q19f0".to_string(),
                playback: Some(YoutubeRequestPlayback::Pear),
            },
            title: "Gangnam Style".to_string(),
            artist: "PSY".to_string(),
        });

        let spotify_view = QueueView {
            current_song: Some(spotify_song_request(
                0,
                "One More Time - Daft Punk".to_string(),
            )),
            queue: vec![spotify_song_request(
                1,
                "The Emptiness Machine - Linkin Park".to_string(),
            )],
            queue_length: 1,
            persistence: None,
        };

        let merged = merge_spotify_and_app_queue(&state, spotify_view).await;

        assert_eq!(
            merged.queue.first().map(|song| song.title.as_str()),
            Some("Spiders")
        );
        assert_eq!(merged.queue.get(1).map(|song| song.title.as_str()), None);
        assert_eq!(merged.queue_length, 1);
    }

    #[tokio::test]
    async fn youtube_queue_filters_pear_and_browser_streams() {
        let config = isolated_config(
            "youtube-stream-filter",
            MusicProvider::Youtube,
            YoutubePlayback::Browser,
        );
        let state = AppState::new(config);
        state.queue.write().await.clear();
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            source: RequestSource::Youtube {
                video_id: "dQw4w9WgXcQ".to_string(),
                playback: Some(YoutubeRequestPlayback::Browser),
            },
            title: "Browser Song".to_string(),
            artist: "YouTube".to_string(),
        });
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/9bZkp7q19f0".to_string(),
            source: RequestSource::Youtube {
                video_id: "9bZkp7q19f0".to_string(),
                playback: Some(YoutubeRequestPlayback::Pear),
            },
            title: "Pear Song".to_string(),
            artist: "YouTube".to_string(),
        });

        let browser = effective_queue_view(&state).await;
        assert_eq!(
            browser.current_song.map(|song| song.title),
            Some("Browser Song".to_string())
        );
        assert_eq!(browser.queue_length, 0);

        crate::config::save_ui_config(
            &state.config.paths,
            crate::config::UiConfigInput {
                default_provider: Some(MusicProvider::Youtube),
                youtube_playback: Some(YoutubePlayback::Pear),
                pear_base_url: None,
                spotify_client_id: None,
                spotify_fallback_enabled: Some(false),
                queue_persistence_enabled: Some(false),
                twitch_client_id: None,
                twitch_bot_username: None,
                twitch_channel: None,
                twitch_bot_oauth_token: None,
                youtube_api_key: None,
                youtube_max_duration_seconds: Some(360),
                youtube_allow_non_music: Some(false),
                command_settings: None,
                queue_limits: Some(crate::config::QueueLimitConfig::default()),
                overlay: None,
            },
        )
        .expect("save config");

        let pear = effective_queue_view(&state).await;
        assert_eq!(
            pear.current_song.map(|song| song.title),
            Some("Pear Song".to_string())
        );
        assert_eq!(pear.queue_length, 0);
    }

    #[tokio::test]
    async fn merged_queue_removes_spotify_request_when_it_starts_playing() {
        let mut config = AppConfig::from_env().expect("config");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        config.paths.queue_file =
            std::env::temp_dir().join(format!("song-request-linux-test-{unique}.json"));
        let state = AppState::new(config);
        state.queue.write().await.clear();
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "system of a down spiders".to_string(),
            source: RequestSource::Spotify {
                uri: "spotify:track:3njyXewwWLp3B0p6rSMeyw".to_string(),
            },
            title: "Spiders".to_string(),
            artist: "System Of A Down".to_string(),
        });
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "scatman".to_string(),
            source: RequestSource::Spotify {
                uri: "spotify:track:0eCcFGeTHv3LquL6pvBFFZ".to_string(),
            },
            title: "Scatman".to_string(),
            artist: "Gummy Bear".to_string(),
        });

        let spotify_view = QueueView {
            current_song: Some(spotify_song_request(
                0,
                "Spiders - System Of A Down".to_string(),
            )),
            queue: Vec::new(),
            queue_length: 0,
            persistence: None,
        };

        let merged = merge_spotify_and_app_queue(&state, spotify_view).await;

        assert_eq!(
            merged.current_song.map(|song| song.title),
            Some("Spiders - System Of A Down".to_string())
        );
        assert_eq!(
            merged.queue.first().map(|song| song.title.as_str()),
            Some("Scatman")
        );
        assert_eq!(merged.queue_length, 1);
        assert_eq!(
            state
                .queue
                .read()
                .await
                .view()
                .current_song
                .map(|song| song.title),
            Some("Scatman".to_string())
        );

        let _ = std::fs::remove_file(&state.config.paths.queue_file);
    }
}
