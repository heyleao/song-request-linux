use anyhow::{anyhow, bail, Context, Result};

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
        if matches!(default_provider, MusicProvider::Spotify) {
            bail!(
                "modo Spotify ativo: link do YouTube nao entra na fila. Troque o provider para YouTube/Pear ou YouTube/OBS, ou peça por texto para buscar no Spotify."
            );
        }
        return add_youtube_url_request(state, input, video_id).await;
    }

    if should_use_spotify(default_provider, &input) {
        let mut token_guard = state.spotify_token.write().await;
        let token = token_guard
            .as_mut()
            .ok_or_else(|| anyhow!("Spotify is not connected"))?;
        let mut request = spotify::search_and_queue(&state.config, token, &input.query).await?;
        request.requester = input.requester.trim().to_string();
        request.query = input.query.trim().to_string();

        return add_resolved_and_persist(state, request).await;
    }

    if should_use_youtube(default_provider, &input) {
        return add_youtube_search_request(state, input).await;
    }

    let request = state.queue.write().await.add(input)?;
    persist_queue_if_enabled(state).await?;
    Ok(request)
}

async fn add_youtube_url_request(
    state: &AppState,
    input: SongRequestInput,
    video_id: String,
) -> Result<SongRequest> {
    let metadata = youtube::video_metadata(
        &state.config,
        &youtube::YoutubeVideoRef {
            video_id: video_id.clone(),
        },
    )
    .await;

    let (title, artist) = match metadata {
        Ok(metadata) => (metadata.title, metadata.channel_title),
        Err(error) => {
            state
                .record_event(
                    "youtube",
                    format!("Nao consegui ler metadata do link YouTube {video_id}: {error}"),
                )
                .await;
            ("YouTube URL".to_string(), "YouTube".to_string())
        }
    };

    let request = SongRequest {
        id: 0,
        requester: input.requester.trim().to_string(),
        query: input.query.trim().to_string(),
        source: RequestSource::Youtube { video_id },
        title,
        artist,
    };

    add_resolved_and_persist(state, request).await
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

    add_resolved_and_persist(state, request).await
}

async fn add_resolved_and_persist(state: &AppState, request: SongRequest) -> Result<SongRequest> {
    let request = state.queue.write().await.add_resolved(request);
    persist_queue_if_enabled(state).await?;
    Ok(request)
}

async fn persist_queue_if_enabled(state: &AppState) -> Result<()> {
    if !config::queue_persistence_enabled(&state.config.paths) {
        return Ok(());
    }

    state
        .queue
        .read()
        .await
        .save(&state.config.paths.queue_file)
        .with_context(|| {
            format!(
                "failed to persist queue at {}",
                state.config.paths.queue_file.display()
            )
        })
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
    let source = RequestSource::from_query_public(&input.query, MusicProvider::Spotify);
    matches!(source, RequestSource::Spotify { .. })
        || (matches!(default_provider, MusicProvider::Spotify)
            && !matches!(source, RequestSource::Youtube { .. }))
}

fn should_use_youtube(default_provider: MusicProvider, input: &SongRequestInput) -> bool {
    let source = RequestSource::from_query_public(&input.query, MusicProvider::Youtube);
    matches!(default_provider, MusicProvider::Youtube)
        && matches!(source, RequestSource::Search { .. })
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
                default_provider: MusicProvider::Youtube,
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
                    follower: 1,
                    subscriber: 2,
                    vip: 3,
                    moderator: 10,
                    streamer: 0,
                }),
                overlay: None,
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
                default_provider: MusicProvider::Youtube,
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
                overlay: None,
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
    async fn youtube_link_is_rejected_when_spotify_is_default() {
        let config = isolated_config("youtube-link-spotify-mode");
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
                overlay: None,
            },
        )
        .expect("save config");
        let state = AppState::new(config);

        let error = add_request(
            &state,
            SongRequestInput {
                requester: "viewer".to_string(),
                query: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            },
        )
        .await
        .expect_err("youtube urls should be blocked in spotify mode");

        assert!(error.to_string().contains("modo Spotify ativo"));
    }

    #[test]
    fn spotify_link_uses_spotify_even_when_youtube_is_default() {
        let input = SongRequestInput {
            requester: "viewer".to_string(),
            query: "https://open.spotify.com/track/3YxaaLqXvyWhQJwVFlvVVa?si=test".to_string(),
        };

        assert!(should_use_spotify(MusicProvider::Youtube, &input));
        assert!(!should_use_youtube(MusicProvider::Youtube, &input));
    }
}
