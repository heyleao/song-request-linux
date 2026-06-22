use anyhow::{bail, Context, Result};
use rand::{distr::Alphanumeric, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::{save_ui_config, AppConfig, UiConfigInput, UiConfigView};

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_VALIDATE_URL: &str = "https://id.twitch.tv/oauth2/validate";
const TWITCH_SCOPES: &str = "chat:read chat:edit";
const TWITCH_REDIRECT_URI: &str = "https://localhost:7443/auth/twitch/callback";

#[derive(Debug, Serialize)]
pub struct TwitchAuthStart {
    pub auth_url: String,
}

#[derive(Debug, Deserialize)]
pub struct TwitchTokenInput {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
struct TwitchValidateResponse {
    login: String,
}

pub fn start_auth(config: &AppConfig) -> Result<TwitchAuthStart> {
    let client_id = config_twitch_client_id(config)?;
    let mut url = Url::parse(TWITCH_AUTH_URL)?;
    url.query_pairs_mut()
        .append_pair("response_type", "token")
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", TWITCH_REDIRECT_URI)
        .append_pair("scope", TWITCH_SCOPES)
        .append_pair("state", &random_string(32))
        .append_pair("force_verify", "true");

    Ok(TwitchAuthStart {
        auth_url: url.to_string(),
    })
}

pub async fn save_bot_token(config: &AppConfig, input: TwitchTokenInput) -> Result<UiConfigView> {
    let token = input
        .access_token
        .trim()
        .trim_start_matches("oauth:")
        .to_string();
    if token.is_empty() {
        bail!("missing Twitch access token");
    }

    let login = validate_token(&token).await?;
    let current = UiConfigView::load(&config.paths);
    let user_config = UiConfigInput {
        default_provider: current.default_provider,
        spotify_client_id: current.spotify_client_id,
        twitch_client_id: current.twitch_client_id,
        twitch_bot_username: Some(login),
        twitch_channel: current.twitch_channel,
        twitch_bot_oauth_token: Some(token),
    };

    save_ui_config(&config.paths, user_config)
}

fn config_twitch_client_id(config: &AppConfig) -> Result<String> {
    let view = UiConfigView::load(&config.paths);
    view.twitch_client_id
        .context("Configure Twitch Client ID before connecting the bot")
}

async fn validate_token(token: &str) -> Result<String> {
    let response = Client::new()
        .get(TWITCH_VALIDATE_URL)
        .bearer_auth(token)
        .send()
        .await
        .context("failed to validate Twitch token")?;

    if !response.status().is_success() {
        bail!("Twitch token validation failed with {}", response.status());
    }

    Ok(response.json::<TwitchValidateResponse>().await?.login)
}

fn random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
