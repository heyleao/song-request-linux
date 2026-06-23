use std::{sync::atomic::Ordering, time::Duration};

use tokio::time::{interval, sleep};

use crate::{
    config::{self, YoutubePlayback},
    pear,
    song_requests::{RequestSource, SongRequest},
    spotify,
    state::AppState,
};

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut shutdown = state.shutdown.subscribe();
        let mut ticker = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = ticker.tick() => coordinate_once(&state).await,
                _ = shutdown.recv() => break,
            }
        }
    });
}

async fn coordinate_once(state: &AppState) {
    if has_local_requests(state).await {
        state
            .spotify_fallback_started
            .store(false, Ordering::SeqCst);
    }

    if matches!(state.config.youtube.playback, YoutubePlayback::Pear) {
        coordinate_pear_once(state).await;
        return;
    }

    if !has_pending_youtube(state).await {
        *state.youtube_waiting_spotify_title.lock().await = None;
        start_spotify_fallback_if_idle(state).await;
        return;
    }

    if state.youtube_player_paused_spotify.load(Ordering::SeqCst) {
        return;
    }

    let waiting_title = state.youtube_waiting_spotify_title.lock().await.clone();
    let Some(waiting_title) = waiting_title else {
        arm_against_current_spotify(state).await;
        return;
    };

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        mark_spotify_released(state, "Spotify nao conectado; liberando pedido YouTube").await;
        return;
    };

    let playback = match spotify::current_playback(&state.config, token).await {
        Ok(playback) => playback,
        Err(error) => {
            state
                .record_event("error", format!("Nao consegui coordenar Spotify: {error}"))
                .await;
            return;
        }
    };
    drop(token_guard);

    let still_same_track = playback
        .as_ref()
        .is_some_and(|playback| playback.is_playing && playback.title == waiting_title);
    if still_same_track {
        return;
    }

    *state.youtube_waiting_spotify_title.lock().await = None;
    pause_spotify_for_youtube(state, "Spotify pausado para iniciar pedido YouTube").await;
}

async fn coordinate_pear_once(state: &AppState) {
    let Some(pending) = first_pending_youtube(state).await else {
        *state.youtube_waiting_spotify_title.lock().await = None;
        *state.youtube_active_pear_video_id.lock().await = None;
        *state.youtube_failed_pear_video_id.lock().await = None;
        resume_spotify_after_youtube(state).await;
        start_spotify_fallback_if_idle(state).await;
        return;
    };

    if state.youtube_player_paused_spotify.load(Ordering::SeqCst) {
        coordinate_active_pear_request(state, pending).await;
        return;
    }

    let waiting_title = state.youtube_waiting_spotify_title.lock().await.clone();
    let Some(waiting_title) = waiting_title else {
        arm_against_current_spotify(state).await;
        if state.youtube_player_paused_spotify.load(Ordering::SeqCst) {
            coordinate_active_pear_request(state, pending).await;
        }
        return;
    };

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        start_pear_request(state, pending).await;
        return;
    };

    let playback = match spotify::current_playback(&state.config, token).await {
        Ok(playback) => playback,
        Err(error) => {
            state
                .record_event("error", format!("Nao consegui coordenar Spotify: {error}"))
                .await;
            return;
        }
    };
    drop(token_guard);

    let still_same_track = playback
        .as_ref()
        .is_some_and(|playback| playback.is_playing && playback.title == waiting_title);
    if still_same_track {
        return;
    }

    *state.youtube_waiting_spotify_title.lock().await = None;
    pause_spotify_for_youtube(state, "Spotify pausado para iniciar pedido YouTube").await;
    if state.youtube_player_paused_spotify.load(Ordering::SeqCst) {
        coordinate_active_pear_request(state, pending).await;
    }
}

async fn coordinate_active_pear_request(state: &AppState, pending: SongRequest) {
    let RequestSource::Youtube { video_id } = pending.source.clone() else {
        return;
    };

    let active_video_id = state.youtube_active_pear_video_id.lock().await.clone();
    let pear_current = pear::now_playing(&state.config).await.ok();
    let pear_current_id = pear_current.as_ref().and_then(|song| song.video_id.clone());

    if active_video_id.as_deref() == Some(video_id.as_str()) {
        if pear_current_id.as_deref() == Some(video_id.as_str()) {
            if pear_current.as_ref().is_some_and(|song| song.is_paused) {
                if let Err(error) = pear::play(&state.config).await {
                    state
                        .record_event("error", format!("Nao consegui retomar Pear: {error}"))
                        .await;
                }
            }
            return;
        }

        finish_pear_request(state, pending.id).await;
        if let Some(next_pending) = first_pending_youtube(state).await {
            start_pear_request(state, next_pending).await;
        } else {
            resume_spotify_after_youtube(state).await;
        }
        return;
    }

    start_pear_request(state, pending).await;
}

async fn start_pear_request(state: &AppState, pending: SongRequest) {
    let RequestSource::Youtube { video_id } = pending.source else {
        return;
    };

    if state.youtube_failed_pear_video_id.lock().await.as_deref() == Some(video_id.as_str()) {
        return;
    }

    let already_selected = pear::now_playing(&state.config)
        .await
        .ok()
        .and_then(|song| song.video_id)
        .is_some_and(|current_video_id| current_video_id == video_id);

    if !already_selected {
        match pear::enqueue_after_current(&state.config, &video_id).await {
            Ok(()) => {}
            Err(error) => {
                state
                    .record_event(
                        "error",
                        format!("Nao consegui enviar pedido ao Pear: {error}"),
                    )
                    .await;
                *state.youtube_failed_pear_video_id.lock().await = Some(video_id);
                return;
            }
        }

        if let Err(error) = pear::next(&state.config).await {
            state
                .record_event(
                    "error",
                    format!("Nao consegui avancar Pear para o pedido: {error}"),
                )
                .await;
            *state.youtube_failed_pear_video_id.lock().await = Some(video_id);
            return;
        }
    }

    if let Err(error) = pear::play(&state.config).await {
        state
            .record_event("error", format!("Nao consegui iniciar Pear: {error}"))
            .await;
        *state.youtube_failed_pear_video_id.lock().await = Some(video_id);
        return;
    }

    sleep(Duration::from_millis(700)).await;
    let confirmed = pear::now_playing(&state.config)
        .await
        .ok()
        .and_then(|song| song.video_id)
        .is_some_and(|current_video_id| current_video_id == video_id);
    if !confirmed {
        state
            .record_event(
                "error",
                format!("Pear nao abriu o video solicitado: {}", pending.title),
            )
            .await;
        *state.youtube_failed_pear_video_id.lock().await = Some(video_id);
        return;
    }

    *state.youtube_failed_pear_video_id.lock().await = None;
    *state.youtube_active_pear_video_id.lock().await = Some(video_id);
    state
        .record_event("player", format!("Pear tocando pedido: {}", pending.title))
        .await;
}

async fn arm_against_current_spotify(state: &AppState) {
    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return;
    };

    let playback = match spotify::current_playback(&state.config, token).await {
        Ok(playback) => playback,
        Err(error) => {
            state
                .record_event(
                    "error",
                    format!("Nao consegui marcar Spotify atual: {error}"),
                )
                .await;
            return;
        }
    };
    drop(token_guard);

    let Some(playback) = playback else {
        pause_spotify_for_youtube(state, "Spotify pausado para pedido YouTube pendente").await;
        return;
    };

    if playback.is_playing {
        let title = playback.title;
        let mut waiting = state.youtube_waiting_spotify_title.lock().await;
        if waiting.as_deref() == Some(title.as_str()) {
            return;
        }
        *waiting = Some(title.clone());
        drop(waiting);
        pause_pear_while_waiting_for_spotify(state).await;
        state
            .record_event(
                "player",
                format!("YouTube aguardando fim do Spotify atual: {title}"),
            )
            .await;
    } else {
        pause_spotify_for_youtube(state, "Spotify pausado para pedido YouTube pendente").await;
    }
}

async fn pause_pear_while_waiting_for_spotify(state: &AppState) {
    if !matches!(state.config.youtube.playback, YoutubePlayback::Pear) {
        return;
    }

    if let Err(error) = pear::pause(&state.config).await {
        state
            .record_event(
                "error",
                format!("Nao consegui pausar Pear enquanto Spotify toca: {error}"),
            )
            .await;
    }
}

async fn first_pending_youtube(state: &AppState) -> Option<SongRequest> {
    state
        .queue
        .read()
        .await
        .first_youtube()
        .filter(|song| matches!(song.source, RequestSource::Youtube { .. }))
}

async fn has_pending_youtube(state: &AppState) -> bool {
    state
        .queue
        .read()
        .await
        .first_youtube()
        .is_some_and(|song| matches!(song.source, RequestSource::Youtube { .. }))
}

async fn has_local_requests(state: &AppState) -> bool {
    let view = state.queue.read().await.view();
    view.current_song.is_some() || !view.queue.is_empty()
}

async fn start_spotify_fallback_if_idle(state: &AppState) {
    if !config::UiConfigView::load(&state.config.paths).spotify_fallback_enabled
        || has_local_requests(state).await
    {
        return;
    }

    if state.spotify_fallback_started.load(Ordering::SeqCst) {
        return;
    }

    let playlist = match spotify::load_fallback_playlist(&state.config) {
        Ok(Some(playlist)) => playlist,
        Ok(None) => return,
        Err(error) => {
            if !state.spotify_fallback_started.swap(true, Ordering::SeqCst) {
                state
                    .record_event(
                        "error",
                        format!("Nao consegui ler playlist fallback: {error}"),
                    )
                    .await;
            }
            return;
        }
    };

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return;
    };

    match spotify::current_playback(&state.config, token).await {
        Ok(Some(playback)) if playback.is_playing => return,
        Ok(_) => {}
        Err(error) => {
            if !state.spotify_fallback_started.swap(true, Ordering::SeqCst) {
                state
                    .record_event(
                        "error",
                        format!("Nao consegui verificar Spotify fallback: {error}"),
                    )
                    .await;
            }
            return;
        }
    }

    if state
        .spotify_fallback_started
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    match spotify::play_context(&state.config, token, &playlist.uri).await {
        Ok(()) => {
            state
                .record_event(
                    "player",
                    format!("Fallback Spotify iniciado: {}", playlist.name),
                )
                .await;
        }
        Err(error) => {
            state
                .record_event(
                    "error",
                    format!("Nao consegui iniciar fallback Spotify: {error}"),
                )
                .await;
        }
    }
}

async fn pause_spotify_for_youtube(state: &AppState, message: &'static str) {
    if state.youtube_player_paused_spotify.load(Ordering::SeqCst) {
        return;
    }

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return;
    };

    match spotify::pause_playback(&state.config, token).await {
        Ok(()) => {
            state
                .youtube_player_paused_spotify
                .store(true, Ordering::SeqCst);
            state.record_event("player", message).await;
        }
        Err(error) => {
            let error = error.to_string();
            if error.contains("Restriction violated") {
                mark_spotify_released(
                    state,
                    "Spotify recusou pausa por restricao; liberando pedido YouTube",
                )
                .await;
                return;
            }

            if error.contains("Nenhum device Spotify disponivel") {
                mark_spotify_released(
                    state,
                    "Nenhum device Spotify ativo; liberando pedido YouTube",
                )
                .await;
                return;
            }

            state
                .record_event("error", format!("Nao consegui pausar Spotify: {}", error))
                .await;
        }
    }
}

async fn mark_spotify_released(state: &AppState, message: &'static str) {
    if state
        .youtube_player_paused_spotify
        .swap(true, Ordering::SeqCst)
    {
        return;
    }

    state.record_event("player", message).await;
}

async fn finish_pear_request(state: &AppState, id: u64) {
    {
        let mut queue = state.queue.write().await;
        queue.remove_by_id(id);
        if crate::config::queue_persistence_enabled(&state.config.paths) {
            if let Err(error) = queue.save(&state.config.paths.queue_file) {
                state
                    .record_event("error", format!("Nao consegui salvar fila: {error}"))
                    .await;
            }
        }
    }

    *state.youtube_active_pear_video_id.lock().await = None;
    *state.youtube_failed_pear_video_id.lock().await = None;
    state.record_event("player", "Pedido Pear finalizado").await;
}

async fn resume_spotify_after_youtube(state: &AppState) {
    if !state
        .youtube_player_paused_spotify
        .swap(false, Ordering::SeqCst)
    {
        return;
    }

    if let Err(error) = pear::pause(&state.config).await {
        state
            .record_event("error", format!("Nao consegui pausar Pear: {error}"))
            .await;
    }

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return;
    };

    match spotify::resume_playback(&state.config, token).await {
        Ok(()) => {
            state
                .record_event("player", "Spotify retomado apos fila YouTube")
                .await;
        }
        Err(error) => {
            state
                .record_event("error", format!("Nao consegui retomar Spotify: {error}"))
                .await;
        }
    }
}
