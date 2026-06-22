mod commands;
mod config;
mod dashboard;
mod diagnostics;
mod http;
mod overlay;
mod song_requests;
mod state;
mod twitch_chat;
mod youtube;

use anyhow::Context;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{config::AppConfig, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let config = AppConfig::from_env()?;
    config.ensure_dirs()?;

    let addr = config.bind_addr;
    let state = AppState::new(config);
    let app = http::router(state.clone());

    if let Some(secrets) = config::TwitchBotSecrets::from_env() {
        twitch_chat::spawn_bot(state.clone(), secrets);
    } else {
        info!("twitch bot disabled; set TWITCH_BOT_USERNAME, TWITCH_BOT_OAUTH_TOKEN and TWITCH_CHANNEL to enable it");
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind local server on {addr}"))?;

    info!("listening on http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server failed")?;

    Ok(())
}

fn init_logging() {
    let filter =
        EnvFilter::try_from_env("SONG_REQUEST_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
