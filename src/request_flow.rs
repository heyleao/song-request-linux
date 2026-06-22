use anyhow::{anyhow, Result};

use crate::{
    song_requests::{MusicProvider, RequestSource, SongRequest, SongRequestInput},
    spotify,
    state::AppState,
};

pub async fn add_request(state: &AppState, input: SongRequestInput) -> Result<SongRequest> {
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

    state.queue.write().await.add(input)
}

fn should_use_spotify(state: &AppState, input: &SongRequestInput) -> bool {
    matches!(state.config.default_provider, MusicProvider::Spotify)
        && !matches!(
            RequestSource::from_query_public(&input.query, MusicProvider::Spotify),
            RequestSource::Youtube { .. }
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::AppConfig, song_requests::RequestSource};

    #[tokio::test]
    async fn youtube_link_uses_app_queue_even_when_spotify_is_default() {
        let mut config = AppConfig::from_env().expect("config");
        config.default_provider = MusicProvider::Spotify;
        let state = AppState::new(config);

        let request = add_request(
            &state,
            SongRequestInput {
                requester: "viewer".to_string(),
                query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            },
        )
        .await
        .expect("youtube request");

        assert!(matches!(request.source, RequestSource::Youtube { .. }));
    }
}
