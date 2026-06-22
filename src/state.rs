use std::sync::Arc;

use serde::Serialize;
use tokio::sync::RwLock;

use crate::config::{AppConfig, APP_NAME};
use crate::song_requests::{QueueView, SongQueue};
use crate::spotify::{load_token, SpotifyAuthSession, SpotifyToken};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub app_name: &'static str,
    pub version: &'static str,
    pub queue: Arc<RwLock<SongQueue>>,
    pub spotify_auth: Arc<RwLock<Option<SpotifyAuthSession>>>,
    pub spotify_token: Arc<RwLock<Option<SpotifyToken>>>,
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
        let queue = SongQueue::new(config.default_provider);

        Self {
            spotify_token: Arc::new(RwLock::new(load_token(&config).ok().flatten())),
            config,
            app_name: APP_NAME,
            version: env!("CARGO_PKG_VERSION"),
            queue: Arc::new(RwLock::new(queue)),
            spotify_auth: Arc::new(RwLock::new(None)),
        }
    }
}

impl StatusResponse {
    pub fn from_queue(state: &AppState, queue: QueueView) -> Self {
        Self {
            app: state.app_name,
            version: state.version,
            provider: match state.config.default_provider {
                crate::song_requests::MusicProvider::Spotify => "spotify",
                crate::song_requests::MusicProvider::Youtube => "youtube",
            },
            current_song: queue.current_song.map(SongView::from),
            queue_length: queue.queue_length,
        }
    }
}

impl From<crate::song_requests::SongRequest> for SongView {
    fn from(song: crate::song_requests::SongRequest) -> Self {
        Self {
            title: song.title,
            artist: song.artist,
            requester: song.requester,
        }
    }
}
