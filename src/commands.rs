use serde::{Deserialize, Serialize};

use crate::song_requests::SongRequestInput;

#[derive(Clone, Debug, Deserialize)]
pub struct ChatCommandInput {
    pub requester: String,
    pub message: String,
    #[serde(default)]
    pub is_moderator: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatCommand {
    SongRequest(SongRequestInput),
    CurrentSong,
    Skip { requester: String },
    Ignored,
}

pub fn parse_chat_command(input: ChatCommandInput) -> ChatCommand {
    let message = input.message.trim();

    if let Some(query) = command_payload(message, "!sr") {
        if query.is_empty() {
            return ChatCommand::Ignored;
        }

        return ChatCommand::SongRequest(SongRequestInput {
            requester: clean_field(&input.requester),
            query: clean_field(query),
        });
    }

    if message.eq_ignore_ascii_case("!song") {
        return ChatCommand::CurrentSong;
    }

    if message.eq_ignore_ascii_case("!skip") && input.is_moderator {
        return ChatCommand::Skip {
            requester: clean_field(&input.requester),
        };
    }

    ChatCommand::Ignored
}

fn command_payload<'a>(message: &'a str, command: &str) -> Option<&'a str> {
    let (head, tail) = message.split_once(char::is_whitespace)?;

    if head.eq_ignore_ascii_case(command) {
        Some(tail.trim())
    } else {
        None
    }
}

fn clean_field(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>()
        .trim()
        .chars()
        .take(240)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_song_request() {
        let command = parse_chat_command(ChatCommandInput {
            requester: "bruno".to_string(),
            message: "!sr daft punk one more time".to_string(),
            is_moderator: false,
        });

        assert_eq!(
            command,
            ChatCommand::SongRequest(SongRequestInput {
                requester: "bruno".to_string(),
                query: "daft punk one more time".to_string()
            })
        );
    }

    #[test]
    fn skip_requires_moderator() {
        let command = parse_chat_command(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!skip".to_string(),
            is_moderator: false,
        });

        assert_eq!(command, ChatCommand::Ignored);
    }
}
