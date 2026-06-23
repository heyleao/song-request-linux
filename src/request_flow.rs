use anyhow::{anyhow, Result};

use crate::{
    config::YoutubePlayback,
    pear,
    song_requests::{MusicProvider, RequestSource, SongRequest, SongRequestInput},
    spotify,
    state::AppState,
    youtube,
};

pub async fn add_request(state: &AppState, input: SongRequestInput) -> Result<SongRequest> {
    if let RequestSource::Youtube { video_id } =
        RequestSource::from_query_public(&input.query, state.config.default_provider)
    {
        if matches!(state.config.youtube.playback, YoutubePlayback::Pear) {
            pear::enqueue_after_current(&state.config, &video_id).await?;
        }
        let request = SongRequest {
            id: 0,
            requester: input.requester.trim().to_string(),
            query: input.query.trim().to_string(),
            source: RequestSource::Youtube { video_id },
            title: "YouTube URL".to_string(),
            artist: "YouTube".to_string(),
        };

        return Ok(state.queue.write().await.add_resolved(request));
    }

    if should_use_spotify(state, &input) {
        let mut token_guard = state.spotify_token.write().await;
        let token = token_guard
            .as_mut()
            .ok_or_else(|| anyhow!("Spotify is not connected"))?;
        let mut request = spotify::search_and_queue(&state.config, token, &input.query).await?;
        request.requester = input.requester.trim().to_string();
        request.query = input.query.trim().to_string();

        return Ok(state.queue.write().await.add_resolved(request));
    }

    if should_use_youtube(state, &input) {
        let metadata = youtube::search_and_validate(&state.config, &input.query).await?;
        if matches!(state.config.youtube.playback, YoutubePlayback::Pear) {
            pear::enqueue_after_current(&state.config, &metadata.video_id).await?;
        }
        let request = SongRequest {
            id: 0,
            requester: input.requester.trim().to_string(),
            query: input.query.trim().to_string(),
            source: RequestSource::Youtube {
                video_id: metadata.video_id,
            },
            title: metadata.title,
            artist: metadata.channel_title,
        };

        return Ok(state.queue.write().await.add_resolved(request));
    }

    state.queue.write().await.add(input)
}

fn should_use_spotify(state: &AppState, input: &SongRequestInput) -> bool {
    matches!(state.config.default_provider, MusicProvider::Spotify)
        && !matches!(
            RequestSource::from_query_public(&input.query, MusicProvider::Spotify),
            RequestSource::Youtube { .. }
        )
}

fn should_use_youtube(state: &AppState, input: &SongRequestInput) -> bool {
    matches!(state.config.default_provider, MusicProvider::Youtube)
        && !matches!(
            RequestSource::from_query_public(&input.query, MusicProvider::Youtube),
            RequestSource::Youtube { .. }
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn youtube_link_requires_metadata_validation_even_when_spotify_is_default() {
        let mut config = AppConfig::from_env().expect("config");
        config.default_provider = MusicProvider::Spotify;
        config.youtube.api_key = None;
        config.youtube.playback = crate::config::YoutubePlayback::Browser;
        let state = AppState::new(config);

        let request = add_request(
            &state,
            SongRequestInput {
                requester: "viewer".to_string(),
                query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            },
        )
        .await
        .expect("youtube urls should not require api key");

        assert_eq!(request.title, "YouTube URL");
    }
}
