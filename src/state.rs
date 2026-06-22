use std::{
    collections::VecDeque,
    sync::{atomic::AtomicBool, Arc},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tokio::sync::{broadcast, RwLock};

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
    pub twitch_bot_running: Arc<AtomicBool>,
    pub events: Arc<RwLock<EventLog>>,
    pub shutdown: broadcast::Sender<()>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AppEvent {
    pub id: u64,
    pub timestamp: u64,
    pub kind: &'static str,
    pub message: String,
}

#[derive(Debug)]
pub struct EventLog {
    next_id: u64,
    events: VecDeque<AppEvent>,
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
        let (shutdown, _) = broadcast::channel(1);

        Self {
            spotify_token: Arc::new(RwLock::new(load_token(&config).ok().flatten())),
            config,
            app_name: APP_NAME,
            version: env!("CARGO_PKG_VERSION"),
            queue: Arc::new(RwLock::new(queue)),
            spotify_auth: Arc::new(RwLock::new(None)),
            twitch_bot_running: Arc::new(AtomicBool::new(false)),
            events: Arc::new(RwLock::new(EventLog::new())),
            shutdown,
        }
    }

    pub async fn record_event(&self, kind: &'static str, message: impl Into<String>) {
        self.events.write().await.push(kind, message.into());
    }
}

impl EventLog {
    fn new() -> Self {
        Self {
            next_id: 1,
            events: VecDeque::new(),
        }
    }

    fn push(&mut self, kind: &'static str, message: String) {
        let event = AppEvent {
            id: self.next_id,
            timestamp: unix_now(),
            kind,
            message,
        };
        self.next_id += 1;
        self.events.push_front(event);

        while self.events.len() > 120 {
            self.events.pop_back();
        }
    }

    pub fn recent(&self, limit: usize) -> Vec<AppEvent> {
        self.events.iter().take(limit).cloned().collect()
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
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
