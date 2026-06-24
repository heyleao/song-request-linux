use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;

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
    #[serde(default, rename = "isPaused")]
    pub is_paused: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PearQueueRequest<'a> {
    video_id: &'a str,
    insert_position: PearInsertPosition,
}

#[derive(Debug, Deserialize)]
pub struct PearVolume {
    pub state: u8,
    #[serde(default, rename = "isMuted")]
    pub is_muted: bool,
}

#[derive(Debug, Deserialize)]
struct PearQueueInfo {
    #[serde(default)]
    items: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PearQueueItem {
    index: usize,
    video_id: Option<String>,
    selected: bool,
}

#[derive(Debug, Serialize)]
struct PearVolumeRequest {
    volume: u8,
}

#[derive(Debug, Serialize)]
struct PearQueueIndexRequest {
    index: usize,
}

#[derive(Clone, Copy, Debug, Serialize)]
enum PearInsertPosition {
    #[serde(rename = "INSERT_AFTER_CURRENT_VIDEO")]
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
    match read_now_playing(config, "song").await {
        Ok(song) => return Ok(song),
        Err(error) => {
            tracing::debug!(%error, "Pear /song failed; falling back to /song-info");
        }
    }

    read_now_playing(config, "song-info").await
}

async fn read_now_playing(config: &AppConfig, path: &str) -> Result<PearNowPlaying> {
    let response = client()?
        .get(endpoint(config, path))
        .send()
        .await
        .with_context(|| format!("Pear nao respondeu em /{path}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("Pear /{path} falhou com {status}: {}", body.trim());
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

pub async fn clear_queue(config: &AppConfig) -> Result<()> {
    let response = client()?
        .delete(endpoint(config, "queue"))
        .send()
        .await
        .context("Pear nao respondeu em DELETE /queue")?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let body = response.text().await.unwrap_or_default();
    bail!("Pear nao aceitou limpar a fila ({status}): {}", body.trim());
}

pub async fn compact_queue_for_app(
    config: &AppConfig,
    current_video_id: &str,
    next_video_id: Option<&str>,
) -> Result<()> {
    let items = queue_items(config).await?;
    let mut kept_current = false;
    let mut kept_next = false;
    let mut delete_indexes = Vec::new();

    for item in items {
        let keep_current = item.selected
            && item
                .video_id
                .as_deref()
                .is_some_and(|video_id| video_id == current_video_id)
            && !kept_current;
        let keep_next = next_video_id.is_some()
            && item.video_id.as_deref() == next_video_id
            && !item.selected
            && !kept_next;

        if keep_current {
            kept_current = true;
        } else if keep_next {
            kept_next = true;
        } else {
            delete_indexes.push(item.index);
        }
    }

    for index in delete_indexes.into_iter().rev() {
        delete_queue_index(config, index).await?;
    }

    Ok(())
}

pub async fn ensure_queued_after_current(config: &AppConfig, video_id: &str) -> Result<()> {
    if queue_contains_video(config, video_id).await? {
        return Ok(());
    }

    enqueue_after_current(config, video_id).await
}

pub async fn select_video_from_queue(config: &AppConfig, video_id: &str) -> Result<bool> {
    let mut index = None;
    for _ in 0..30 {
        index = queue_video_ids(config)
            .await?
            .into_iter()
            .position(|id| id == video_id);
        if index.is_some() {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }

    let Some(index) = index else {
        return Ok(false);
    };

    let response = client()?
        .patch(endpoint(config, "queue"))
        .json(&PearQueueIndexRequest { index })
        .send()
        .await
        .context("Pear nao respondeu em PATCH /queue")?;
    let status = response.status();
    if status.is_success() {
        return Ok(true);
    }

    let body = response.text().await.unwrap_or_default();
    bail!(
        "Pear nao aceitou selecionar o indice {index} da fila ({status}): {}",
        body.trim()
    );
}

async fn queue_contains_video(config: &AppConfig, video_id: &str) -> Result<bool> {
    Ok(queue_video_ids(config)
        .await?
        .into_iter()
        .any(|id| id == video_id))
}

async fn queue_video_ids(config: &AppConfig) -> Result<Vec<String>> {
    Ok(queue_items(config)
        .await?
        .into_iter()
        .filter_map(|item| item.video_id)
        .collect())
}

async fn queue_items(config: &AppConfig) -> Result<Vec<PearQueueItem>> {
    let response = client()?
        .get(endpoint(config, "queue"))
        .send()
        .await
        .context("Pear nao respondeu em /queue")?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("Pear /queue falhou com {status}: {}", body.trim());
    }

    let queue = response.json::<PearQueueInfo>().await?;
    Ok(queue
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| PearQueueItem {
            index,
            video_id: extract_queue_video_id(item),
            selected: extract_queue_selected(item),
        })
        .collect())
}

async fn delete_queue_index(config: &AppConfig, index: usize) -> Result<()> {
    let response = client()?
        .delete(endpoint(config, &format!("queue/{index}")))
        .send()
        .await
        .with_context(|| format!("Pear nao respondeu em DELETE /queue/{index}"))?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let body = response.text().await.unwrap_or_default();
    bail!(
        "Pear nao aceitou remover item {index} da fila ({status}): {}",
        body.trim()
    );
}

pub async fn current_volume(config: &AppConfig) -> Result<PearVolume> {
    let response = client()?
        .get(endpoint(config, "volume"))
        .send()
        .await
        .context("Pear nao respondeu em /volume")?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("Pear /volume falhou com {status}: {}", body.trim());
    }

    Ok(response.json::<PearVolume>().await?)
}

pub async fn set_volume(config: &AppConfig, level: u8) -> Result<u8> {
    let level = level.min(100);
    let response = client()?
        .post(endpoint(config, "volume"))
        .json(&PearVolumeRequest { volume: level })
        .send()
        .await
        .context("Pear nao respondeu em /volume")?;
    let status = response.status();
    if status.is_success() {
        return Ok(level);
    }

    let body = response.text().await.unwrap_or_default();
    bail!("Pear nao aceitou mudar volume ({status}): {}", body.trim());
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

fn extract_queue_video_id(item: &Value) -> Option<String> {
    item.pointer("/videoId")
        .or_else(|| item.pointer("/playlistPanelVideoRenderer/videoId"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
}

fn extract_queue_selected(item: &Value) -> bool {
    item.pointer("/selected")
        .or_else(|| item.pointer("/playlistPanelVideoRenderer/selected"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_direct_queue_video_id() {
        let item = json!({ "videoId": "abc123" });

        assert_eq!(extract_queue_video_id(&item).as_deref(), Some("abc123"));
    }

    #[test]
    fn extracts_renderer_queue_video_id() {
        let item = json!({
            "playlistPanelVideoRenderer": {
                "title": { "runs": [{ "text": "ATWA" }] },
                "selected": true,
                "videoId": "Ph8Qt3DHVwo"
            }
        });

        assert_eq!(
            extract_queue_video_id(&item).as_deref(),
            Some("Ph8Qt3DHVwo")
        );
        assert!(extract_queue_selected(&item));
    }
}
