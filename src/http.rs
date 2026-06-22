use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    commands::{parse_chat_command, ChatCommand, ChatCommandInput},
    dashboard,
    diagnostics::DiagnosticsResponse,
    overlay,
    song_requests::{QueueView, SongRequest, SongRequestInput},
    state::{AppState, HealthResponse, StatusResponse},
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(dashboard::page))
        .route("/health", get(health))
        .route("/api/status", get(status))
        .route("/api/diagnostics", get(diagnostics))
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

async fn queue(State(state): State<AppState>) -> Json<QueueView> {
    Json(state.queue.read().await.view())
}

async fn add_song_request(
    State(state): State<AppState>,
    Json(input): Json<SongRequestInput>,
) -> Result<Json<SongRequest>, ApiError> {
    let request = state
        .queue
        .write()
        .await
        .add(input)
        .map_err(ApiError::bad_request)?;

    Ok(Json(request))
}

async fn chat_command(
    State(state): State<AppState>,
    Json(input): Json<ChatCommandInput>,
) -> Result<Json<ChatCommandResponse>, ApiError> {
    let command = parse_chat_command(input);
    let response = match command {
        ChatCommand::SongRequest(input) => {
            let request = state
                .queue
                .write()
                .await
                .add(input)
                .map_err(ApiError::bad_request)?;
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
