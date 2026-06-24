use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    commands::{ChatUserRole, CommandAccess, CommandSettings},
    song_requests::MusicProvider,
};

pub const APP_ID: &str = "song-request-linux";
pub const APP_NAME: &str = "Song Request Linux";

#[derive(Clone, Debug, Serialize)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub https_bind_addr: SocketAddr,
    pub default_provider: MusicProvider,
    pub spotify: SpotifyConfig,
    pub youtube: YoutubeConfig,
    pub twitch: TwitchBotConfig,
    pub paths: AppPaths,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct UserConfig {
    pub default_provider: MusicProvider,
    pub spotify_client_id: Option<String>,
    pub spotify_redirect_uri: Option<String>,
    pub spotify_fallback_enabled: bool,
    pub queue_persistence_enabled: bool,
    pub youtube_playback: YoutubePlayback,
    pub pear_base_url: Option<String>,
    pub youtube_max_duration_seconds: u64,
    pub youtube_allow_non_music: bool,
    pub twitch_client_id: Option<String>,
    pub twitch_bot_username: Option<String>,
    pub twitch_channel: Option<String>,
    pub command_settings: CommandSettings,
    pub queue_limits: QueueLimitConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct UserSecrets {
    pub twitch_bot_oauth_token: Option<String>,
    pub youtube_api_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UiConfigInput {
    pub default_provider: MusicProvider,
    pub youtube_playback: YoutubePlayback,
    pub pear_base_url: Option<String>,
    pub spotify_client_id: Option<String>,
    pub spotify_fallback_enabled: bool,
    pub queue_persistence_enabled: bool,
    pub twitch_client_id: Option<String>,
    pub twitch_bot_username: Option<String>,
    pub twitch_channel: Option<String>,
    pub twitch_bot_oauth_token: Option<String>,
    pub youtube_api_key: Option<String>,
    pub youtube_max_duration_seconds: Option<u64>,
    pub youtube_allow_non_music: bool,
    pub command_settings: Option<CommandSettings>,
    pub queue_limits: Option<QueueLimitConfig>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UiConfigView {
    pub default_provider: MusicProvider,
    pub youtube_playback: YoutubePlayback,
    pub pear_base_url: String,
    pub spotify_client_id: Option<String>,
    pub spotify_fallback_enabled: bool,
    pub queue_persistence_enabled: bool,
    pub twitch_client_id: Option<String>,
    pub twitch_bot_username: Option<String>,
    pub twitch_channel: Option<String>,
    pub twitch_bot_token_configured: bool,
    pub youtube_api_key_configured: bool,
    pub youtube_max_duration_seconds: u64,
    pub youtube_allow_non_music: bool,
    pub command_settings: CommandSettings,
    pub queue_limits: QueueLimitConfig,
    pub commands_summary: Vec<CommandSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct QueueLimitConfig {
    pub viewer: u16,
    #[serde(default = "default_subscriber_queue_limit")]
    pub subscriber: u16,
    pub vip: u16,
    pub moderator: u16,
    pub streamer: u16,
}

fn default_subscriber_queue_limit() -> u16 {
    2
}

impl Default for QueueLimitConfig {
    fn default() -> Self {
        Self {
            viewer: 1,
            subscriber: 2,
            vip: 3,
            moderator: 10,
            streamer: 0,
        }
    }
}

impl QueueLimitConfig {
    pub fn limit_for(&self, role: ChatUserRole) -> u16 {
        match role {
            ChatUserRole::Viewer => self.viewer,
            ChatUserRole::Subscriber => self.subscriber,
            ChatUserRole::Vip => self.vip,
            ChatUserRole::Moderator => self.moderator,
            ChatUserRole::Streamer => self.streamer,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandSummary {
    pub name: &'static str,
    pub aliases: Vec<String>,
    pub access: crate::commands::CommandAccess,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpotifyConfig {
    pub client_id: Option<String>,
    pub redirect_uri: String,
    pub fallback_enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct YoutubeConfig {
    pub api_key: Option<String>,
    pub playback: YoutubePlayback,
    pub pear_base_url: String,
    pub max_duration_seconds: u64,
    pub allow_non_music: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YoutubePlayback {
    #[default]
    Browser,
    Pear,
}

#[derive(Clone, Debug, Serialize)]
pub struct TwitchBotConfig {
    pub username: Option<String>,
    pub channel: Option<String>,
    pub token_configured: bool,
}

#[derive(Clone, Debug)]
pub struct TwitchBotSecrets {
    pub username: String,
    pub oauth_token: String,
    pub channel: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub state_dir: PathBuf,
    pub log_dir: PathBuf,
    pub tls_dir: PathBuf,
    pub queue_file: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let paths = AppPaths::from_env()?;
        let user_config = load_user_config_from_paths(&paths).unwrap_or_default();
        let user_secrets = load_user_secrets_from_paths(&paths).unwrap_or_default();
        let bind_addr = env::var("SONG_REQUEST_BIND")
            .unwrap_or_else(|_| "127.0.0.1:7384".to_string())
            .parse()
            .context("invalid SONG_REQUEST_BIND value")?;
        let https_bind_addr = env::var("SONG_REQUEST_HTTPS_BIND")
            .unwrap_or_else(|_| "127.0.0.1:7443".to_string())
            .parse()
            .context("invalid SONG_REQUEST_HTTPS_BIND value")?;

        Ok(Self {
            bind_addr,
            https_bind_addr,
            default_provider: MusicProvider::from_env().unwrap_or(user_config.default_provider),
            spotify: SpotifyConfig::from_sources(&user_config),
            youtube: YoutubeConfig::from_sources(&user_config, &user_secrets),
            twitch: TwitchBotConfig::from_sources(&user_config, &user_secrets),
            paths,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        self.paths.ensure_dirs()
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            default_provider: MusicProvider::Youtube,
            spotify_client_id: None,
            spotify_redirect_uri: None,
            spotify_fallback_enabled: false,
            queue_persistence_enabled: false,
            youtube_playback: YoutubePlayback::Browser,
            pear_base_url: None,
            youtube_max_duration_seconds: 360,
            youtube_allow_non_music: false,
            twitch_client_id: None,
            twitch_bot_username: None,
            twitch_channel: None,
            command_settings: CommandSettings::default(),
            queue_limits: QueueLimitConfig::default(),
        }
    }
}

impl SpotifyConfig {
    fn from_sources(user_config: &UserConfig) -> Self {
        Self {
            client_id: clean_optional_env("SPOTIFY_CLIENT_ID")
                .or_else(|| user_config.spotify_client_id.clone()),
            redirect_uri: clean_optional_env("SPOTIFY_REDIRECT_URI")
                .or_else(|| user_config.spotify_redirect_uri.clone())
                .unwrap_or_else(|| "http://127.0.0.1:7384/auth/spotify/callback".to_string()),
            fallback_enabled: clean_optional_env("SPOTIFY_FALLBACK_ENABLED")
                .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
                .unwrap_or(user_config.spotify_fallback_enabled),
        }
    }
}

impl YoutubeConfig {
    fn from_sources(user_config: &UserConfig, user_secrets: &UserSecrets) -> Self {
        let max_duration_seconds = clean_optional_env("YOUTUBE_MAX_DURATION_SECONDS")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(user_config.youtube_max_duration_seconds)
            .clamp(30, 86_400);
        let allow_non_music = clean_optional_env("YOUTUBE_ALLOW_NON_MUSIC")
            .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(user_config.youtube_allow_non_music);
        let playback = clean_optional_env("YOUTUBE_PLAYBACK")
            .and_then(|value| YoutubePlayback::parse(&value))
            .unwrap_or(user_config.youtube_playback);
        let pear_base_url = clean_optional_env("PEAR_BASE_URL")
            .or_else(|| user_config.pear_base_url.clone())
            .unwrap_or_else(default_pear_base_url);

        Self {
            api_key: clean_optional_env("YOUTUBE_API_KEY")
                .or_else(|| user_secrets.youtube_api_key.clone()),
            playback,
            pear_base_url,
            max_duration_seconds,
            allow_non_music,
        }
    }
}

impl YoutubePlayback {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "browser" | "obs" | "browser_source" => Some(Self::Browser),
            "pear" | "pear_desktop" | "youtube_music" => Some(Self::Pear),
            _ => None,
        }
    }
}

impl TwitchBotConfig {
    fn from_sources(user_config: &UserConfig, user_secrets: &UserSecrets) -> Self {
        Self {
            username: clean_optional_env("TWITCH_BOT_USERNAME")
                .or_else(|| user_config.twitch_bot_username.clone()),
            channel: clean_optional_env("TWITCH_CHANNEL")
                .or_else(|| user_config.twitch_channel.clone()),
            token_configured: clean_optional_env("TWITCH_BOT_OAUTH_TOKEN")
                .or_else(|| user_secrets.twitch_bot_oauth_token.clone())
                .is_some(),
        }
    }
}

impl TwitchBotSecrets {
    pub fn from_env() -> Option<Self> {
        let paths = AppPaths::from_env().ok()?;
        let user_config = load_user_config_from_paths(&paths).unwrap_or_default();
        let user_secrets = load_user_secrets_from_paths(&paths).unwrap_or_default();
        let username =
            clean_optional_env("TWITCH_BOT_USERNAME").or(user_config.twitch_bot_username)?;
        let oauth_token =
            clean_optional_env("TWITCH_BOT_OAUTH_TOKEN").or(user_secrets.twitch_bot_oauth_token)?;
        let channel = clean_optional_env("TWITCH_CHANNEL").or(user_config.twitch_channel)?;

        Some(Self {
            username: username.trim_start_matches('@').to_lowercase(),
            oauth_token,
            channel: channel.trim_start_matches('#').to_lowercase(),
        })
    }
}

impl UiConfigView {
    pub fn load(paths: &AppPaths) -> Self {
        let user_config = load_user_config_from_paths(paths).unwrap_or_default();
        let user_secrets = load_user_secrets_from_paths(paths).unwrap_or_default();

        Self {
            default_provider: user_config.default_provider,
            youtube_playback: user_config.youtube_playback,
            pear_base_url: user_config
                .pear_base_url
                .unwrap_or_else(default_pear_base_url),
            spotify_client_id: user_config.spotify_client_id,
            spotify_fallback_enabled: user_config.spotify_fallback_enabled,
            queue_persistence_enabled: user_config.queue_persistence_enabled,
            twitch_client_id: user_config.twitch_client_id,
            twitch_bot_username: user_config.twitch_bot_username,
            twitch_channel: user_config.twitch_channel,
            twitch_bot_token_configured: user_secrets.twitch_bot_oauth_token.is_some(),
            youtube_api_key_configured: user_secrets.youtube_api_key.is_some(),
            youtube_max_duration_seconds: user_config.youtube_max_duration_seconds,
            youtube_allow_non_music: user_config.youtube_allow_non_music,
            command_settings: user_config.command_settings.clone(),
            queue_limits: user_config.queue_limits.clone(),
            commands_summary: command_summary(&user_config.command_settings),
        }
    }
}

pub fn queue_persistence_enabled(paths: &AppPaths) -> bool {
    load_user_config_from_paths(paths)
        .map(|config| config.queue_persistence_enabled)
        .unwrap_or_default()
}

pub fn save_ui_config(paths: &AppPaths, input: UiConfigInput) -> Result<UiConfigView> {
    fs::create_dir_all(&paths.config_dir)?;
    fs::create_dir_all(&paths.state_dir)?;

    let existing_secrets = load_user_secrets_from_paths(paths).unwrap_or_default();
    let user_config = UserConfig {
        default_provider: input.default_provider,
        youtube_playback: input.youtube_playback,
        pear_base_url: clean_optional_value(input.pear_base_url),
        spotify_client_id: clean_optional_value(input.spotify_client_id),
        spotify_redirect_uri: None,
        spotify_fallback_enabled: input.spotify_fallback_enabled,
        queue_persistence_enabled: input.queue_persistence_enabled,
        youtube_max_duration_seconds: input
            .youtube_max_duration_seconds
            .unwrap_or(360)
            .clamp(30, 86_400),
        youtube_allow_non_music: input.youtube_allow_non_music,
        twitch_client_id: clean_optional_value(input.twitch_client_id),
        twitch_bot_username: clean_optional_value(input.twitch_bot_username),
        twitch_channel: clean_optional_value(input.twitch_channel),
        command_settings: normalize_command_settings(input.command_settings.unwrap_or_default()),
        queue_limits: normalize_queue_limits(input.queue_limits.unwrap_or_default()),
    };
    let user_secrets = UserSecrets {
        twitch_bot_oauth_token: clean_optional_value(input.twitch_bot_oauth_token)
            .or(existing_secrets.twitch_bot_oauth_token),
        youtube_api_key: clean_optional_value(input.youtube_api_key)
            .or(existing_secrets.youtube_api_key),
    };

    fs::write(
        user_config_path(paths),
        serde_json::to_vec_pretty(&user_config)?,
    )?;
    let secrets_path = user_secrets_path(paths);
    fs::write(&secrets_path, serde_json::to_vec_pretty(&user_secrets)?)?;
    restrict_file_permissions(&secrets_path);

    Ok(UiConfigView::load(paths))
}

pub fn command_settings(paths: &AppPaths) -> CommandSettings {
    load_user_config_from_paths(paths)
        .map(|config| normalize_command_settings(config.command_settings))
        .unwrap_or_default()
}

fn normalize_queue_limits(mut limits: QueueLimitConfig) -> QueueLimitConfig {
    limits.viewer = limits.viewer.min(100);
    limits.subscriber = limits.subscriber.min(100);
    limits.vip = limits.vip.min(100);
    limits.moderator = limits.moderator.min(100);
    limits.streamer = limits.streamer.min(100);
    limits
}

fn normalize_command_settings(mut settings: CommandSettings) -> CommandSettings {
    if settings.access.play == CommandAccess::Moderator
        && settings.access.pause == CommandAccess::Moderator
        && settings.access.next == CommandAccess::Moderator
        && settings.access.playback != CommandAccess::Moderator
    {
        settings.access.play = settings.access.playback;
        settings.access.pause = settings.access.playback;
        settings.access.next = settings.access.playback;
    }

    settings.aliases.song_request = normalize_aliases(settings.aliases.song_request, &["!sr"]);
    settings.aliases.current_song = normalize_aliases(settings.aliases.current_song, &["!song"]);
    settings.aliases.queue = normalize_aliases(settings.aliases.queue, &["!queue", "!fila", "!q"]);
    settings.aliases.remove = normalize_aliases(settings.aliases.remove, &["!rm", "!remove"]);
    settings.aliases.skip = normalize_aliases(settings.aliases.skip, &["!skip"]);
    settings.aliases.play = normalize_aliases(settings.aliases.play, &["!play", "!resume"]);
    settings.aliases.pause = normalize_aliases(settings.aliases.pause, &["!pause", "!stop"]);
    settings.aliases.next = normalize_aliases(settings.aliases.next, &["!next", "!pular"]);
    settings.aliases.volume = normalize_aliases(settings.aliases.volume, &["!vol", "!volume"]);
    settings.aliases.help =
        normalize_aliases(settings.aliases.help, &["!commands", "!comandos", "!help"]);
    settings
}

fn normalize_aliases(values: Vec<String>, fallback: &[&str]) -> Vec<String> {
    let mut aliases = Vec::new();
    for value in values {
        let Some(alias) = sanitize_command_alias(&value) else {
            continue;
        };
        if !aliases
            .iter()
            .any(|item: &String| item.eq_ignore_ascii_case(&alias))
        {
            aliases.push(alias);
        }
    }
    if aliases.is_empty() {
        return fallback.iter().map(|value| value.to_string()).collect();
    }
    aliases
}

fn sanitize_command_alias(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 32 || value.chars().any(char::is_whitespace) {
        return None;
    }

    let alias = if value.starts_with('!') {
        value.to_string()
    } else {
        format!("!{value}")
    };

    let rest = alias.strip_prefix('!')?;
    if rest.is_empty()
        || !rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
    {
        return None;
    }

    Some(alias)
}

fn command_summary(settings: &CommandSettings) -> Vec<CommandSummary> {
    vec![
        CommandSummary {
            name: "Pedido",
            aliases: settings.aliases.song_request.clone(),
            access: settings.access.song_request,
        },
        CommandSummary {
            name: "Atual",
            aliases: settings.aliases.current_song.clone(),
            access: settings.access.current_song,
        },
        CommandSummary {
            name: "Fila",
            aliases: settings.aliases.queue.clone(),
            access: settings.access.queue,
        },
        CommandSummary {
            name: "Remover ultimo pedido",
            aliases: settings.aliases.remove.clone(),
            access: settings.access.remove,
        },
        CommandSummary {
            name: "Skip",
            aliases: settings.aliases.skip.clone(),
            access: settings.access.skip,
        },
        CommandSummary {
            name: "Play",
            aliases: settings.aliases.play.clone(),
            access: settings.access.play,
        },
        CommandSummary {
            name: "Pause/Stop",
            aliases: settings.aliases.pause.clone(),
            access: settings.access.pause,
        },
        CommandSummary {
            name: "Next/Pular",
            aliases: settings.aliases.next.clone(),
            access: settings.access.next,
        },
        CommandSummary {
            name: "Volume atual",
            aliases: settings.aliases.volume.clone(),
            access: settings.access.volume_read,
        },
        CommandSummary {
            name: "Mudar volume",
            aliases: settings.aliases.volume.clone(),
            access: settings.access.volume_set,
        },
        CommandSummary {
            name: "Ajuda",
            aliases: settings.aliases.help.clone(),
            access: settings.access.help,
        },
    ]
}

impl AppPaths {
    fn from_env() -> Result<Self> {
        let config_base = env_path("XDG_CONFIG_HOME").unwrap_or_else(|| home_path(".config"));
        let cache_base = env_path("XDG_CACHE_HOME").unwrap_or_else(|| home_path(".cache"));
        let state_base = env_path("XDG_STATE_HOME").unwrap_or_else(|| home_path(".local/state"));

        let config_dir = config_base.join(APP_ID);
        let cache_dir = cache_base.join(APP_ID);
        let state_dir = state_base.join(APP_ID);
        let log_dir = state_dir.join("logs");
        let tls_dir = state_dir.join("tls");
        let queue_file = state_dir.join("queue.json");

        Ok(Self {
            config_dir,
            cache_dir,
            state_dir,
            log_dir,
            tls_dir,
            queue_file,
        })
    }

    fn ensure_dirs(&self) -> Result<()> {
        for path in [
            &self.config_dir,
            &self.cache_dir,
            &self.state_dir,
            &self.log_dir,
            &self.tls_dir,
        ] {
            std::fs::create_dir_all(path)
                .with_context(|| format!("failed to create {}", path.display()))?;
        }

        Ok(())
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn clean_optional_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn clean_optional_value(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn default_pear_base_url() -> String {
    "http://127.0.0.1:26538/api/v1".to_string()
}

fn load_user_config_from_paths(paths: &AppPaths) -> Result<UserConfig> {
    let data = fs::read_to_string(user_config_path(paths))?;
    Ok(serde_json::from_str(&data)?)
}

fn load_user_secrets_from_paths(paths: &AppPaths) -> Result<UserSecrets> {
    let data = fs::read_to_string(user_secrets_path(paths))?;
    Ok(serde_json::from_str(&data)?)
}

fn user_config_path(paths: &AppPaths) -> PathBuf {
    paths.config_dir.join("config.json")
}

fn user_secrets_path(paths: &AppPaths) -> PathBuf {
    paths.state_dir.join("secrets.json")
}

#[cfg(unix)]
fn restrict_file_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &std::path::Path) {}

fn home_path(relative: impl AsRef<Path>) -> PathBuf {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    home.join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_paths_are_under_app_id() {
        let paths = AppPaths::from_env().expect("paths");

        assert!(paths.config_dir.ends_with(APP_ID));
        assert!(paths.cache_dir.ends_with(APP_ID));
        assert!(paths.state_dir.ends_with(APP_ID));
        assert!(paths.log_dir.ends_with(format!("{APP_ID}/logs")));
        assert!(paths.tls_dir.ends_with(format!("{APP_ID}/tls")));
    }

    #[test]
    fn command_aliases_reject_control_whitespace_and_shell_chars() {
        let mut settings = CommandSettings::default();
        settings.aliases.song_request = vec![
            "!ssr".to_string(),
            "!bad\r\nPRIVMSG".to_string(),
            "!with space".to_string(),
            "!semi;colon".to_string(),
        ];

        let settings = normalize_command_settings(settings);

        assert_eq!(settings.aliases.song_request, vec!["!ssr"]);
    }

    #[test]
    fn command_aliases_fallback_when_all_values_are_rejected() {
        let mut settings = CommandSettings::default();
        settings.aliases.song_request = vec!["!bad\r\nPRIVMSG".to_string()];

        let settings = normalize_command_settings(settings);

        assert_eq!(settings.aliases.song_request, vec!["!sr"]);
    }
}
