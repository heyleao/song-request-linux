use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::{
    collections::HashMap,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

use crate::{
    commands::{parse_chat_command, ChatCommand, ChatCommandInput, ChatUserRole, PlaybackAction},
    config::{self, TwitchBotSecrets, YoutubePlayback},
    display, request_flow,
    song_requests::{MusicProvider, QueueView, RequestSource, SongRequest, YoutubeRequestPlayback},
    state::AppState,
};

const TWITCH_IRC_WS: &str = "wss://irc-ws.chat.twitch.tv:443";

pub fn spawn_bot(state: AppState, secrets: TwitchBotSecrets) {
    if state
        .twitch_bot_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        info!("twitch bot already running");
        return;
    }

    tokio::spawn(async move {
        if let Err(error) = run_bot(state, secrets).await {
            error!(error = %error, "twitch bot stopped");
        }
    });
}

async fn run_bot(state: AppState, secrets: TwitchBotSecrets) -> Result<()> {
    let running = state.twitch_bot_running.clone();
    let result = run_bot_inner(&state, secrets).await;
    running.store(false, Ordering::SeqCst);
    result
}

async fn run_bot_inner(state: &AppState, secrets: TwitchBotSecrets) -> Result<()> {
    info!(
        channel = %secrets.channel,
        username = %secrets.username,
        "connecting twitch bot"
    );

    let (stream, _) = connect_async(TWITCH_IRC_WS)
        .await
        .context("failed to connect to Twitch IRC websocket")?;
    let (mut writer, mut reader) = stream.split();

    writer
        .send(Message::Text(
            "CAP REQ :twitch.tv/tags twitch.tv/commands".into(),
        ))
        .await?;
    writer
        .send(Message::Text(
            format!("PASS oauth:{}", secrets.oauth_token).into(),
        ))
        .await?;
    writer
        .send(Message::Text(format!("NICK {}", secrets.username).into()))
        .await?;
    writer
        .send(Message::Text(format!("JOIN #{}", secrets.channel).into()))
        .await?;

    info!(channel = %secrets.channel, "twitch bot connected");
    let mut rate_limiter = ChatRateLimiter::default();

    while let Some(message) = reader.next().await {
        let message = message.context("failed to read Twitch message")?;

        let Message::Text(text) = message else {
            continue;
        };

        for line in text.lines() {
            if line.starts_with("PING ") {
                let response = line.replacen("PING", "PONG", 1);
                writer.send(Message::Text(response.into())).await?;
                continue;
            }

            let Some(privmsg) = Privmsg::parse(line) else {
                continue;
            };

            let Some(reply) = handle_privmsg(state, privmsg, &mut rate_limiter).await else {
                continue;
            };

            let reply = format!(
                "PRIVMSG #{} :{}",
                secrets.channel,
                sanitize_irc_reply(&reply)
            );
            writer.send(Message::Text(reply.into())).await?;
        }
    }

    warn!("twitch websocket closed");
    Ok(())
}

async fn handle_privmsg(
    state: &AppState,
    privmsg: Privmsg,
    rate_limiter: &mut ChatRateLimiter,
) -> Option<String> {
    let settings = config::command_settings(&state.config.paths);
    let command = parse_chat_command(
        ChatCommandInput {
            requester: privmsg.sender.clone(),
            message: privmsg.message.clone(),
            is_moderator: privmsg.role >= ChatUserRole::Moderator,
            role: privmsg.role,
        },
        &settings,
    );
    if !matches!(command, ChatCommand::Ignored) {
        state
            .record_event(
                "chat",
                format!(
                    "{} ({:?}): {}",
                    privmsg.sender, privmsg.role, privmsg.message
                ),
            )
            .await;
    }

    if let Some(reply) = rate_limiter.check(&privmsg, &command) {
        state.record_event("chat", reply.clone()).await;
        return Some(reply);
    }

    match command {
        ChatCommand::SongRequest { input, role } => {
            let requester = input.requester.clone();
            match request_flow::add_request_for_role(state, input, role).await {
                Ok(request) => {
                    let display_title = display::chat_song_title(&request);
                    state
                        .record_event(
                            "request",
                            format!("{} pediu {}", request.requester, display_title),
                        )
                        .await;
                    Some(format!("@{requester} pedido adicionado: {display_title}"))
                }
                Err(error) => {
                    let message = format!("@{requester} nao consegui adicionar: {error}");
                    state.record_event("error", message.clone()).await;
                    Some(message)
                }
            }
        }
        ChatCommand::CurrentSong => Some(current_song_reply(state).await),
        ChatCommand::Queue => Some(queue_reply(state).await),
        ChatCommand::RemoveLast { requester } => {
            Some(remove_last_request_reply(state, requester).await)
        }
        ChatCommand::Skip { requester } => Some(skip_reply(state, requester).await),
        ChatCommand::Playback { requester, action } => {
            Some(playback_reply(state, requester, action).await)
        }
        ChatCommand::Volume { requester, level } => {
            Some(volume_reply(state, requester, level).await)
        }
        ChatCommand::Help => Some(format!(
            "Comandos: {}.",
            help_commands(&settings).join(", ")
        )),
        ChatCommand::AccessDenied {
            requester,
            command,
            required,
        } => {
            let message = access_denied_reply(requester, &command, required);
            state.record_event("access", message.clone()).await;
            Some(message)
        }
        ChatCommand::Ignored => None,
    }
}

#[derive(Default)]
struct ChatRateLimiter {
    last_command: HashMap<String, Instant>,
    last_song_request: HashMap<String, Instant>,
    last_remove: HashMap<String, Instant>,
}

impl ChatRateLimiter {
    fn check(&mut self, privmsg: &Privmsg, command: &ChatCommand) -> Option<String> {
        if privmsg.role >= ChatUserRole::Moderator
            || matches!(
                command,
                ChatCommand::Ignored | ChatCommand::AccessDenied { .. }
            )
        {
            return None;
        }

        let now = Instant::now();
        let user = privmsg.sender.to_ascii_lowercase();
        if Self::is_limited(&mut self.last_command, &user, now, Duration::from_secs(2)) {
            return Some(format!(
                "@{} aguarde alguns segundos antes de usar outro comando.",
                privmsg.sender
            ));
        }

        match command {
            ChatCommand::SongRequest { .. } => {
                if Self::is_limited(
                    &mut self.last_song_request,
                    &user,
                    now,
                    Duration::from_secs(10),
                ) {
                    return Some(format!(
                        "@{} aguarde alguns segundos antes de pedir outra musica.",
                        privmsg.sender
                    ));
                }
            }
            ChatCommand::RemoveLast { .. }
                if Self::is_limited(&mut self.last_remove, &user, now, Duration::from_secs(5)) =>
            {
                return Some(format!(
                    "@{} aguarde alguns segundos antes de remover de novo.",
                    privmsg.sender
                ));
            }
            _ => {}
        }

        None
    }

    fn is_limited(
        store: &mut HashMap<String, Instant>,
        user: &str,
        now: Instant,
        cooldown: Duration,
    ) -> bool {
        if store
            .get(user)
            .is_some_and(|last| now.duration_since(*last) < cooldown)
        {
            return true;
        }

        store.insert(user.to_string(), now);
        false
    }
}

fn sanitize_irc_reply(reply: &str) -> String {
    reply
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(430)
        .collect()
}

async fn remove_last_request_reply(state: &AppState, requester: String) -> String {
    let removed = {
        let mut queue = state.queue.write().await;
        queue.remove_last_by_requester(&requester)
    };
    save_queue_if_enabled(state).await;

    let message = match removed {
        Some(song) => {
            let suffix = if matches!(
                song.source,
                crate::song_requests::RequestSource::Spotify { .. }
            ) {
                " Se ela ja entrou na fila interna do Spotify, use skip quando chegar."
            } else {
                ""
            };
            let title = display::chat_song_title(&song);
            format!("@{requester} removi seu ultimo pedido: {title}.{suffix}")
        }
        None => format!("@{requester} nao encontrei pedido seu pendente para remover."),
    };
    state.record_event("request", message.clone()).await;
    message
}

async fn skip_reply(state: &AppState, requester: String) -> String {
    if is_youtube_browser_mode(state) {
        let message = local_browser_skip_reply(state, &requester).await;
        state.record_event("player", message.clone()).await;
        return message;
    }

    if matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(current_youtube_playback(state), YoutubePlayback::Pear)
    {
        let message = pear_skip_reply(state, &requester).await;
        state.record_event("player", message.clone()).await;
        return message;
    }

    if let Some(message) =
        spotify_playback_reply(state, requester.clone(), PlaybackAction::Next).await
    {
        state.record_event("player", message.clone()).await;
        return message;
    }

    let current_song = state.queue.write().await.skip();
    save_queue_if_enabled(state).await;
    let message = match current_song {
        Some(song) => format!(
            "@{requester} skip feito. Agora: {}",
            display::chat_song_title(&song)
        ),
        None => format!("@{requester} skip feito. Fila vazia."),
    };
    state.record_event("player", message.clone()).await;
    message
}

async fn pear_skip_reply(state: &AppState, requester: &str) -> String {
    match crate::pear::skip_next(&state.config).await {
        Ok(outcome) if outcome.changed => {
            let suffix = if outcome.fallback_used {
                " usando fila interna"
            } else {
                ""
            };
            match outcome.after {
                Some(after) => format!("@{requester} pulei no Pear{suffix}. Agora: {after}."),
                None => format!("@{requester} pulei no Pear{suffix}."),
            }
        }
        Ok(outcome) => {
            let current = outcome
                .after
                .or(outcome.before)
                .map(|song| format!(" Musica atual: {song}."))
                .unwrap_or_default();
            format!("@{requester} enviei skip ao Pear, mas a musica nao mudou.{current}")
        }
        Err(error) => format!("@{requester} nao consegui pular no Pear: {error}"),
    }
}

fn help_commands(settings: &crate::commands::CommandSettings) -> Vec<String> {
    let aliases = &settings.aliases;
    vec![
        format!("{} nome/link", aliases.song_request[0]),
        aliases.current_song[0].clone(),
        aliases.queue[0].clone(),
        aliases.remove[0].clone(),
        aliases.volume[0].clone(),
        format!("{} 30 mod", aliases.volume[0]),
        format!("{} mod", aliases.play[0]),
        format!("{} mod", aliases.pause[0]),
        format!("{} mod", aliases.skip[0]),
    ]
}

async fn playback_reply(state: &AppState, requester: String, action: PlaybackAction) -> String {
    if is_youtube_browser_mode(state) {
        let message = match action {
            PlaybackAction::Next => local_browser_skip_reply(state, &requester).await,
            PlaybackAction::Play | PlaybackAction::Pause => {
                local_browser_playback_reply(state, &requester, action)
            }
        };
        state.record_event("player", message.clone()).await;
        return message;
    }

    if matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(current_youtube_playback(state), YoutubePlayback::Pear)
    {
        let message = match action {
            PlaybackAction::Play => match crate::pear::play(&state.config).await {
                Ok(()) => format!("@{requester} Pear retomado."),
                Err(error) => format!("@{requester} nao consegui controlar o Pear: {error}"),
            },
            PlaybackAction::Pause => match crate::pear::pause(&state.config).await {
                Ok(()) => format!("@{requester} Pear pausado."),
                Err(error) => format!("@{requester} nao consegui controlar o Pear: {error}"),
            },
            PlaybackAction::Next => pear_skip_reply(state, &requester).await,
        };
        state.record_event("player", message.clone()).await;
        return message;
    }

    let message = spotify_playback_reply(state, requester, action)
        .await
        .unwrap_or_else(|| "Spotify nao conectado.".to_string());
    state.record_event("player", message.clone()).await;
    message
}

async fn spotify_playback_reply(
    state: &AppState,
    requester: String,
    action: PlaybackAction,
) -> Option<String> {
    let mut token_guard = state.spotify_token.write().await;
    let token = token_guard.as_mut()?;

    let result = match action {
        PlaybackAction::Play => crate::spotify::resume_playback(&state.config, token).await,
        PlaybackAction::Pause => crate::spotify::pause_playback(&state.config, token).await,
        PlaybackAction::Next => crate::spotify::skip_next(&state.config, token).await,
    };

    Some(match result {
        Ok(()) => match action {
            PlaybackAction::Play => format!("@{requester} playback retomado."),
            PlaybackAction::Pause => format!("@{requester} playback pausado."),
            PlaybackAction::Next => format!("@{requester} pulei para a proxima."),
        },
        Err(error) => format!("@{requester} nao consegui controlar o Spotify: {error}"),
    })
}

async fn current_song_reply(state: &AppState) -> String {
    match current_provider(state) {
        MusicProvider::Spotify => {
            if let Some(snapshot) = spotify_queue_snapshot(state).await {
                if let Some(song) = snapshot.currently_playing {
                    return format!("Tocando agora: {song}");
                }
            }
        }
        MusicProvider::Youtube
            if matches!(current_youtube_playback(state), YoutubePlayback::Pear) =>
        {
            if let Ok(Some(song)) = crate::pear::now_playing_request(&state.config).await {
                return format!(
                    "Tocando agora: {} - pedido por {}",
                    display::chat_song_title(&song),
                    song.requester
                );
            }
        }
        MusicProvider::Youtube => {}
    }

    let queue = active_app_queue_view(state).await;
    match queue.current_song {
        Some(song) => format!(
            "Tocando agora: {} - pedido por {}",
            display::chat_song_title(&song),
            song.requester
        ),
        None => "Nenhuma musica tocando agora.".to_string(),
    }
}

async fn queue_reply(state: &AppState) -> String {
    if matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(current_youtube_playback(state), YoutubePlayback::Pear)
    {
        return pear_queue_reply(state).await;
    }

    if matches!(current_provider(state), MusicProvider::Spotify) {
        if let Some(snapshot) = spotify_queue_snapshot(state).await {
            if snapshot.upcoming.is_empty() {
                return snapshot
                    .currently_playing
                    .map(|song| format!("Tocando agora: {song}. Fila vazia."))
                    .unwrap_or_else(|| "Fila vazia.".to_string());
            }

            return format!("Proximas: {}", snapshot.upcoming.join(" | "));
        }
    }

    app_queue_reply(state).await
}

async fn pear_queue_reply(state: &AppState) -> String {
    match crate::pear::queue_display_items(&state.config).await {
        Ok(items) if !items.is_empty() => {
            let selected = items.iter().position(|item| item.selected);
            let current = selected
                .and_then(|index| items.get(index))
                .map(pear_queue_item_label);
            let start = selected.map_or(0, |index| index.saturating_add(1));
            let upcoming = items
                .iter()
                .skip(start)
                .take(5)
                .map(pear_queue_item_label)
                .collect::<Vec<_>>();

            match (current, upcoming.is_empty()) {
                (Some(current), false) => {
                    format!(
                        "Pear tocando: {current}. Proximas: {}",
                        upcoming.join(" | ")
                    )
                }
                (Some(current), true) => format!("Pear tocando: {current}. Sem proximas no Pear."),
                (None, false) => format!("Pear proximas: {}", upcoming.join(" | ")),
                (None, true) => app_queue_reply(state).await,
            }
        }
        Ok(_) => app_queue_reply(state).await,
        Err(error) => format!("Nao consegui ler a fila do Pear: {error}"),
    }
}

fn pear_queue_item_label(item: &crate::pear::PearQueueDisplayItem) -> String {
    match &item.artist {
        Some(artist) if !artist.trim().is_empty() => format!("{} - {}", artist, item.title),
        _ => item.title.clone(),
    }
}

async fn app_queue_reply(state: &AppState) -> String {
    let queue = active_app_queue_view(state).await;
    let current = queue.current_song.map(|song| {
        format!(
            "Tocando agora: {} por {}",
            display::chat_song_title(&song),
            song.requester
        )
    });
    let upcoming = queue
        .queue
        .into_iter()
        .take(5)
        .map(|song| format!("{} por {}", display::chat_song_title(&song), song.requester))
        .collect::<Vec<_>>();

    match (current, upcoming.is_empty()) {
        (Some(current), false) => format!("{current}. Proximas: {}", upcoming.join(" | ")),
        (Some(current), true) => format!("{current}. Fila vazia."),
        (None, false) => format!("Proximas: {}", upcoming.join(" | ")),
        (None, true) => "Fila vazia.".to_string(),
    }
}

async fn volume_reply(state: &AppState, requester: String, level: Option<u8>) -> String {
    match level {
        Some(level) => {
            let level = level.clamp(1, 100);
            let mut changed = Vec::new();
            let mut errors = Vec::new();

            match (current_provider(state), current_youtube_playback(state)) {
                (MusicProvider::Youtube, YoutubePlayback::Pear) => {
                    match crate::pear::set_volume(&state.config, level).await {
                        Ok(level) => {
                            persist_volume_setting(
                                state,
                                MusicProvider::Youtube,
                                YoutubePlayback::Pear,
                                level,
                            )
                            .await;
                            changed.push(format!("Pear/YouTube {level}%"));
                        }
                        Err(error) => errors.push(format!("Pear/YouTube: {error}")),
                    }
                }
                (MusicProvider::Youtube, YoutubePlayback::Browser) => {
                    let level = set_browser_volume(state, level).await;
                    changed.push(format!("OBS Browser {level}%"));
                }
                (MusicProvider::Spotify, _) => match set_spotify_volume(state, level).await {
                    Some(Ok(level)) => {
                        persist_volume_setting(
                            state,
                            MusicProvider::Spotify,
                            YoutubePlayback::Browser,
                            level,
                        )
                        .await;
                        changed.push(format!("Spotify {level}%"));
                    }
                    Some(Err(error)) => errors.push(format!("Spotify: {error}")),
                    None => errors.push("Spotify nao conectado".to_string()),
                },
            }

            if !changed.is_empty() {
                let message = format!("@{requester} volume ajustado: {}.", changed.join(", "));
                state.record_event("volume", message.clone()).await;
                if !errors.is_empty() {
                    state
                        .record_event(
                            "volume",
                            format!(
                                "Volume parcial: {}. Falhas: {}",
                                changed.join(", "),
                                errors.join(" | ")
                            ),
                        )
                        .await;
                }
                message
            } else {
                let message = format!(
                    "@{requester} nao consegui mudar volume: {}",
                    errors.join(" | ")
                );
                state.record_event("error", message.clone()).await;
                message
            }
        }
        None => current_volume_reply(state).await,
    }
}

async fn set_spotify_volume(state: &AppState, level: u8) -> Option<Result<u8>> {
    let mut token_guard = state.spotify_token.write().await;
    let token = token_guard.as_mut()?;
    Some(crate::spotify::set_volume(&state.config, token, level).await)
}

async fn set_browser_volume(state: &AppState, level: u8) -> u8 {
    let level = level.clamp(1, 100);
    state.youtube_browser_volume.store(level, Ordering::SeqCst);
    persist_volume_setting(
        state,
        MusicProvider::Youtube,
        YoutubePlayback::Browser,
        level,
    )
    .await;
    level
}

async fn persist_volume_setting(
    state: &AppState,
    provider: MusicProvider,
    playback: YoutubePlayback,
    level: u8,
) {
    if let Err(error) =
        config::update_volume_setting(&state.config.paths, provider, playback, level)
    {
        state
            .record_event("error", format!("Nao consegui salvar volume: {error}"))
            .await;
    }
}

async fn current_volume_reply(state: &AppState) -> String {
    if matches!(
        (current_provider(state), current_youtube_playback(state)),
        (MusicProvider::Youtube, YoutubePlayback::Pear)
    ) {
        return match crate::pear::current_volume(&state.config).await {
            Ok(volume) => {
                let suffix = if volume.is_muted { " (mutado)" } else { "" };
                format!("Volume atual Pear/YouTube: {}%{suffix}.", volume.state)
            }
            Err(error) => format!("Nao consegui ler o volume Pear/YouTube: {error}"),
        };
    }

    if is_youtube_browser_mode(state) {
        let level = state
            .youtube_browser_volume
            .load(Ordering::SeqCst)
            .clamp(1, 100);
        return format!(
            "Volume atual OBS Browser: {level}%. O fader do OBS continua separado no mixer."
        );
    }

    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return "Spotify nao conectado.".to_string();
    };

    match crate::spotify::current_volume(&state.config, token).await {
        Ok(Some(level)) => format!("Volume atual Spotify: {level}%."),
        Ok(None) => "Nao encontrei um device Spotify ativo para ler o volume.".to_string(),
        Err(error) => format!("Nao consegui ler o volume Spotify: {error}"),
    }
}

async fn active_app_queue_view(state: &AppState) -> QueueView {
    match current_provider(state) {
        MusicProvider::Spotify => {
            filtered_app_queue_view(state, MusicProvider::Spotify, None).await
        }
        MusicProvider::Youtube => {
            let playback = match current_youtube_playback(state) {
                YoutubePlayback::Pear => YoutubeRequestPlayback::Pear,
                YoutubePlayback::Browser => YoutubeRequestPlayback::Browser,
            };
            filtered_app_queue_view(state, MusicProvider::Youtube, Some(playback)).await
        }
    }
}

async fn filtered_app_queue_view(
    state: &AppState,
    provider: MusicProvider,
    playback: Option<YoutubeRequestPlayback>,
) -> QueueView {
    let view = state.queue.read().await.view();
    let mut requests = Vec::new();
    if let Some(song) = view.current_song {
        requests.push(song);
    }
    requests.extend(view.queue);
    let mut requests = requests
        .into_iter()
        .filter(|song| request_matches_stream(song, provider, playback))
        .collect::<Vec<_>>();
    let current_song = requests.first().cloned();
    if current_song.is_some() {
        requests.remove(0);
    }
    QueueView {
        current_song,
        queue_length: requests.len(),
        queue: requests,
        persistence: None,
    }
}

fn request_matches_stream(
    song: &SongRequest,
    provider: MusicProvider,
    playback: Option<YoutubeRequestPlayback>,
) -> bool {
    match provider {
        MusicProvider::Spotify => matches!(
            song.source,
            RequestSource::Spotify { .. }
                | RequestSource::Search {
                    provider: MusicProvider::Spotify
                }
        ),
        MusicProvider::Youtube => match (&song.source, playback) {
            (
                RequestSource::Youtube {
                    playback: Some(source_playback),
                    ..
                },
                Some(active_playback),
            ) => *source_playback == active_playback,
            _ => false,
        },
    }
}

fn current_provider(state: &AppState) -> MusicProvider {
    config::UiConfigView::load(&state.config.paths).default_provider
}

fn current_youtube_playback(state: &AppState) -> YoutubePlayback {
    config::UiConfigView::load(&state.config.paths).youtube_playback
}

fn is_youtube_browser_mode(state: &AppState) -> bool {
    matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(current_youtube_playback(state), YoutubePlayback::Browser)
}

async fn save_queue_if_enabled(state: &AppState) {
    if !config::queue_persistence_enabled(&state.config.paths) {
        return;
    }

    if let Err(error) = state
        .queue
        .read()
        .await
        .save(&state.config.paths.queue_file)
    {
        state.record_event("error", error.to_string()).await;
    }
}

async fn local_browser_skip_reply(state: &AppState, requester: &str) -> String {
    state.youtube_browser_paused.store(false, Ordering::SeqCst);
    let current_id = filtered_app_queue_view(
        state,
        MusicProvider::Youtube,
        Some(YoutubeRequestPlayback::Browser),
    )
    .await
    .current_song
    .map(|song| song.id);

    let current_song = if let Some(id) = current_id {
        let mut queue = state.queue.write().await;
        queue.remove_by_id(id);
        drop(queue);
        save_queue_if_enabled(state).await;
        filtered_app_queue_view(
            state,
            MusicProvider::Youtube,
            Some(YoutubeRequestPlayback::Browser),
        )
        .await
        .current_song
    } else {
        None
    };

    match current_song {
        Some(song) => format!(
            "@{requester} skip feito. Agora: {}",
            display::chat_song_title(&song)
        ),
        None => format!("@{requester} skip feito. Fila vazia."),
    }
}

fn local_browser_playback_reply(
    state: &AppState,
    requester: &str,
    action: PlaybackAction,
) -> String {
    match action {
        PlaybackAction::Play => {
            state.youtube_browser_paused.store(false, Ordering::SeqCst);
            format!("@{requester} player OBS retomado.")
        }
        PlaybackAction::Pause => {
            state.youtube_browser_paused.store(true, Ordering::SeqCst);
            format!("@{requester} player OBS pausado.")
        }
        PlaybackAction::Next => {
            state.youtube_browser_paused.store(false, Ordering::SeqCst);
            format!("@{requester} pulando player OBS.")
        }
    }
}

fn access_denied_reply(
    requester: String,
    command: &str,
    required: crate::commands::CommandAccess,
) -> String {
    match required {
        crate::commands::CommandAccess::Follower => {
            format!("@{requester} {command} precisa seguir o canal.")
        }
        crate::commands::CommandAccess::Subscriber => {
            format!("@{requester} {command} precisa de sub, VIP, moderador ou streamer.")
        }
        crate::commands::CommandAccess::Vip => {
            format!("@{requester} {command} precisa de VIP, moderador ou streamer.")
        }
        crate::commands::CommandAccess::Moderator => {
            format!("@{requester} {command} precisa de moderador ou streamer.")
        }
        crate::commands::CommandAccess::Streamer => {
            format!("@{requester} {command} precisa do streamer.")
        }
    }
}

async fn spotify_queue_snapshot(state: &AppState) -> Option<crate::spotify::SpotifyQueueSnapshot> {
    let mut token_guard = state.spotify_token.write().await;
    let token = token_guard.as_mut()?;

    crate::spotify::queue_snapshot(&state.config, token)
        .await
        .ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Privmsg {
    sender: String,
    message: String,
    role: ChatUserRole,
}

impl Privmsg {
    fn parse(line: &str) -> Option<Self> {
        if !line.contains(" PRIVMSG ") {
            return None;
        }

        let (tags, rest) = if let Some(stripped) = line.strip_prefix('@') {
            let (tags, rest) = stripped.split_once(' ')?;
            (Some(tags), rest)
        } else {
            (None, line)
        };

        let sender = parse_sender(rest)?;
        let message = rest.split_once(" :")?.1.to_string();
        let role = ChatUserRole::from_twitch_tags(tags);

        Some(Self {
            sender,
            message,
            role,
        })
    }
}

fn parse_sender(rest: &str) -> Option<String> {
    let sender_part = rest.strip_prefix(':')?.split_once('!')?.0;

    if sender_part.is_empty() {
        None
    } else {
        Some(sender_part.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn app_queue_reply_shows_current_and_upcoming_requests() {
        let mut config = crate::config::AppConfig::from_env().expect("config");
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("song-request-linux-twitch-{unique}"));
        config.paths.config_dir = root.join("config");
        config.paths.state_dir = root.join("state");
        config.paths.queue_file = config.paths.state_dir.join("queue.json");
        crate::config::save_ui_config(
            &config.paths,
            crate::config::UiConfigInput {
                default_provider: Some(MusicProvider::Youtube),
                youtube_playback: Some(YoutubePlayback::Browser),
                pear_base_url: None,
                spotify_client_id: None,
                spotify_fallback_enabled: Some(false),
                queue_persistence_enabled: Some(false),
                twitch_client_id: None,
                twitch_bot_username: None,
                twitch_channel: None,
                twitch_bot_oauth_token: None,
                youtube_api_key: None,
                youtube_max_duration_seconds: Some(360),
                youtube_allow_non_music: Some(false),
                command_settings: None,
                queue_limits: Some(crate::config::QueueLimitConfig::default()),
                overlay: None,
            },
        )
        .expect("save config");
        let state = AppState::new(config);
        state.queue.write().await.clear();
        state
            .queue
            .write()
            .await
            .add_resolved(crate::song_requests::SongRequest {
                id: 0,
                requester: "viewer".to_string(),
                query: "https://youtu.be/current".to_string(),
                source: crate::song_requests::RequestSource::Youtube {
                    video_id: "current".to_string(),
                    playback: Some(crate::song_requests::YoutubeRequestPlayback::Browser),
                },
                title: "Current Song".to_string(),
                artist: "Current Artist".to_string(),
            });
        state
            .queue
            .write()
            .await
            .add_resolved(crate::song_requests::SongRequest {
                id: 0,
                requester: "mod".to_string(),
                query: "https://youtu.be/next".to_string(),
                source: crate::song_requests::RequestSource::Youtube {
                    video_id: "next".to_string(),
                    playback: Some(crate::song_requests::YoutubeRequestPlayback::Browser),
                },
                title: "Next Song".to_string(),
                artist: "Next Artist".to_string(),
            });

        let reply = app_queue_reply(&state).await;

        assert!(reply.contains("Tocando agora: Current Artist - Current Song por viewer"));
        assert!(reply.contains("Proximas: Next Artist - Next Song por mod"));
    }

    #[test]
    fn parses_privmsg_with_tags() {
        let line = "@badge-info=;badges=moderator/1;color=#fff;mod=1 :viewer!viewer@viewer.tmi.twitch.tv PRIVMSG #heyleao :!sr one more time";
        let message = Privmsg::parse(line).expect("privmsg");

        assert_eq!(message.sender, "viewer");
        assert_eq!(message.message, "!sr one more time");
        assert_eq!(message.role, ChatUserRole::Moderator);
    }

    #[test]
    fn parses_privmsg_with_subscriber_tag() {
        let line = "@badge-info=subscriber/12;badges=subscriber/12;color=#fff;mod=0;subscriber=1 :viewer!viewer@viewer.tmi.twitch.tv PRIVMSG #heyleao :!sr spiders";
        let message = Privmsg::parse(line).expect("privmsg");

        assert_eq!(message.role, ChatUserRole::Subscriber);
    }

    #[test]
    fn parses_plain_privmsg_as_follower() {
        let line = "@badge-info=;badges=;color=#fff;mod=0 :viewer!viewer@viewer.tmi.twitch.tv PRIVMSG #heyleao :!sr spiders";
        let message = Privmsg::parse(line).expect("privmsg");

        assert_eq!(message.role, ChatUserRole::Follower);
    }

    #[test]
    fn parses_privmsg_with_vip_badge() {
        let line = "@badge-info=;badges=vip/1;color=#fff;mod=0 :viewer!viewer@viewer.tmi.twitch.tv PRIVMSG #heyleao :!sr one more time";
        let message = Privmsg::parse(line).expect("privmsg");

        assert_eq!(message.role, ChatUserRole::Vip);
    }

    #[test]
    fn parses_privmsg_with_broadcaster_badge() {
        let line = "@badge-info=;badges=broadcaster/1;color=#fff;mod=0 :heyleao!heyleao@heyleao.tmi.twitch.tv PRIVMSG #heyleao :!skip";
        let message = Privmsg::parse(line).expect("privmsg");

        assert_eq!(message.role, ChatUserRole::Streamer);
    }

    #[test]
    fn ignores_non_privmsg() {
        assert!(Privmsg::parse(":tmi.twitch.tv 001 bot :Welcome").is_none());
    }

    #[test]
    fn sanitizes_irc_reply_controls_and_length() {
        let reply = sanitize_irc_reply("@viewer ok\r\nPRIVMSG #x :owned\u{0007}");

        assert!(!reply.contains('\r'));
        assert!(!reply.contains('\n'));
        assert!(!reply.contains('\u{0007}'));
        assert!(reply.len() <= 430);
    }
}
