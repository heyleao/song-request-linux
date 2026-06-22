use std::{
    env,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::song_requests::MusicProvider;

pub const APP_ID: &str = "song-request-linux";
pub const APP_NAME: &str = "Song Request Linux";

#[derive(Clone, Debug, Serialize)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub default_provider: MusicProvider,
    pub paths: AppPaths,
}

#[derive(Clone, Debug, Serialize)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub state_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let bind_addr = env::var("SONG_REQUEST_BIND")
            .unwrap_or_else(|_| "127.0.0.1:7384".to_string())
            .parse()
            .context("invalid SONG_REQUEST_BIND value")?;

        Ok(Self {
            bind_addr,
            default_provider: MusicProvider::from_env(),
            paths: AppPaths::from_env()?,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        self.paths.ensure_dirs()
    }
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

        Ok(Self {
            config_dir,
            cache_dir,
            state_dir,
            log_dir,
        })
    }

    fn ensure_dirs(&self) -> Result<()> {
        for path in [
            &self.config_dir,
            &self.cache_dir,
            &self.state_dir,
            &self.log_dir,
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
    }
}
