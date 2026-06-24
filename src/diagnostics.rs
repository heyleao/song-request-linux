use std::{env, path::Path};

use serde::Serialize;

use crate::config::AppConfig;

#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticsResponse {
    pub bind_addr: String,
    pub storage: StorageDiagnostics,
    pub secrets: SecretsDiagnostics,
    pub integrations: IntegrationDiagnostics,
    pub runtime: RuntimeDiagnostics,
}

#[derive(Clone, Debug, Serialize)]
pub struct StorageDiagnostics {
    pub config_dir: PathCheck,
    pub cache_dir: PathCheck,
    pub state_dir: PathCheck,
    pub log_dir: PathCheck,
}

#[derive(Clone, Debug, Serialize)]
pub struct PathCheck {
    pub exists: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SecretsDiagnostics {
    pub provider: &'static str,
    pub ready: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct IntegrationDiagnostics {
    pub twitch: IntegrationStatus,
    pub spotify: IntegrationStatus,
    pub youtube: IntegrationStatus,
}

#[derive(Clone, Debug, Serialize)]
pub struct RuntimeDiagnostics {
    pub setsid: CommandCheck,
    pub curl: CommandCheck,
    pub xdg_open: CommandCheck,
    pub ss: CommandCheck,
    pub yt_dlp: CommandCheck,
    pub pear: CommandCheck,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandCheck {
    pub command: &'static str,
    pub available: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct IntegrationStatus {
    pub configured: bool,
    pub authenticated: bool,
}

impl DiagnosticsResponse {
    pub fn collect(config: &AppConfig) -> Self {
        Self {
            bind_addr: config.bind_addr.to_string(),
            storage: StorageDiagnostics {
                config_dir: PathCheck::new(&config.paths.config_dir),
                cache_dir: PathCheck::new(&config.paths.cache_dir),
                state_dir: PathCheck::new(&config.paths.state_dir),
                log_dir: PathCheck::new(&config.paths.log_dir),
            },
            secrets: SecretsDiagnostics {
                provider: "not_configured",
                ready: false,
            },
            integrations: IntegrationDiagnostics {
                twitch: IntegrationStatus {
                    configured: config.twitch.username.is_some()
                        && config.twitch.channel.is_some()
                        && config.twitch.token_configured,
                    authenticated: false,
                },
                spotify: IntegrationStatus {
                    configured: config.spotify.client_id.is_some(),
                    authenticated: false,
                },
                youtube: IntegrationStatus {
                    configured: !config.youtube.api_keys.is_empty(),
                    authenticated: false,
                },
            },
            runtime: RuntimeDiagnostics {
                setsid: CommandCheck::new("setsid"),
                curl: CommandCheck::new("curl"),
                xdg_open: CommandCheck::new("xdg-open"),
                ss: CommandCheck::new("ss"),
                yt_dlp: CommandCheck::new("yt-dlp"),
                pear: CommandCheck {
                    command: "pear/pear-desktop",
                    available: command_exists("pear") || command_exists("pear-desktop"),
                },
            },
        }
    }
}

impl CommandCheck {
    fn new(command: &'static str) -> Self {
        Self {
            command,
            available: command_exists(command),
        }
    }
}

fn command_exists(command: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&paths).any(|path| {
        let candidate = path.join(command);
        Path::new(&candidate).is_file()
    })
}

impl PathCheck {
    fn new(path: &std::path::Path) -> Self {
        Self {
            exists: path.exists(),
        }
    }
}
