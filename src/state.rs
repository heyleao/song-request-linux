use serde::Serialize;

use crate::config::{AppConfig, APP_NAME};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub app_name: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub app: &'static str,
    pub version: &'static str,
    pub provider: &'static str,
    pub current_song: Option<SongView>,
    pub queue_length: usize,
}

#[derive(Serialize)]
pub struct SongView {
    pub title: String,
    pub artist: String,
    pub requester: String,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            app_name: APP_NAME,
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

impl StatusResponse {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            app: state.app_name,
            version: state.version,
            provider: "not_configured",
            current_song: None,
            queue_length: 0,
        }
    }
}
