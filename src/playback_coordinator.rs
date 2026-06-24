use std::{sync::atomic::Ordering, time::Duration};

use tokio::time::{interval, sleep};

use crate::{
    config::{self, YoutubePlayback},
    display, pear,
    song_requests::{MusicProvider, RequestSource, SongRequest},
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
    let ui_config = config::UiConfigView::load(&state.config.paths);
    if has_local_requests(state).await {
        state
            .spotify_fallback_started
            .store(false, Ordering::SeqCst);
        state.pear_idle_stopped.store(false, Ordering::SeqCst);
    }

    if !matches!(ui_config.default_provider, MusicProvider::Spotify) {
        state
            .spotify_fallback_started
            .store(false, Ordering::SeqCst);
    }

    if matches!(ui_config.default_provider, MusicProvider::Spotify) {
        *state.youtube_waiting_spotify_title.lock().await = None;
        state
            .youtube_player_paused_spotify
            .store(false, Ordering::SeqCst);
        coordinate_spotify_requests_once(state).await;
        start_spotify_fallback_if_idle(state).await;
        return;
    }

    if matches!(ui_config.default_provider, MusicProvider::Youtube)
        && matches!(ui_config.youtube_playback, YoutubePlayback::Pear)
    {
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

async fn coordinate_spotify_requests_once(state: &AppState) {
    let Some(request) = first_pending_spotify(state).await else {
        return;
    };

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return;
    };

    match spotify::ensure_request_queued(&state.config, token, &request).await {
        Ok(true) => {
            state
                .record_event(
                    "player",
                    format!(
                        "Pedido Spotify restaurado na fila: {}",
                        display::chat_song_title(&request)
                    ),
                )
                .await;
        }
        Ok(false) => {}
        Err(error) => {
            state
                .record_event(
                    "error",
                    format!("Nao consegui restaurar pedido Spotify salvo: {error}"),
                )
                .await;
        }
    }
}

async fn coordinate_pear_once(state: &AppState) {
    let Some(pending) = first_pending_youtube(state).await else {
        *state.youtube_waiting_spotify_title.lock().await = None;
        *state.pear_waiting_video_id.lock().await = None;
        *state.youtube_active_pear_video_id.lock().await = None;
        *state.youtube_failed_pear_video_id.lock().await = None;
        finish_pear_background(state).await;
        return;
    };

    if state.youtube_active_pear_video_id.lock().await.is_some()
        || state.youtube_player_paused_spotify.load(Ordering::SeqCst)
    {
        coordinate_active_pear_request(state, pending).await;
        return;
    }

    if wait_for_current_spotify_before_pear(state).await {
        return;
    }

    if wait_for_current_pear_before_request(state, &pending).await {
        return;
    }

    *state.youtube_waiting_spotify_title.lock().await = None;
    *state.pear_waiting_video_id.lock().await = None;
    start_pear_request(state, pending).await;
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
            prime_next_pear_request(state, pending.id).await;
            return;
        }

        finish_pear_request(state, pending.id).await;
        if let Some(next_pending) = first_pending_youtube(state).await {
            start_pear_request(state, next_pending).await;
        } else {
            finish_pear_background(state).await;
        }
        return;
    }

    start_pear_request(state, pending).await;
}

async fn start_pear_request(state: &AppState, pending: SongRequest) {
    let RequestSource::Youtube { video_id } = &pending.source else {
        return;
    };
    let video_id = video_id.clone();

    if state.youtube_failed_pear_video_id.lock().await.as_deref() == Some(video_id.as_str()) {
        return;
    }

    let already_selected = pear::now_playing(&state.config)
        .await
        .ok()
        .and_then(|song| song.video_id)
        .is_some_and(|current_video_id| current_video_id == video_id);

    if !already_selected {
        if let Err(error) = pear::clear_queue(&state.config).await {
            state
                .record_event(
                    "error",
                    format!("Nao consegui limpar fila interna do Pear: {error}"),
                )
                .await;
        }

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

        match pear::select_video_from_queue(&state.config, &video_id).await {
            Ok(true) => {}
            Ok(false) => {
                state
                    .record_event(
                        "player",
                        "Pear ainda nao listou o pedido na fila; aguardando sincronizar",
                    )
                    .await;
                return;
            }
            Err(error) => {
                state
                    .record_event(
                        "error",
                        format!("Nao consegui selecionar pedido no Pear: {error}"),
                    )
                    .await;
                return;
            }
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
        let display_title = display::chat_song_title(&pending);
        if let Err(error) = pear::pause(&state.config).await {
            state
                .record_event(
                    "error",
                    format!("Pear abriu musica errada e nao consegui pausar: {error}"),
                )
                .await;
        }
        state
            .record_event(
                "error",
                format!(
                    "Pear nao abriu o video solicitado: {display_title}. Playback pausado para evitar musica errada."
                ),
            )
            .await;
        *state.youtube_failed_pear_video_id.lock().await = Some(video_id);
        return;
    }

    *state.youtube_failed_pear_video_id.lock().await = None;
    *state.youtube_active_pear_video_id.lock().await = Some(video_id);
    let display_title = display::chat_song_title(&pending);
    state
        .record_event("player", format!("Pear tocando pedido: {display_title}"))
        .await;
    prime_next_pear_request(state, pending.id).await;
    compact_pear_to_app_queue(state, pending.id).await;
}

async fn prime_next_pear_request(state: &AppState, current_id: u64) {
    let Some(next) = next_youtube_after(state, current_id).await else {
        return;
    };
    let RequestSource::Youtube { video_id } = next.source else {
        return;
    };

    match pear::ensure_queued_after_current(&state.config, &video_id).await {
        Ok(()) => {}
        Err(error) => {
            state
                .record_event(
                    "error",
                    format!("Nao consegui preparar proximo pedido no Pear: {error}"),
                )
                .await;
        }
    }
}

async fn compact_pear_to_app_queue(state: &AppState, current_id: u64) {
    let Some(current) = current_youtube_by_id(state, current_id).await else {
        return;
    };
    let RequestSource::Youtube {
        video_id: current_video_id,
    } = current.source
    else {
        return;
    };
    let next_video_id = next_youtube_after(state, current_id)
        .await
        .and_then(|song| match song.source {
            RequestSource::Youtube { video_id } => Some(video_id),
            _ => None,
        });

    if let Err(error) =
        pear::compact_queue_for_app(&state.config, &current_video_id, next_video_id.as_deref())
            .await
    {
        state
            .record_event(
                "error",
                format!("Nao consegui compactar fila do Pear: {error}"),
            )
            .await;
    }
}

async fn wait_for_current_pear_before_request(state: &AppState, pending: &SongRequest) -> bool {
    let RequestSource::Youtube {
        video_id: pending_video_id,
    } = &pending.source
    else {
        return false;
    };

    let current = match pear::now_playing(&state.config).await {
        Ok(current) => current,
        Err(_) => {
            *state.pear_waiting_video_id.lock().await = None;
            return false;
        }
    };

    if current.is_paused || current.video_id.as_deref() == Some(pending_video_id.as_str()) {
        *state.pear_waiting_video_id.lock().await = None;
        return false;
    }

    let current_key = pear_playback_key(&current);
    let Some(current_key) = current_key else {
        return false;
    };

    let mut waiting = state.pear_waiting_video_id.lock().await;
    match waiting.as_deref() {
        Some(waiting_key) if waiting_key == current_key => true,
        Some(_) => {
            *waiting = None;
            false
        }
        None => {
            let display = current
                .clone()
                .display_name()
                .unwrap_or_else(|| "musica atual do Pear".to_string());
            *waiting = Some(current_key.to_string());
            drop(waiting);
            state
                .record_event(
                    "player",
                    format!("Pear aguardando fim da musica atual: {display}"),
                )
                .await;
            true
        }
    }
}

fn pear_playback_key(song: &pear::PearNowPlaying) -> Option<&str> {
    song.video_id
        .as_deref()
        .or(song.title.as_deref())
        .filter(|value| !value.trim().is_empty())
}

async fn wait_for_current_spotify_before_pear(state: &AppState) -> bool {
    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return false;
    };

    let playback = match spotify::current_playback(&state.config, token).await {
        Ok(playback) => playback,
        Err(error) => {
            tracing::debug!(%error, "Spotify unavailable while coordinating Pear");
            return false;
        }
    };
    drop(token_guard);

    let Some(playback) = playback else {
        return false;
    };

    if !playback.is_playing {
        return false;
    }

    let mut waiting = state.youtube_waiting_spotify_title.lock().await;
    if waiting.as_deref() == Some(playback.title.as_str()) {
        return true;
    }

    *waiting = Some(playback.title.clone());
    drop(waiting);
    pause_pear_while_waiting_for_spotify(state).await;
    state
        .record_event(
            "player",
            format!("Pear aguardando fim do Spotify atual: {}", playback.title),
        )
        .await;
    true
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
        .view()
        .current_song
        .filter(|song| matches!(song.source, RequestSource::Youtube { .. }))
}

async fn first_pending_spotify(state: &AppState) -> Option<SongRequest> {
    let view = state.queue.read().await.view();
    view.current_song
        .into_iter()
        .chain(view.queue)
        .find(is_spotify_request)
}

fn is_spotify_request(song: &SongRequest) -> bool {
    matches!(
        song.source,
        RequestSource::Spotify { .. }
            | RequestSource::Search {
                provider: MusicProvider::Spotify
            }
    )
}

async fn next_youtube_after(state: &AppState, current_id: u64) -> Option<SongRequest> {
    let view = state.queue.read().await.view();
    let mut songs = Vec::with_capacity(usize::from(view.current_song.is_some()) + view.queue.len());
    if let Some(current) = view.current_song {
        songs.push(current);
    }
    songs.extend(view.queue);

    songs
        .windows(2)
        .find(|window| window[0].id == current_id)
        .and_then(|window| {
            window
                .get(1)
                .filter(|song| matches!(song.source, RequestSource::Youtube { .. }))
                .cloned()
        })
}

async fn current_youtube_by_id(state: &AppState, current_id: u64) -> Option<SongRequest> {
    let view = state.queue.read().await.view();
    view.current_song
        .into_iter()
        .chain(view.queue)
        .find(|song| song.id == current_id && matches!(song.source, RequestSource::Youtube { .. }))
}

async fn has_pending_youtube(state: &AppState) -> bool {
    state
        .queue
        .read()
        .await
        .view()
        .current_song
        .is_some_and(|song| matches!(song.source, RequestSource::Youtube { .. }))
}

async fn has_local_requests(state: &AppState) -> bool {
    let provider = config::UiConfigView::load(&state.config.paths).default_provider;
    let view = state.queue.read().await.view();
    view.current_song
        .iter()
        .chain(view.queue.iter())
        .any(|song| match provider {
            MusicProvider::Spotify => is_spotify_request(song),
            MusicProvider::Youtube => matches!(song.source, RequestSource::Youtube { .. }),
        })
}

async fn start_spotify_fallback_if_idle(state: &AppState) {
    let ui_config = config::UiConfigView::load(&state.config.paths);
    if !matches!(ui_config.default_provider, MusicProvider::Spotify) {
        state
            .spotify_fallback_started
            .store(false, Ordering::SeqCst);
        return;
    }

    if !ui_config.spotify_fallback_enabled || has_local_requests(state).await {
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
        Ok(Some(playback))
            if playback.is_playing
                && playback.context_uri.as_deref() == Some(playlist.uri.as_str()) =>
        {
            *state.spotify_manual_playback_title.lock().await = None;
            state.spotify_fallback_started.store(true, Ordering::SeqCst);
            return;
        }
        Ok(Some(playback)) if playback.context_uri.as_deref() == Some(playlist.uri.as_str()) => {
            remember_manual_spotify_pause(state, &playback.title).await;
            return;
        }
        Ok(Some(playback)) => {
            if !playback.is_playing {
                remember_manual_spotify_pause(state, &playback.title).await;
                return;
            } else if spotify_playback_finished(&playback) {
                *state.spotify_manual_playback_title.lock().await = None;
            } else {
                remember_manual_spotify_playback(state, &playback.title).await;
                return;
            }
        }
        Ok(None) => {
            *state.spotify_manual_playback_title.lock().await = None;
        }
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

    match spotify::play_context(&state.config, token, &playlist.uri).await {
        Ok(()) => {
            state.spotify_fallback_started.store(true, Ordering::SeqCst);
            state
                .record_event(
                    "player",
                    format!("Fallback Spotify iniciado: {}", playlist.name),
                )
                .await;
        }
        Err(error) => {
            if !state.spotify_fallback_started.swap(true, Ordering::SeqCst) {
                state
                    .record_event(
                        "error",
                        format!("Nao consegui iniciar fallback Spotify: {error}"),
                    )
                    .await;
            }
        }
    }
}

fn spotify_playback_finished(playback: &spotify::SpotifyPlayback) -> bool {
    if playback.is_playing {
        return false;
    }

    let (Some(progress_ms), Some(duration_ms)) = (playback.progress_ms, playback.duration_ms)
    else {
        return false;
    };

    duration_ms > 0 && progress_ms.saturating_add(1_500) >= duration_ms
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

async fn remember_manual_spotify_playback(state: &AppState, title: &str) {
    let mut manual_title = state.spotify_manual_playback_title.lock().await;
    if manual_title.as_deref() == Some(title) {
        return;
    }

    *manual_title = Some(title.to_string());
    drop(manual_title);

    state
        .record_event(
            "player",
            format!("Spotify manual detectado; fallback aguardando fim: {title}"),
        )
        .await;
}

async fn remember_manual_spotify_pause(state: &AppState, title: &str) {
    let mut manual_title = state.spotify_manual_playback_title.lock().await;
    if manual_title.as_deref() == Some(title) {
        return;
    }

    *manual_title = Some(title.to_string());
    drop(manual_title);

    state
        .record_event(
            "player",
            format!("Spotify pausado manualmente; fallback aguardando play: {title}"),
        )
        .await;
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
    *state.pear_waiting_video_id.lock().await = None;
    state.record_event("player", "Pedido Pear finalizado").await;
}

async fn finish_pear_background(state: &AppState) {
    if state.youtube_player_paused_spotify.load(Ordering::SeqCst) {
        resume_spotify_after_youtube(state).await;
        state.pear_idle_stopped.store(true, Ordering::SeqCst);
        return;
    }

    if should_let_pear_continue_when_idle(state).await {
        if state.pear_idle_stopped.swap(true, Ordering::SeqCst) {
            return;
        }

        if let Ok(Some(song)) = pear::now_playing_request(&state.config).await {
            state
                .record_event(
                    "player",
                    format!(
                        "Pear livre sem fila do app; seguindo a seguir: {} - {}",
                        song.artist, song.title
                    ),
                )
                .await;
        }
        return;
    }

    if state.pear_idle_stopped.swap(true, Ordering::SeqCst) {
        return;
    }

    let pear_current = pear::now_playing(&state.config).await.ok();
    if pear_current.as_ref().is_some_and(|song| !song.is_paused) {
        if let Err(error) = pear::pause(&state.config).await {
            state
                .record_event("error", format!("Nao consegui pausar Pear ocioso: {error}"))
                .await;
        }
    }

    if let Err(error) = pear::clear_queue(&state.config).await {
        state
            .record_event(
                "error",
                format!("Nao consegui limpar fila interna do Pear ocioso: {error}"),
            )
            .await;
    }

    if let Some(song) = pear_current.and_then(pear::PearNowPlaying::display_name) {
        state
            .record_event(
                "player",
                format!("Pear pausado sem pedidos pendentes. Ultima musica: {song}"),
            )
            .await;
    } else {
        state
            .record_event("player", "Pear pausado sem pedidos pendentes")
            .await;
    }
}

async fn should_let_pear_continue_when_idle(_state: &AppState) -> bool {
    true
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{AppConfig, YoutubePlayback},
        song_requests::{RequestSource, SongRequest},
    };

    fn playback(
        is_playing: bool,
        progress_ms: Option<u64>,
        duration_ms: Option<u64>,
    ) -> spotify::SpotifyPlayback {
        spotify::SpotifyPlayback {
            title: "Manual Song - Artist".to_string(),
            is_playing,
            context_uri: None,
            progress_ms,
            duration_ms,
        }
    }

    #[test]
    fn manual_spotify_playback_paused_mid_song_is_not_finished() {
        assert!(!spotify_playback_finished(&playback(
            false,
            Some(30_000),
            Some(180_000)
        )));
    }

    #[test]
    fn manual_spotify_playback_at_end_is_finished() {
        assert!(spotify_playback_finished(&playback(
            false,
            Some(179_000),
            Some(180_000)
        )));
    }

    #[tokio::test]
    async fn spotify_mode_finds_spotify_request_behind_stale_youtube() {
        let mut config = AppConfig::from_env().expect("config");
        config.default_provider = MusicProvider::Spotify;
        let state = AppState::new(config);
        state.queue.write().await.clear();
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/UFFa0QoHWvE".to_string(),
            source: RequestSource::Youtube {
                video_id: "UFFa0QoHWvE".to_string(),
            },
            title: "Tank!".to_string(),
            artist: "SEATBELTS".to_string(),
        });
        let spotify = state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "metallica one".to_string(),
            source: RequestSource::Search {
                provider: MusicProvider::Spotify,
            },
            title: "One - Metallica".to_string(),
            artist: "Spotify".to_string(),
        });

        assert_eq!(first_pending_spotify(&state).await, Some(spotify));
        assert!(has_local_requests(&state).await);
    }

    #[tokio::test]
    async fn spotify_mode_does_not_pause_for_pending_youtube() {
        let mut config = AppConfig::from_env().expect("config");
        config.default_provider = MusicProvider::Spotify;
        config.youtube.playback = YoutubePlayback::Pear;
        let state = AppState::new(config);
        state.queue.write().await.clear();
        state.queue.write().await.add_resolved(SongRequest {
            id: 0,
            requester: "viewer".to_string(),
            query: "https://youtu.be/UFFa0QoHWvE".to_string(),
            source: RequestSource::Youtube {
                video_id: "UFFa0QoHWvE".to_string(),
            },
            title: "Tank!".to_string(),
            artist: "Seatbelts".to_string(),
        });

        coordinate_once(&state).await;

        assert!(!state.youtube_player_paused_spotify.load(Ordering::SeqCst));
        assert!(state.youtube_waiting_spotify_title.lock().await.is_none());
    }
}
