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
    Queue,
    Skip {
        requester: String,
    },
    Volume {
        requester: String,
        level: Option<u8>,
    },
    Help,
    AccessDenied {
        requester: String,
        command: String,
        required: CommandAccess,
    },
    Ignored,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandAccess {
    Moderator,
}

pub fn parse_chat_command(input: ChatCommandInput) -> ChatCommand {
    let message = input.message.trim();
    let requester = clean_field(&input.requester);

    if let Some(query) = command_payload(message, "!sr") {
        if query.is_empty() {
            return ChatCommand::Ignored;
        }

        return ChatCommand::SongRequest(SongRequestInput {
            requester,
            query: clean_field(query),
        });
    }

    if message.eq_ignore_ascii_case("!song") {
        return ChatCommand::CurrentSong;
    }

    if matches_command(message, &["!queue", "!fila", "!q"]) {
        return ChatCommand::Queue;
    }

    if message.eq_ignore_ascii_case("!skip") {
        if !input.is_moderator {
            return ChatCommand::AccessDenied {
                requester,
                command: "!skip".to_string(),
                required: CommandAccess::Moderator,
            };
        }

        return ChatCommand::Skip { requester };
    }

    if let Some(level) = volume_payload(message) {
        if level.is_some() && !input.is_moderator {
            return ChatCommand::AccessDenied {
                requester,
                command: "!vol <0-100>".to_string(),
                required: CommandAccess::Moderator,
            };
        }

        return ChatCommand::Volume { requester, level };
    }

    if matches_command(message, &["!commands", "!comandos", "!help"]) {
        return ChatCommand::Help;
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

fn matches_command(message: &str, commands: &[&str]) -> bool {
    commands
        .iter()
        .any(|command| message.eq_ignore_ascii_case(command))
}

fn volume_payload(message: &str) -> Option<Option<u8>> {
    if matches_command(message, &["!vol", "!volume"]) {
        return Some(None);
    }

    let payload = command_payload(message, "!vol")
        .or_else(|| command_payload(message, "!volume"))?
        .trim();
    let level = payload.parse::<u8>().ok()?.min(100);

    Some(Some(level))
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

        assert_eq!(
            command,
            ChatCommand::AccessDenied {
                requester: "viewer".to_string(),
                command: "!skip".to_string(),
                required: CommandAccess::Moderator
            }
        );
    }

    #[test]
    fn parses_queue_aliases() {
        let command = parse_chat_command(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!fila".to_string(),
            is_moderator: false,
        });

        assert_eq!(command, ChatCommand::Queue);
    }

    #[test]
    fn parses_volume_read_and_write() {
        let read = parse_chat_command(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!vol".to_string(),
            is_moderator: false,
        });
        let write = parse_chat_command(ChatCommandInput {
            requester: "mod".to_string(),
            message: "!vol 35".to_string(),
            is_moderator: true,
        });

        assert_eq!(
            read,
            ChatCommand::Volume {
                requester: "viewer".to_string(),
                level: None,
            }
        );
        assert_eq!(
            write,
            ChatCommand::Volume {
                requester: "mod".to_string(),
                level: Some(35),
            }
        );
    }

    #[test]
    fn volume_set_requires_moderator() {
        let command = parse_chat_command(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!vol 35".to_string(),
            is_moderator: false,
        });

        assert_eq!(
            command,
            ChatCommand::AccessDenied {
                requester: "viewer".to_string(),
                command: "!vol <0-100>".to_string(),
                required: CommandAccess::Moderator
            }
        );
    }
}
