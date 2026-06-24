use serde::{Deserialize, Serialize};

use crate::song_requests::SongRequestInput;

#[derive(Clone, Debug, Deserialize)]
pub struct ChatCommandInput {
    pub requester: String,
    pub message: String,
    #[serde(default)]
    pub is_moderator: bool,
    #[serde(default)]
    pub role: ChatUserRole,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatCommand {
    SongRequest {
        input: SongRequestInput,
        role: ChatUserRole,
    },
    CurrentSong,
    Queue,
    RemoveLast {
        requester: String,
    },
    Skip {
        requester: String,
    },
    Playback {
        requester: String,
        action: PlaybackAction,
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandAccess {
    #[default]
    Everyone,
    Vip,
    Moderator,
    Streamer,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ChatUserRole {
    #[default]
    Viewer,
    Vip,
    Moderator,
    Streamer,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct CommandSettings {
    pub aliases: CommandAliases,
    pub access: CommandAccessConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CommandAliases {
    pub song_request: Vec<String>,
    pub current_song: Vec<String>,
    pub queue: Vec<String>,
    pub remove: Vec<String>,
    pub skip: Vec<String>,
    pub play: Vec<String>,
    pub pause: Vec<String>,
    pub next: Vec<String>,
    pub volume: Vec<String>,
    pub help: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CommandAccessConfig {
    pub song_request: CommandAccess,
    pub current_song: CommandAccess,
    pub queue: CommandAccess,
    pub remove: CommandAccess,
    pub skip: CommandAccess,
    pub play: CommandAccess,
    pub pause: CommandAccess,
    pub next: CommandAccess,
    pub playback: CommandAccess,
    pub volume_read: CommandAccess,
    pub volume_set: CommandAccess,
    pub help: CommandAccess,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackAction {
    Play,
    Pause,
    Next,
}

impl Default for CommandAliases {
    fn default() -> Self {
        Self {
            song_request: commands(&["!sr"]),
            current_song: commands(&["!song"]),
            queue: commands(&["!queue", "!fila", "!q"]),
            remove: commands(&["!rm", "!remove"]),
            skip: commands(&["!skip"]),
            play: commands(&["!play", "!resume"]),
            pause: commands(&["!pause", "!stop"]),
            next: commands(&["!next", "!pular"]),
            volume: commands(&["!vol", "!volume"]),
            help: commands(&["!commands", "!comandos", "!help"]),
        }
    }
}

impl Default for CommandAccessConfig {
    fn default() -> Self {
        Self {
            song_request: CommandAccess::Everyone,
            current_song: CommandAccess::Everyone,
            queue: CommandAccess::Everyone,
            remove: CommandAccess::Everyone,
            skip: CommandAccess::Moderator,
            play: CommandAccess::Moderator,
            pause: CommandAccess::Moderator,
            next: CommandAccess::Moderator,
            playback: CommandAccess::Moderator,
            volume_read: CommandAccess::Everyone,
            volume_set: CommandAccess::Moderator,
            help: CommandAccess::Everyone,
        }
    }
}

pub fn parse_chat_command(input: ChatCommandInput, settings: &CommandSettings) -> ChatCommand {
    let message = input.message.trim();
    let requester = clean_field(&input.requester);
    let role = input.effective_role();

    if let Some(query) = command_payload_any(message, &settings.aliases.song_request) {
        if query.is_empty() {
            return ChatCommand::Ignored;
        }
        if let Some(denied) = deny_if_needed(
            &requester,
            &settings.aliases.song_request[0],
            settings.access.song_request,
            role,
        ) {
            return denied;
        }

        return ChatCommand::SongRequest {
            input: SongRequestInput {
                requester,
                query: clean_field(query),
            },
            role,
        };
    }

    if matches_command(message, &settings.aliases.current_song) {
        if let Some(denied) = deny_if_needed(
            &requester,
            &settings.aliases.current_song[0],
            settings.access.current_song,
            role,
        ) {
            return denied;
        }
        return ChatCommand::CurrentSong;
    }

    if matches_command(message, &settings.aliases.queue) {
        if let Some(denied) = deny_if_needed(
            &requester,
            &settings.aliases.queue[0],
            settings.access.queue,
            role,
        ) {
            return denied;
        }
        return ChatCommand::Queue;
    }

    if matches_command(message, &settings.aliases.remove) {
        if let Some(denied) = deny_if_needed(
            &requester,
            &settings.aliases.remove[0],
            settings.access.remove,
            role,
        ) {
            return denied;
        }
        return ChatCommand::RemoveLast { requester };
    }

    if let Some(action) = playback_action(message, &settings.aliases) {
        if let Some(denied) = deny_if_needed(
            &requester,
            playback_command_name(action, &settings.aliases),
            playback_access(action, &settings.access),
            role,
        ) {
            return denied;
        }
        return ChatCommand::Playback { requester, action };
    }

    if matches_command(message, &settings.aliases.skip) {
        if let Some(denied) = deny_if_needed(
            &requester,
            &settings.aliases.skip[0],
            settings.access.skip,
            role,
        ) {
            return denied;
        }

        return ChatCommand::Skip { requester };
    }

    if let Some(level) = volume_payload(message, &settings.aliases.volume) {
        let access = if level.is_some() {
            settings.access.volume_set
        } else {
            settings.access.volume_read
        };
        if let Some(denied) = deny_if_needed(&requester, &settings.aliases.volume[0], access, role)
        {
            return denied;
        }

        return ChatCommand::Volume { requester, level };
    }

    if matches_command(message, &settings.aliases.help) {
        if let Some(denied) = deny_if_needed(
            &requester,
            &settings.aliases.help[0],
            settings.access.help,
            role,
        ) {
            return denied;
        }
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

fn command_payload_any<'a>(message: &'a str, commands: &[String]) -> Option<&'a str> {
    commands
        .iter()
        .find_map(|command| command_payload(message, command))
}

fn matches_command(message: &str, commands: &[String]) -> bool {
    commands
        .iter()
        .any(|command| message.eq_ignore_ascii_case(command))
}

fn volume_payload(message: &str, commands: &[String]) -> Option<Option<u8>> {
    if matches_command(message, commands) {
        return Some(None);
    }

    let payload = command_payload_any(message, commands)?.trim();
    let level = payload.parse::<u8>().ok()?.min(100);

    Some(Some(level))
}

fn playback_action(message: &str, aliases: &CommandAliases) -> Option<PlaybackAction> {
    if matches_command(message, &aliases.play) {
        return Some(PlaybackAction::Play);
    }
    if matches_command(message, &aliases.pause) {
        return Some(PlaybackAction::Pause);
    }
    if matches_command(message, &aliases.next) {
        return Some(PlaybackAction::Next);
    }

    None
}

fn playback_access(action: PlaybackAction, access: &CommandAccessConfig) -> CommandAccess {
    match action {
        PlaybackAction::Play => access.play,
        PlaybackAction::Pause => access.pause,
        PlaybackAction::Next => access.next,
    }
}

fn playback_command_name(action: PlaybackAction, aliases: &CommandAliases) -> &str {
    match action {
        PlaybackAction::Play => &aliases.play[0],
        PlaybackAction::Pause => &aliases.pause[0],
        PlaybackAction::Next => &aliases.next[0],
    }
}

fn deny_if_needed(
    requester: &str,
    command: &str,
    access: CommandAccess,
    role: ChatUserRole,
) -> Option<ChatCommand> {
    if !role.allows(access) {
        return Some(ChatCommand::AccessDenied {
            requester: requester.to_string(),
            command: command.to_string(),
            required: access,
        });
    }

    None
}

impl ChatCommandInput {
    fn effective_role(&self) -> ChatUserRole {
        if self.is_moderator && self.role < ChatUserRole::Moderator {
            ChatUserRole::Moderator
        } else {
            self.role
        }
    }
}

impl ChatUserRole {
    pub fn from_twitch_tags(tags: Option<&str>) -> Self {
        let Some(tags) = tags else {
            return Self::Viewer;
        };

        let mut role = if tags
            .split(';')
            .any(|tag| tag == "mod=1" || tag == "user-type=mod")
        {
            Self::Moderator
        } else {
            Self::Viewer
        };

        if let Some(badges) = twitch_tag_value(tags, "badges") {
            for badge in badges.split(',') {
                if badge.starts_with("broadcaster/") {
                    return Self::Streamer;
                }
                if badge.starts_with("moderator/") {
                    role = role.max(Self::Moderator);
                }
                if badge.starts_with("vip/") {
                    role = role.max(Self::Vip);
                }
            }
        }

        role
    }

    fn allows(self, access: CommandAccess) -> bool {
        match access {
            CommandAccess::Everyone => true,
            CommandAccess::Vip => self >= Self::Vip,
            CommandAccess::Moderator => self >= Self::Moderator,
            CommandAccess::Streamer => self >= Self::Streamer,
        }
    }
}

fn twitch_tag_value<'a>(tags: &'a str, name: &str) -> Option<&'a str> {
    tags.split(';')
        .find_map(|tag| tag.strip_prefix(name)?.strip_prefix('='))
}

fn commands(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
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

    fn parse(input: ChatCommandInput) -> ChatCommand {
        parse_chat_command(input, &CommandSettings::default())
    }

    #[test]
    fn parses_song_request() {
        let command = parse(ChatCommandInput {
            requester: "bruno".to_string(),
            message: "!sr daft punk one more time".to_string(),
            is_moderator: false,
            role: ChatUserRole::Viewer,
        });

        assert_eq!(
            command,
            ChatCommand::SongRequest {
                input: SongRequestInput {
                    requester: "bruno".to_string(),
                    query: "daft punk one more time".to_string()
                },
                role: ChatUserRole::Viewer
            }
        );
    }

    #[test]
    fn skip_requires_moderator() {
        let command = parse(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!skip".to_string(),
            is_moderator: false,
            role: ChatUserRole::Viewer,
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
        let command = parse(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!fila".to_string(),
            is_moderator: false,
            role: ChatUserRole::Viewer,
        });

        assert_eq!(command, ChatCommand::Queue);
    }

    #[test]
    fn parses_volume_read_and_write() {
        let read = parse(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!vol".to_string(),
            is_moderator: false,
            role: ChatUserRole::Viewer,
        });
        let write = parse(ChatCommandInput {
            requester: "mod".to_string(),
            message: "!vol 35".to_string(),
            is_moderator: true,
            role: ChatUserRole::Moderator,
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
        let command = parse(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!vol 35".to_string(),
            is_moderator: false,
            role: ChatUserRole::Viewer,
        });

        assert_eq!(
            command,
            ChatCommand::AccessDenied {
                requester: "viewer".to_string(),
                command: "!vol".to_string(),
                required: CommandAccess::Moderator
            }
        );
    }

    #[test]
    fn playback_requires_moderator() {
        let command = parse(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!pause".to_string(),
            is_moderator: false,
            role: ChatUserRole::Viewer,
        });

        assert_eq!(
            command,
            ChatCommand::AccessDenied {
                requester: "viewer".to_string(),
                command: "!pause".to_string(),
                required: CommandAccess::Moderator
            }
        );
    }

    #[test]
    fn parses_moderator_playback() {
        let command = parse(ChatCommandInput {
            requester: "mod".to_string(),
            message: "!pause".to_string(),
            is_moderator: true,
            role: ChatUserRole::Moderator,
        });

        assert_eq!(
            command,
            ChatCommand::Playback {
                requester: "mod".to_string(),
                action: PlaybackAction::Pause
            }
        );
    }

    #[test]
    fn parses_remove_aliases() {
        let command = parse(ChatCommandInput {
            requester: "viewer".to_string(),
            message: "!remove".to_string(),
            is_moderator: false,
            role: ChatUserRole::Viewer,
        });

        assert_eq!(
            command,
            ChatCommand::RemoveLast {
                requester: "viewer".to_string()
            }
        );
    }

    #[test]
    fn vip_access_allows_vip_and_blocks_viewer() {
        let mut settings = CommandSettings::default();
        settings.access.song_request = CommandAccess::Vip;

        let vip = parse_chat_command(
            ChatCommandInput {
                requester: "vip".to_string(),
                message: "!sr daft punk".to_string(),
                is_moderator: false,
                role: ChatUserRole::Vip,
            },
            &settings,
        );
        let viewer = parse_chat_command(
            ChatCommandInput {
                requester: "viewer".to_string(),
                message: "!sr daft punk".to_string(),
                is_moderator: false,
                role: ChatUserRole::Viewer,
            },
            &settings,
        );

        assert!(matches!(vip, ChatCommand::SongRequest { .. }));
        assert_eq!(
            viewer,
            ChatCommand::AccessDenied {
                requester: "viewer".to_string(),
                command: "!sr".to_string(),
                required: CommandAccess::Vip
            }
        );
    }

    #[test]
    fn streamer_access_only_allows_streamer() {
        let mut settings = CommandSettings::default();
        settings.access.skip = CommandAccess::Streamer;

        let mod_command = parse_chat_command(
            ChatCommandInput {
                requester: "mod".to_string(),
                message: "!skip".to_string(),
                is_moderator: true,
                role: ChatUserRole::Moderator,
            },
            &settings,
        );
        let streamer_command = parse_chat_command(
            ChatCommandInput {
                requester: "heyleao".to_string(),
                message: "!skip".to_string(),
                is_moderator: false,
                role: ChatUserRole::Streamer,
            },
            &settings,
        );

        assert_eq!(
            mod_command,
            ChatCommand::AccessDenied {
                requester: "mod".to_string(),
                command: "!skip".to_string(),
                required: CommandAccess::Streamer
            }
        );
        assert_eq!(
            streamer_command,
            ChatCommand::Skip {
                requester: "heyleao".to_string()
            }
        );
    }

    #[test]
    fn parses_custom_song_request_alias() {
        let mut settings = CommandSettings::default();
        settings.aliases.song_request = vec!["!ssr".to_string()];
        let command = parse_chat_command(
            ChatCommandInput {
                requester: "viewer".to_string(),
                message: "!ssr scatman".to_string(),
                is_moderator: false,
                role: ChatUserRole::Viewer,
            },
            &settings,
        );

        assert_eq!(
            command,
            ChatCommand::SongRequest {
                input: SongRequestInput {
                    requester: "viewer".to_string(),
                    query: "scatman".to_string()
                },
                role: ChatUserRole::Viewer
            }
        );
    }
}
