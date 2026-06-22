use std::{sync::atomic::Ordering, time::Duration};

use tokio::time::interval;

use crate::{song_requests::RequestSource, spotify, state::AppState};

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
    if !has_pending_youtube(state).await {
        *state.youtube_waiting_spotify_title.lock().await = None;
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
        *state.youtube_waiting_spotify_title.lock().await = Some(title.clone());
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

async fn has_pending_youtube(state: &AppState) -> bool {
    state
        .queue
        .read()
        .await
        .first_youtube()
        .is_some_and(|song| matches!(song.source, RequestSource::Youtube { .. }))
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
            if error.to_string().contains("Restriction violated") {
                state
                    .youtube_player_paused_spotify
                    .store(true, Ordering::SeqCst);
                state
                    .record_event(
                        "player",
                        "Spotify recusou pausa por restricao; liberando pedido YouTube",
                    )
                    .await;
                return;
            }

            state
                .record_event("error", format!("Nao consegui pausar Spotify: {error}"))
                .await;
        }
    }
}
