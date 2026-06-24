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
    song_requests::MusicProvider,
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
    if config::queue_persistence_enabled(&state.config.paths) {
        if let Err(error) = state
            .queue
            .read()
            .await
            .save(&state.config.paths.queue_file)
        {
            state.record_event("error", error.to_string()).await;
        }
    }

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
    if matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(state.config.youtube.playback, YoutubePlayback::Pear)
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
    if matches!(current_provider(state), MusicProvider::Youtube)
        && matches!(state.config.youtube.playback, YoutubePlayback::Pear)
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
    if let Some(snapshot) = spotify_queue_snapshot(state).await {
        if let Some(song) = snapshot.currently_playing {
            return format!("Tocando agora: {song}");
        }
    }

    let queue = state.queue.read().await.view();
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
    if let Some(snapshot) = spotify_queue_snapshot(state).await {
        if snapshot.upcoming.is_empty() {
            return snapshot
                .currently_playing
                .map(|song| format!("Tocando agora: {song}. Fila vazia."))
                .unwrap_or_else(|| "Fila vazia.".to_string());
        }

        return format!("Proximas: {}", snapshot.upcoming.join(" | "));
    }

    let queue = state.queue.read().await.view();
    if queue.queue.is_empty() {
        return "Fila vazia.".to_string();
    }

    let upcoming = queue
        .queue
        .into_iter()
        .take(5)
        .map(|song| format!("{} por {}", display::chat_song_title(&song), song.requester))
        .collect::<Vec<_>>();
    format!("Proximas: {}", upcoming.join(" | "))
}

async fn volume_reply(state: &AppState, requester: String, level: Option<u8>) -> String {
    match level {
        Some(level) => {
            let level = level.clamp(1, 100);
            let mut changed = Vec::new();
            let mut errors = Vec::new();

            match set_spotify_volume(state, level).await {
                Some(Ok(level)) => changed.push(format!("Spotify {level}%")),
                Some(Err(error)) => errors.push(format!("Spotify: {error}")),
                None => errors.push("Spotify nao conectado".to_string()),
            }

            if matches!(state.config.youtube.playback, YoutubePlayback::Pear) {
                match crate::pear::set_volume(&state.config, level).await {
                    Ok(level) => changed.push(format!("Pear/YouTube {level}%")),
                    Err(error) => errors.push(format!("Pear/YouTube: {error}")),
                }
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

async fn current_volume_reply(state: &AppState) -> String {
    if matches!(
        (current_provider(state), state.config.youtube.playback),
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

fn current_provider(state: &AppState) -> MusicProvider {
    config::UiConfigView::load(&state.config.paths).default_provider
}

fn access_denied_reply(
    requester: String,
    command: &str,
    required: crate::commands::CommandAccess,
) -> String {
    match required {
        crate::commands::CommandAccess::Everyone => {
            format!("@{requester} {command} esta liberado para todos.")
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

    #[test]
    fn parses_privmsg_with_tags() {
        let line = "@badge-info=;badges=moderator/1;color=#fff;mod=1 :viewer!viewer@viewer.tmi.twitch.tv PRIVMSG #heyleao :!sr one more time";
        let message = Privmsg::parse(line).expect("privmsg");

        assert_eq!(message.sender, "viewer");
        assert_eq!(message.message, "!sr one more time");
        assert_eq!(message.role, ChatUserRole::Moderator);
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
