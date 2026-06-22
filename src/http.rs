use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::atomic::Ordering};
use tower_http::trace::TraceLayer;

use crate::{
    commands::{parse_chat_command, ChatCommand, ChatCommandInput, PlaybackAction},
    config, connections, dashboard,
    diagnostics::DiagnosticsResponse,
    overlay, player, request_flow,
    song_requests::{MusicProvider, QueueView, RequestSource, SongRequest, SongRequestInput},
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
        .route("/api/shutdown", post(shutdown))
        .route("/api/status", get(status))
        .route("/api/events", get(events))
        .route("/api/diagnostics", get(diagnostics))
        .route("/api/config", get(get_config).post(save_config))
        .route("/api/connections/status", get(connections_status))
        .route("/api/connections/spotify/start", post(spotify_start))
        .route("/api/connections/twitch/start", post(twitch_start))
        .route("/api/connections/twitch/token", post(twitch_token))
        .route("/api/spotify/devices", get(spotify_devices))
        .route("/api/spotify/playlists", get(spotify_playlists))
        .route(
            "/api/spotify/fallback-playlist",
            post(spotify_fallback_playlist),
        )
        .route("/api/queue", get(queue))
        .route("/api/song-requests", post(add_song_request))
        .route("/api/chat-command", post(chat_command))
        .route("/api/skip", post(skip))
        .route("/overlay", get(overlay::page))
        .route("/player", get(player::page))
        .route("/api/player/youtube", get(youtube_player_current))
        .route("/api/player/youtube/start", post(youtube_player_start))
        .route("/api/player/youtube/finish", post(youtube_player_finish))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
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

    Json(StatusResponse::from_queue(&state, queue))
}

async fn diagnostics(State(state): State<AppState>) -> Json<DiagnosticsResponse> {
    Json(DiagnosticsResponse::collect(&state.config))
}

async fn events(State(state): State<AppState>) -> Json<Vec<crate::state::AppEvent>> {
    Json(state.events.read().await.recent(80))
}

async fn get_config(State(state): State<AppState>) -> Json<config::UiConfigView> {
    Json(config::UiConfigView::load(&state.config.paths))
}

async fn save_config(
    State(state): State<AppState>,
    Json(input): Json<config::UiConfigInput>,
) -> Result<Json<config::UiConfigView>, ApiError> {
    let view = config::save_ui_config(&state.config.paths, input).map_err(ApiError::bad_request)?;

    Ok(Json(view))
}

async fn connections_status(State(state): State<AppState>) -> Json<ConnectionsStatus> {
    let spotify_token = state.spotify_token.read().await;

    Json(ConnectionsStatus {
        spotify: spotify::connection_status(&state.config, spotify_token.as_ref()),
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
  <title>Twitch Bot conectado</title>
  <style>
    body { font-family: system-ui; background:#101114; color:#f4f6f8; }
    a { color:#62a8ff; }
  </style>
</head>
<body>
  <h1 id="title">Conectando Twitch Bot...</h1>
  <p id="message">Aguarde.</p>
  <p><a href="/connections">Voltar</a></p>
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
        message.textContent = `Bot conectado como ${data.twitch_bot_username}. Voce ja pode testar no chat.`;
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
        r#"<!doctype html><html lang="pt-BR"><meta charset="utf-8"><title>Spotify conectado</title><body style="font-family: system-ui; background:#101114; color:#f4f6f8;"><h1>Spotify conectado</h1><p>Voce pode fechar esta aba e voltar ao dashboard.</p><p><a style="color:#62a8ff" href="/connections">Voltar</a></p></body></html>"#,
    ))
}

async fn queue(State(state): State<AppState>) -> Json<QueueView> {
    Json(effective_queue_view(&state).await)
}

async fn youtube_player_current(State(state): State<AppState>) -> Json<YoutubePlayerResponse> {
    Json(youtube_player_response(&state).await)
}

async fn youtube_player_start(
    State(state): State<AppState>,
    Json(input): Json<YoutubePlayerSyncInput>,
) -> Result<Json<YoutubePlayerResponse>, ApiError> {
    let current_song = current_youtube_song(&state).await;
    if current_song
        .as_ref()
        .is_some_and(|song| song.id == input.id)
    {
        pause_spotify_for_youtube(&state).await;
    }

    Ok(Json(YoutubePlayerResponse { current_song }))
}

async fn youtube_player_finish(
    State(state): State<AppState>,
    Json(input): Json<YoutubePlayerSyncInput>,
) -> Result<Json<YoutubePlayerResponse>, ApiError> {
    {
        let mut queue = state.queue.write().await;
        let current_matches = queue
            .view()
            .current_song
            .as_ref()
            .is_some_and(|song| song.id == input.id);
        if current_matches {
            queue.skip();
        }
    }

    let current_song = current_youtube_song(&state).await;
    if current_song.is_none() {
        resume_spotify_after_youtube(&state).await;
    }

    Ok(Json(YoutubePlayerResponse { current_song }))
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
    let command = parse_chat_command(input);
    let response = match command {
        ChatCommand::SongRequest(input) => {
            let request = add_request_to_queue(&state, input).await?;
            state
                .record_event(
                    "request",
                    format!(
                        "{} pediu {} - {}",
                        request.requester, request.title, request.artist
                    ),
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
            commands: vec![
                "!sr nome/link".to_string(),
                "!song".to_string(),
                "!fila".to_string(),
                "!vol".to_string(),
                "!vol 30".to_string(),
                "!play".to_string(),
                "!pause".to_string(),
                "!skip".to_string(),
            ],
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

async fn skip_message(state: &AppState, requester: String) -> String {
    if let Some(message) =
        spotify_playback_message(state, requester.clone(), PlaybackAction::Next).await
    {
        state.record_event("player", message.clone()).await;
        return message;
    }

    let current_song = state.queue.write().await.skip();
    let message = match current_song {
        Some(song) => format!("@{requester} skip feito. Agora: {}", song.title),
        None => format!("@{requester} skip feito. Fila vazia."),
    };
    state.record_event("player", message.clone()).await;
    message
}

async fn playback_message(state: &AppState, requester: String, action: PlaybackAction) -> String {
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
    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return "Spotify nao conectado.".to_string();
    };

    match level {
        Some(level) => match spotify::set_volume(&state.config, token, level).await {
            Ok(level) => {
                let message = format!("@{requester} volume ajustado para {level}%.");
                state.record_event("volume", message.clone()).await;
                message
            }
            Err(error) => {
                let message = format!("@{requester} nao consegui mudar volume: {error}");
                state.record_event("error", message.clone()).await;
                message
            }
        },
        None => match spotify::current_volume(&state.config, token).await {
            Ok(Some(level)) => {
                let message = format!("Volume atual: {level}%.");
                state.record_event("volume", message.clone()).await;
                message
            }
            Ok(None) => {
                let message =
                    "Nao encontrei um device Spotify ativo para ler o volume.".to_string();
                state.record_event("volume", message.clone()).await;
                message
            }
            Err(error) => {
                let message = format!("Nao consegui ler o volume: {error}");
                state.record_event("error", message.clone()).await;
                message
            }
        },
    }
}

fn access_denied_message(
    requester: String,
    command: &str,
    required: crate::commands::CommandAccess,
) -> String {
    match required {
        crate::commands::CommandAccess::Moderator => {
            format!("@{requester} {command} precisa de moderador/broadcaster.")
        }
    }
}

async fn add_request_to_queue(
    state: &AppState,
    input: SongRequestInput,
) -> Result<SongRequest, ApiError> {
    match request_flow::add_request(state, input).await {
        Ok(request) => Ok(request),
        Err(error) => {
            state.record_event("error", error.to_string()).await;
            Err(ApiError::bad_request(error))
        }
    }
}

async fn effective_queue_view(state: &AppState) -> QueueView {
    if let Some(view) = spotify_queue_view(state).await {
        return merge_spotify_and_app_queue(state, view).await;
    }

    state.queue.read().await.view()
}

async fn merge_spotify_and_app_queue(state: &AppState, mut spotify_view: QueueView) -> QueueView {
    let app_view = state.queue.read().await.view();
    let mut app_youtube = youtube_requests(app_view);

    if spotify_view.current_song.is_none() && !app_youtube.is_empty() {
        spotify_view.current_song = Some(app_youtube.remove(0));
    }

    spotify_view.queue.extend(app_youtube);
    spotify_view.queue_length = spotify_view.queue.len();
    spotify_view
}

fn youtube_requests(view: QueueView) -> Vec<SongRequest> {
    let mut requests = Vec::new();

    if let Some(song) = view.current_song {
        if matches!(song.source, RequestSource::Youtube { .. }) {
            requests.push(song);
        }
    }

    requests.extend(
        view.queue
            .into_iter()
            .filter(|song| matches!(song.source, RequestSource::Youtube { .. })),
    );

    requests
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

    Json(SkipResponse { current_song })
}

async fn youtube_player_response(state: &AppState) -> YoutubePlayerResponse {
    YoutubePlayerResponse {
        current_song: current_youtube_song(state).await,
    }
}

async fn current_youtube_song(state: &AppState) -> Option<YoutubePlayerSong> {
    state
        .queue
        .read()
        .await
        .view()
        .current_song
        .and_then(|song| YoutubePlayerSong::from_request(&song))
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
            state
                .record_event("error", format!("Nao consegui pausar Spotify: {error}"))
                .await;
        }
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

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
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

#[derive(Debug, Serialize)]
struct YoutubePlayerResponse {
    current_song: Option<YoutubePlayerSong>,
}

#[derive(Debug, Deserialize)]
struct YoutubePlayerSyncInput {
    id: u64,
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
        let RequestSource::Youtube { video_id } = &song.source else {
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::*;
    use crate::config::AppConfig;

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
        let config = AppConfig::from_env().expect("config");
        let app = router(AppState::new(config));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/song-requests")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"requester":"bruno","query":"one more time"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
