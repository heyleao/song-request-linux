use anyhow::{anyhow, bail, Result};

use crate::{
    commands::ChatUserRole,
    config,
    song_requests::{MusicProvider, RequestSource, SongRequest, SongRequestInput},
    spotify,
    state::AppState,
    youtube,
};

pub async fn add_request(state: &AppState, input: SongRequestInput) -> Result<SongRequest> {
    add_request_for_role(state, input, ChatUserRole::Streamer).await
}

pub async fn add_request_for_role(
    state: &AppState,
    input: SongRequestInput,
    role: ChatUserRole,
) -> Result<SongRequest> {
    enforce_queue_limit(state, &input, role).await?;
    add_request_unchecked(state, input).await
}

async fn add_request_unchecked(state: &AppState, input: SongRequestInput) -> Result<SongRequest> {
    let default_provider = config::UiConfigView::load(&state.config.paths).default_provider;

    if let RequestSource::Youtube { video_id } =
        RequestSource::from_query_public(&input.query, default_provider)
    {
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

    if should_use_spotify(default_provider, &input) {
        let mut token_guard = state.spotify_token.write().await;
        let token = token_guard
            .as_mut()
            .ok_or_else(|| anyhow!("Spotify is not connected"))?;
        let mut request = spotify::search_and_queue(&state.config, token, &input.query).await?;
        request.requester = input.requester.trim().to_string();
        request.query = input.query.trim().to_string();

        return Ok(state.queue.write().await.add_resolved(request));
    }

    if should_use_youtube(default_provider, &input) {
        return add_youtube_search_request(state, input).await;
    }

    state.queue.write().await.add(input)
}

async fn add_youtube_search_request(
    state: &AppState,
    input: SongRequestInput,
) -> Result<SongRequest> {
    let metadata = youtube::search_and_validate(&state.config, &input.query).await?;
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

    Ok(state.queue.write().await.add_resolved(request))
}

async fn enforce_queue_limit(
    state: &AppState,
    input: &SongRequestInput,
    role: ChatUserRole,
) -> Result<()> {
    let config = config::UiConfigView::load(&state.config.paths);
    let limit = config.queue_limits.limit_for(role);
    if limit == 0 {
        return Ok(());
    }

    let pending = state
        .queue
        .read()
        .await
        .pending_count_by_requester(&input.requester);
    if pending >= usize::from(limit) {
        bail!(
            "limite de {} musica(s) pendente(s) para seu cargo atingido. Remova um pedido com !rm ou aguarde a fila andar.",
            limit
        );
    }

    Ok(())
}

fn should_use_spotify(default_provider: MusicProvider, input: &SongRequestInput) -> bool {
    matches!(default_provider, MusicProvider::Spotify)
        && !matches!(
            RequestSource::from_query_public(&input.query, MusicProvider::Spotify),
            RequestSource::Youtube { .. }
        )
}

fn should_use_youtube(default_provider: MusicProvider, input: &SongRequestInput) -> bool {
    matches!(default_provider, MusicProvider::Youtube)
        && !matches!(
            RequestSource::from_query_public(&input.query, MusicProvider::Youtube),
            RequestSource::Youtube { .. }
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn isolated_config(name: &str) -> AppConfig {
        let mut config = AppConfig::from_env().expect("config");
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("song-request-linux-{name}-{unique}"));
        config.paths.config_dir = root.join("config");
        config.paths.state_dir = root.join("state");
        config.paths.queue_file = config.paths.state_dir.join("queue.json");
        config
    }

    #[tokio::test]
    async fn queue_limit_blocks_second_viewer_request() {
        let config = isolated_config("limit");
        crate::config::save_ui_config(
            &config.paths,
            crate::config::UiConfigInput {
                default_provider: MusicProvider::Spotify,
                youtube_playback: crate::config::YoutubePlayback::Browser,
                pear_base_url: None,
                spotify_client_id: None,
                spotify_fallback_enabled: false,
                queue_persistence_enabled: false,
                twitch_client_id: None,
                twitch_bot_username: None,
                twitch_channel: None,
                twitch_bot_oauth_token: None,
                youtube_api_key: None,
                youtube_max_duration_seconds: Some(360),
                youtube_allow_non_music: false,
                command_settings: None,
                queue_limits: Some(crate::config::QueueLimitConfig {
                    viewer: 1,
                    vip: 3,
                    moderator: 10,
                    streamer: 0,
                }),
            },
        )
        .expect("save config");
        let state = AppState::new(config);

        add_request_for_role(
            &state,
            SongRequestInput {
                requester: "viewer".to_string(),
                query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            },
            ChatUserRole::Viewer,
        )
        .await
        .expect("first request");

        let error = add_request_for_role(
            &state,
            SongRequestInput {
                requester: "viewer".to_string(),
                query: "https://youtu.be/9bZkp7q19f0".to_string(),
            },
            ChatUserRole::Viewer,
        )
        .await
        .expect_err("second request should hit limit");

        assert!(error.to_string().contains("limite de 1 musica"));
    }

    #[tokio::test]
    async fn streamer_zero_limit_is_unlimited() {
        let config = isolated_config("streamer-unlimited");
        crate::config::save_ui_config(
            &config.paths,
            crate::config::UiConfigInput {
                default_provider: MusicProvider::Spotify,
                youtube_playback: crate::config::YoutubePlayback::Browser,
                pear_base_url: None,
                spotify_client_id: None,
                spotify_fallback_enabled: false,
                queue_persistence_enabled: false,
                twitch_client_id: None,
                twitch_bot_username: None,
                twitch_channel: None,
                twitch_bot_oauth_token: None,
                youtube_api_key: None,
                youtube_max_duration_seconds: Some(360),
                youtube_allow_non_music: false,
                command_settings: None,
                queue_limits: Some(crate::config::QueueLimitConfig::default()),
            },
        )
        .expect("save config");
        let state = AppState::new(config);

        for query in [
            "https://youtu.be/dQw4w9WgXcQ",
            "https://youtu.be/9bZkp7q19f0",
        ] {
            add_request_for_role(
                &state,
                SongRequestInput {
                    requester: "heyleao".to_string(),
                    query: query.to_string(),
                },
                ChatUserRole::Streamer,
            )
            .await
            .expect("streamer request");
        }

        assert_eq!(
            state
                .queue
                .read()
                .await
                .pending_count_by_requester("heyleao"),
            2
        );
    }

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
