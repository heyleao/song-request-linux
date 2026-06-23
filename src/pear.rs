use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

#[derive(Clone, Debug, Serialize)]
pub struct PearStatus {
    pub configured: bool,
    pub reachable: bool,
    pub base_url: String,
    pub now_playing: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PearNowPlaying {
    #[serde(default, alias = "videoId", alias = "id")]
    pub video_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub artist: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PearQueueRequest<'a> {
    video_id: &'a str,
    insert_position: PearInsertPosition,
}

#[derive(Clone, Copy, Debug, Serialize)]
enum PearInsertPosition {
    InsertAfterCurrentVideo,
}

pub async fn status(config: &AppConfig) -> PearStatus {
    let now_playing = now_playing(config).await.ok();
    PearStatus {
        configured: matches!(
            config.youtube.playback,
            crate::config::YoutubePlayback::Pear
        ),
        reachable: now_playing.is_some(),
        base_url: config.youtube.pear_base_url.clone(),
        now_playing: now_playing.and_then(PearNowPlaying::display_name),
    }
}

pub async fn now_playing(config: &AppConfig) -> Result<PearNowPlaying> {
    let response = client()?
        .get(endpoint(config, "song-info"))
        .send()
        .await
        .context("Pear nao respondeu em /song-info")?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("Pear /song-info falhou com {status}: {}", body.trim());
    }

    Ok(response.json::<PearNowPlaying>().await?)
}

pub async fn enqueue_after_current(config: &AppConfig, video_id: &str) -> Result<()> {
    let response = client()?
        .post(endpoint(config, "queue"))
        .json(&PearQueueRequest {
            video_id,
            insert_position: PearInsertPosition::InsertAfterCurrentVideo,
        })
        .send()
        .await
        .context("Pear nao respondeu em /queue")?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let body = response.text().await.unwrap_or_default();
    bail!(
        "Pear nao aceitou o video {video_id} ({status}): {}",
        body.trim()
    );
}

pub async fn play(config: &AppConfig) -> Result<()> {
    empty_post(config, "play").await
}

pub async fn pause(config: &AppConfig) -> Result<()> {
    empty_post(config, "pause").await
}

pub async fn next(config: &AppConfig) -> Result<()> {
    empty_post(config, "next").await
}

impl PearNowPlaying {
    fn display_name(self) -> Option<String> {
        match (self.artist, self.title, self.video_id) {
            (Some(artist), Some(title), _) if !artist.is_empty() && !title.is_empty() => {
                Some(format!("{artist} - {title}"))
            }
            (_, Some(title), _) if !title.is_empty() => Some(title),
            (_, _, Some(video_id)) if !video_id.is_empty() => Some(format!("YouTube {video_id}")),
            _ => None,
        }
    }
}

async fn empty_post(config: &AppConfig, path: &str) -> Result<()> {
    let response = client()?
        .post(endpoint(config, path))
        .send()
        .await
        .with_context(|| format!("Pear nao respondeu em /{path}"))?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let body = response.text().await.unwrap_or_default();
    bail!("Pear /{path} falhou com {status}: {}", body.trim());
}

fn endpoint(config: &AppConfig, path: &str) -> String {
    format!(
        "{}/{}",
        config.youtube.pear_base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn client() -> Result<Client> {
    Ok(Client::builder().timeout(Duration::from_secs(3)).build()?)
}
