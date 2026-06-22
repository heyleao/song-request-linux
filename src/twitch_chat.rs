use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::Ordering;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

use crate::{
    commands::{parse_chat_command, ChatCommand, ChatCommandInput},
    config::TwitchBotSecrets,
    request_flow,
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

            let Some(reply) = handle_privmsg(state, privmsg).await else {
                continue;
            };

            let reply = format!("PRIVMSG #{} :{}", secrets.channel, reply);
            writer.send(Message::Text(reply.into())).await?;
        }
    }

    warn!("twitch websocket closed");
    Ok(())
}

async fn handle_privmsg(state: &AppState, privmsg: Privmsg) -> Option<String> {
    let command = parse_chat_command(ChatCommandInput {
        requester: privmsg.sender,
        message: privmsg.message,
        is_moderator: privmsg.is_moderator,
    });

    match command {
        ChatCommand::SongRequest(input) => {
            let requester = input.requester.clone();
            match request_flow::add_request(state, input).await {
                Ok(request) => Some(format!("@{requester} pedido adicionado: {}", request.title)),
                Err(error) => Some(format!("@{requester} nao consegui adicionar: {error}")),
            }
        }
        ChatCommand::CurrentSong => Some(current_song_reply(state).await),
        ChatCommand::Queue => Some(queue_reply(state).await),
        ChatCommand::Skip { requester } => {
            let current_song = state.queue.write().await.skip();
            match current_song {
                Some(song) => Some(format!("@{requester} skip feito. Agora: {}", song.title)),
                None => Some(format!("@{requester} skip feito. Fila vazia.")),
            }
        }
        ChatCommand::Volume {
            requester,
            level,
            can_set,
        } => Some(volume_reply(state, requester, level, can_set).await),
        ChatCommand::Help => {
            Some("Comandos: !sr nome/link, !song, !fila, !vol, !vol 30 mod, !skip mod.".to_string())
        }
        ChatCommand::Ignored => None,
    }
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
            song.title, song.requester
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
        .map(|song| format!("{} por {}", song.title, song.requester))
        .collect::<Vec<_>>();
    format!("Proximas: {}", upcoming.join(" | "))
}

async fn volume_reply(
    state: &AppState,
    requester: String,
    level: Option<u8>,
    can_set: bool,
) -> String {
    let mut token_guard = state.spotify_token.write().await;
    let Some(token) = token_guard.as_mut() else {
        return "Spotify nao conectado.".to_string();
    };

    match level {
        Some(_) if !can_set => format!("@{requester} apenas moderadores podem mudar volume."),
        Some(level) => match crate::spotify::set_volume(&state.config, token, level).await {
            Ok(level) => format!("@{requester} volume ajustado para {level}%."),
            Err(error) => format!("@{requester} nao consegui mudar volume: {error}"),
        },
        None => match crate::spotify::current_volume(&state.config, token).await {
            Ok(Some(level)) => format!("Volume atual: {level}%."),
            Ok(None) => "Nao encontrei um device Spotify ativo para ler o volume.".to_string(),
            Err(error) => format!("Nao consegui ler o volume: {error}"),
        },
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
    is_moderator: bool,
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
        let is_moderator = tags.is_some_and(tags_include_moderator);

        Some(Self {
            sender,
            message,
            is_moderator,
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

fn tags_include_moderator(tags: &str) -> bool {
    tags.split(';').any(|tag| {
        tag == "mod=1"
            || tag.strip_prefix("badges=").is_some_and(|badges| {
                badges.contains("broadcaster/") || badges.contains("moderator/")
            })
    })
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
        assert!(message.is_moderator);
    }

    #[test]
    fn ignores_non_privmsg() {
        assert!(Privmsg::parse(":tmi.twitch.tv 001 bot :Welcome").is_none());
    }
}
