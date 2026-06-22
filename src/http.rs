use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::collections::HashMap;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    commands::{parse_chat_command, ChatCommand, ChatCommandInput},
    config, connections, dashboard,
    diagnostics::DiagnosticsResponse,
    overlay, request_flow,
    song_requests::{QueueView, SongRequest, SongRequestInput},
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
        .route("/api/status", get(status))
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
        .fallback(not_found)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let queue = state.queue.read().await.view();

    Json(StatusResponse::from_queue(&state, queue))
}

async fn diagnostics(State(state): State<AppState>) -> Json<DiagnosticsResponse> {
    Json(DiagnosticsResponse::collect(&state.config))
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
        message.textContent = `Bot conectado como ${data.twitch_bot_username}. Reinicie o app para conectar ao chat.`;
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
    Json(state.queue.read().await.view())
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
            ChatCommandResponse::SongRequest { request }
        }
        ChatCommand::CurrentSong => {
            let queue = state.queue.read().await.view();
            ChatCommandResponse::CurrentSong {
                current_song: queue.current_song,
            }
        }
        ChatCommand::Skip { requester } => {
            let current_song = state.queue.write().await.skip();
            ChatCommandResponse::Skipped {
                requester,
                current_song,
            }
        }
        ChatCommand::Ignored => ChatCommandResponse::Ignored,
    };

    Ok(Json(response))
}

async fn add_request_to_queue(
    state: &AppState,
    input: SongRequestInput,
) -> Result<SongRequest, ApiError> {
    request_flow::add_request(&state, input)
        .await
        .map_err(ApiError::bad_request)
}

async fn skip(State(state): State<AppState>) -> Json<SkipResponse> {
    let current_song = state.queue.write().await.skip();

    Json(SkipResponse { current_song })
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ChatCommandResponse {
    SongRequest {
        request: SongRequest,
    },
    CurrentSong {
        current_song: Option<SongRequest>,
    },
    Skipped {
        requester: String,
        current_song: Option<SongRequest>,
    },
    Ignored,
}

#[derive(Debug, Serialize)]
struct SkipResponse {
    current_song: Option<SongRequest>,
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
                        r#"{"requester":"bruno","query":"https://youtu.be/dQw4w9WgXcQ"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
