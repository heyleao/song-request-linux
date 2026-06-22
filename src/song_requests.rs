use std::collections::VecDeque;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::youtube::YoutubeVideoRef;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SongRequestInput {
    pub requester: String,
    pub query: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SongRequest {
    pub id: u64,
    pub requester: String,
    pub query: String,
    pub source: RequestSource,
    pub title: String,
    pub artist: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestSource {
    Search { provider: MusicProvider },
    Spotify { uri: String },
    Youtube { video_id: String },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MusicProvider {
    Spotify,
    Youtube,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueueView {
    pub current_song: Option<SongRequest>,
    pub queue: Vec<SongRequest>,
    pub queue_length: usize,
}

#[derive(Clone, Debug)]
pub struct SongQueue {
    next_id: u64,
    current_song: Option<SongRequest>,
    queue: VecDeque<SongRequest>,
    default_provider: MusicProvider,
}

impl Default for SongQueue {
    fn default() -> Self {
        Self {
            next_id: 1,
            current_song: None,
            queue: VecDeque::new(),
            default_provider: MusicProvider::Youtube,
        }
    }
}

impl SongQueue {
    pub fn new(default_provider: MusicProvider) -> Self {
        Self {
            default_provider,
            ..Self::default()
        }
    }

    pub fn add(&mut self, input: SongRequestInput) -> Result<SongRequest> {
        validate_input(&input)?;

        let source = RequestSource::from_query(&input.query, self.default_provider);
        let request = SongRequest {
            id: self.allocate_id(),
            requester: input.requester.trim().to_string(),
            title: title_from_source(&source, &input.query),
            artist: artist_from_source(&source),
            query: input.query.trim().to_string(),
            source,
        };

        if self.current_song.is_none() {
            self.current_song = Some(request.clone());
        } else {
            self.queue.push_back(request.clone());
        }

        Ok(request)
    }

    pub fn add_resolved(&mut self, mut request: SongRequest) -> SongRequest {
        request.id = self.allocate_id();

        if self.current_song.is_none() {
            self.current_song = Some(request.clone());
        } else {
            self.queue.push_back(request.clone());
        }

        request
    }

    pub fn skip(&mut self) -> Option<SongRequest> {
        self.current_song = self.queue.pop_front();
        self.current_song.clone()
    }

    pub fn first_youtube(&self) -> Option<SongRequest> {
        self.current_song
            .iter()
            .chain(self.queue.iter())
            .find(|song| matches!(song.source, RequestSource::Youtube { .. }))
            .cloned()
    }

    pub fn remove_by_id(&mut self, id: u64) -> Option<SongRequest> {
        if self.current_song.as_ref().is_some_and(|song| song.id == id) {
            let removed = self.current_song.take();
            self.current_song = self.queue.pop_front();
            return removed;
        }

        let index = self.queue.iter().position(|song| song.id == id)?;
        self.queue.remove(index)
    }

    pub fn view(&self) -> QueueView {
        QueueView {
            current_song: self.current_song.clone(),
            queue: self.queue.iter().cloned().collect(),
            queue_length: self.queue.len(),
        }
    }

    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl MusicProvider {
    pub fn from_env() -> Option<Self> {
        match std::env::var("SONG_REQUEST_PROVIDER") {
            Ok(value) if value.eq_ignore_ascii_case("spotify") => Some(Self::Spotify),
            Ok(value) if value.eq_ignore_ascii_case("youtube") => Some(Self::Youtube),
            _ => None,
        }
    }
}

impl RequestSource {
    pub fn from_query_public(query: &str, default_provider: MusicProvider) -> Self {
        Self::from_query(query, default_provider)
    }

    fn from_query(query: &str, default_provider: MusicProvider) -> Self {
        if let Some(video) = YoutubeVideoRef::parse(query) {
            return Self::Youtube {
                video_id: video.video_id,
            };
        }

        Self::Search {
            provider: default_provider,
        }
    }
}

fn validate_input(input: &SongRequestInput) -> Result<()> {
    if input.requester.trim().is_empty() {
        bail!("requester is required");
    }

    let query = input.query.trim();
    if query.is_empty() {
        bail!("query is required");
    }

    if query.chars().count() > 300 {
        bail!("query is too long");
    }

    Ok(())
}

fn title_from_source(source: &RequestSource, query: &str) -> String {
    match source {
        RequestSource::Youtube { video_id } => format!("YouTube video {video_id}"),
        RequestSource::Spotify { .. } => query.trim().to_string(),
        RequestSource::Search { .. } => query.trim().to_string(),
    }
}

fn artist_from_source(source: &RequestSource) -> String {
    match source {
        RequestSource::Youtube { .. } => "YouTube".to_string(),
        RequestSource::Spotify { .. } => "Spotify".to_string(),
        RequestSource::Search { provider } => match provider {
            MusicProvider::Spotify => "Spotify search".to_string(),
            MusicProvider::Youtube => "YouTube search".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_request_becomes_current_song() {
        let mut queue = SongQueue::default();
        let request = queue
            .add(SongRequestInput {
                requester: "bruno".to_string(),
                query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            })
            .expect("request");

        let view = queue.view();

        assert_eq!(view.current_song, Some(request));
        assert_eq!(view.queue_length, 0);
    }

    #[test]
    fn second_request_waits_in_queue() {
        let mut queue = SongQueue::default();
        queue
            .add(SongRequestInput {
                requester: "one".to_string(),
                query: "first song".to_string(),
            })
            .expect("first");
        queue
            .add(SongRequestInput {
                requester: "two".to_string(),
                query: "second song".to_string(),
            })
            .expect("second");

        let view = queue.view();

        assert_eq!(view.current_song.expect("current").query, "first song");
        assert_eq!(view.queue_length, 1);
    }

    #[test]
    fn skip_advances_queue() {
        let mut queue = SongQueue::default();
        queue
            .add(SongRequestInput {
                requester: "one".to_string(),
                query: "first song".to_string(),
            })
            .expect("first");
        queue
            .add(SongRequestInput {
                requester: "two".to_string(),
                query: "second song".to_string(),
            })
            .expect("second");

        let current = queue.skip().expect("next");

        assert_eq!(current.query, "second song");
        assert_eq!(queue.view().queue_length, 0);
    }

    #[test]
    fn first_youtube_finds_request_behind_spotify() {
        let mut queue = SongQueue::new(MusicProvider::Spotify);
        queue.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "daft punk one more time".to_string(),
            source: RequestSource::Spotify {
                uri: "spotify:track:test".to_string(),
            },
            title: "One More Time".to_string(),
            artist: "Daft Punk".to_string(),
        });
        let youtube = queue.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            source: RequestSource::Youtube {
                video_id: "dQw4w9WgXcQ".to_string(),
            },
            title: "Never Gonna Give You Up".to_string(),
            artist: "Rick Astley".to_string(),
        });

        assert_eq!(queue.first_youtube(), Some(youtube));
    }

    #[test]
    fn remove_by_id_removes_queued_youtube_without_advancing_spotify() {
        let mut queue = SongQueue::new(MusicProvider::Spotify);
        let spotify = queue.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "daft punk one more time".to_string(),
            source: RequestSource::Spotify {
                uri: "spotify:track:test".to_string(),
            },
            title: "One More Time".to_string(),
            artist: "Daft Punk".to_string(),
        });
        let youtube = queue.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            source: RequestSource::Youtube {
                video_id: "dQw4w9WgXcQ".to_string(),
            },
            title: "Never Gonna Give You Up".to_string(),
            artist: "Rick Astley".to_string(),
        });

        assert_eq!(queue.remove_by_id(youtube.id), Some(youtube));
        assert_eq!(queue.view().current_song, Some(spotify));
        assert!(queue.first_youtube().is_none());
    }
}
