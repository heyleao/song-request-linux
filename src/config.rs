use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::song_requests::MusicProvider;

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
    pub youtube_max_duration_seconds: u64,
    pub youtube_allow_non_music: bool,
    pub twitch_client_id: Option<String>,
    pub twitch_bot_username: Option<String>,
    pub twitch_channel: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct UserSecrets {
    pub twitch_bot_oauth_token: Option<String>,
    pub youtube_api_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UiConfigInput {
    pub default_provider: MusicProvider,
    pub spotify_client_id: Option<String>,
    pub twitch_client_id: Option<String>,
    pub twitch_bot_username: Option<String>,
    pub twitch_channel: Option<String>,
    pub twitch_bot_oauth_token: Option<String>,
    pub youtube_api_key: Option<String>,
    pub youtube_max_duration_seconds: Option<u64>,
    pub youtube_allow_non_music: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct UiConfigView {
    pub default_provider: MusicProvider,
    pub spotify_client_id: Option<String>,
    pub twitch_client_id: Option<String>,
    pub twitch_bot_username: Option<String>,
    pub twitch_channel: Option<String>,
    pub twitch_bot_token_configured: bool,
    pub youtube_api_key_configured: bool,
    pub youtube_max_duration_seconds: u64,
    pub youtube_allow_non_music: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpotifyConfig {
    pub client_id: Option<String>,
    pub redirect_uri: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct YoutubeConfig {
    pub api_key: Option<String>,
    pub max_duration_seconds: u64,
    pub allow_non_music: bool,
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
            youtube_max_duration_seconds: 360,
            youtube_allow_non_music: false,
            twitch_client_id: None,
            twitch_bot_username: None,
            twitch_channel: None,
        }
    }
}

impl Default for UserSecrets {
    fn default() -> Self {
        Self {
            twitch_bot_oauth_token: None,
            youtube_api_key: None,
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

        Self {
            api_key: clean_optional_env("YOUTUBE_API_KEY")
                .or_else(|| user_secrets.youtube_api_key.clone()),
            max_duration_seconds,
            allow_non_music,
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
            spotify_client_id: user_config.spotify_client_id,
            twitch_client_id: user_config.twitch_client_id,
            twitch_bot_username: user_config.twitch_bot_username,
            twitch_channel: user_config.twitch_channel,
            twitch_bot_token_configured: user_secrets.twitch_bot_oauth_token.is_some(),
            youtube_api_key_configured: user_secrets.youtube_api_key.is_some(),
            youtube_max_duration_seconds: user_config.youtube_max_duration_seconds,
            youtube_allow_non_music: user_config.youtube_allow_non_music,
        }
    }
}

pub fn save_ui_config(paths: &AppPaths, input: UiConfigInput) -> Result<UiConfigView> {
    fs::create_dir_all(&paths.config_dir)?;
    fs::create_dir_all(&paths.state_dir)?;

    let existing_secrets = load_user_secrets_from_paths(paths).unwrap_or_default();
    let user_config = UserConfig {
        default_provider: input.default_provider,
        spotify_client_id: clean_optional_value(input.spotify_client_id),
        spotify_redirect_uri: None,
        youtube_max_duration_seconds: input
            .youtube_max_duration_seconds
            .unwrap_or(360)
            .clamp(30, 86_400),
        youtube_allow_non_music: input.youtube_allow_non_music,
        twitch_client_id: clean_optional_value(input.twitch_client_id),
        twitch_bot_username: clean_optional_value(input.twitch_bot_username),
        twitch_channel: clean_optional_value(input.twitch_channel),
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

        Ok(Self {
            config_dir,
            cache_dir,
            state_dir,
            log_dir,
            tls_dir,
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
}
