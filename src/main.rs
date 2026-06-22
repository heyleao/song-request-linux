mod commands;
mod config;
mod connections;
mod dashboard;
mod diagnostics;
mod http;
mod overlay;
mod song_requests;
mod spotify;
mod state;
mod twitch_chat;
mod youtube;

use anyhow::Context;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::Targets, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

use crate::{config::AppConfig, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;
    config.ensure_dirs()?;
    let _log_guard = init_logging(&config);

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

fn init_logging(config: &AppConfig) -> WorkerGuard {
    let filter =
        EnvFilter::try_from_env("SONG_REQUEST_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let file_appender =
        tracing_appender::rolling::daily(&config.paths.log_dir, "song-request-linux.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_filter(Targets::new().with_default(LevelFilter::INFO)),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false)
                .with_target(true),
        )
        .init();

    info!(path = %config.paths.log_dir.display(), "logging initialized");
    guard
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
