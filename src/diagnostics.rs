use serde::Serialize;

use crate::config::AppConfig;

#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticsResponse {
    pub bind_addr: String,
    pub storage: StorageDiagnostics,
    pub secrets: SecretsDiagnostics,
    pub integrations: IntegrationDiagnostics,
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
    pub path: String,
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
                youtube: IntegrationStatus::empty(),
            },
        }
    }
}

impl PathCheck {
    fn new(path: &std::path::Path) -> Self {
        Self {
            path: path.display().to_string(),
            exists: path.exists(),
        }
    }
}

impl IntegrationStatus {
    fn empty() -> Self {
        Self {
            configured: false,
            authenticated: false,
        }
    }
}
