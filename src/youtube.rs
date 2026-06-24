use std::collections::HashSet;

use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::debug;

use crate::config::AppConfig;

const API_URL: &str = "https://www.googleapis.com/youtube/v3";
const MUSIC_CATEGORY_ID: &str = "10";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YoutubeVideoRef {
    pub video_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YoutubeVideoMetadata {
    pub video_id: String,
    pub title: String,
    pub channel_title: String,
    pub duration_seconds: u64,
    pub category_id: String,
}

#[derive(Debug, Deserialize)]
struct VideosResponse {
    items: Vec<VideoItem>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    id: SearchItemId,
}

#[derive(Debug, Deserialize)]
struct SearchItemId {
    #[serde(default, rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VideoItem {
    id: String,
    snippet: VideoSnippet,
    #[serde(rename = "contentDetails")]
    content_details: VideoContentDetails,
}

#[derive(Debug, Deserialize)]
struct VideoSnippet {
    title: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
    #[serde(rename = "categoryId")]
    category_id: String,
}

#[derive(Debug, Deserialize)]
struct VideoContentDetails {
    duration: String,
}

impl YoutubeVideoRef {
    pub fn parse(value: &str) -> Option<Self> {
        let trimmed = value.trim();

        parse_youtu_be(trimmed)
            .or_else(|| parse_watch_url(trimmed))
            .filter(|video_id| is_valid_video_id(video_id))
            .map(|video_id| Self { video_id })
    }
}

pub async fn validate_video(
    config: &AppConfig,
    video: &YoutubeVideoRef,
) -> Result<YoutubeVideoMetadata> {
    let metadata = fetch_video_metadata(config, &video.video_id).await?;

    if metadata.duration_seconds > config.youtube.max_duration_seconds {
        bail!(
            "Video YouTube bloqueado: {} tem {}, limite atual e {}.",
            metadata.title,
            format_duration(metadata.duration_seconds),
            format_duration(config.youtube.max_duration_seconds)
        );
    }

    if !config.youtube.allow_non_music && metadata.category_id != MUSIC_CATEGORY_ID {
        bail!(
            "Video YouTube bloqueado: {} nao esta marcado como Musica no YouTube. Ative aceitar nao-musica para liberar excecoes.",
            metadata.title
        );
    }

    Ok(metadata)
}

pub async fn search_and_validate(config: &AppConfig, query: &str) -> Result<YoutubeVideoMetadata> {
    let candidates = search_videos(config, query).await?;
    let mut last_error = None;
    let mut valid_candidates = Vec::new();

    for video_id in candidates {
        match validate_video(config, &YoutubeVideoRef { video_id }).await {
            Ok(metadata) => valid_candidates.push(metadata),
            Err(error) => last_error = Some(error),
        }
    }

    choose_best_video(query, valid_candidates).ok_or_else(|| {
        last_error.unwrap_or_else(|| anyhow!("Nenhum video YouTube encontrado para {query}"))
    })
}

async fn search_videos(config: &AppConfig, query: &str) -> Result<Vec<String>> {
    let api_key = youtube_api_key(config)?;
    let response = Client::new()
        .get(format!("{API_URL}/search"))
        .query(&[
            ("part", "snippet"),
            ("type", "video"),
            ("maxResults", "5"),
            ("q", query.trim()),
            ("key", api_key),
        ])
        .send()
        .await
        .context("failed to search YouTube videos")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("YouTube search failed with {status}: {body}");
    }

    let video_ids = response
        .json::<SearchResponse>()
        .await
        .context("failed to decode YouTube search response")?
        .items
        .into_iter()
        .filter_map(|item| item.id.video_id)
        .filter(|video_id| is_valid_video_id(video_id))
        .collect::<Vec<_>>();

    if video_ids.is_empty() {
        bail!("Nenhum video YouTube encontrado para {}", query.trim());
    }

    Ok(video_ids)
}

async fn fetch_video_metadata(config: &AppConfig, video_id: &str) -> Result<YoutubeVideoMetadata> {
    let api_key = youtube_api_key(config)?;
    let response = Client::new()
        .get(format!("{API_URL}/videos"))
        .query(&[
            ("part", "snippet,contentDetails"),
            ("id", video_id),
            ("key", api_key),
        ])
        .send()
        .await
        .context("failed to fetch YouTube video metadata")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("YouTube metadata failed with {status}: {body}");
    }

    let item = response
        .json::<VideosResponse>()
        .await
        .context("failed to decode YouTube metadata response")?
        .items
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("Video YouTube nao encontrado ou indisponivel"))?;
    let duration_seconds =
        parse_iso8601_duration(&item.content_details.duration).ok_or_else(|| {
            anyhow!(
                "Duracao YouTube invalida: {}",
                item.content_details.duration
            )
        })?;

    Ok(YoutubeVideoMetadata {
        video_id: item.id,
        title: item.snippet.title,
        channel_title: item.snippet.channel_title,
        duration_seconds,
        category_id: item.snippet.category_id,
    })
}

fn youtube_api_key(config: &AppConfig) -> Result<&str> {
    config
        .youtube
        .api_key
        .as_deref()
        .context("YouTube API key nao configurada; configure para validar duracao/categoria")
}

fn parse_iso8601_duration(value: &str) -> Option<u64> {
    let chars = value.strip_prefix('P')?.chars().peekable();
    let mut in_time = false;
    let mut number = String::new();
    let mut seconds = 0u64;

    for ch in chars {
        if ch == 'T' {
            in_time = true;
            continue;
        }
        if ch.is_ascii_digit() {
            number.push(ch);
            continue;
        }

        let amount = number.parse::<u64>().ok()?;
        number.clear();
        match ch {
            'D' => seconds += amount * 86_400,
            'H' if in_time => seconds += amount * 3_600,
            'M' if in_time => seconds += amount * 60,
            'S' if in_time => seconds += amount,
            _ => return None,
        }
    }

    if number.is_empty() {
        Some(seconds)
    } else {
        None
    }
}

fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{minutes}:{seconds:02}")
}

fn parse_youtu_be(value: &str) -> Option<String> {
    let marker = "youtu.be/";
    let start = value.find(marker)? + marker.len();
    let id = value[start..]
        .split(['?', '&', '/', '#'])
        .next()
        .unwrap_or_default();

    Some(id.to_string())
}

fn parse_watch_url(value: &str) -> Option<String> {
    if !value.contains("youtube.com/") && !value.contains("music.youtube.com/") {
        return None;
    }

    let query = value.split_once('?')?.1;

    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == "v" {
            return Some(value.to_string());
        }
    }

    None
}

fn is_valid_video_id(id: &str) -> bool {
    id.len() == 11
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn choose_best_video(
    query: &str,
    videos: Vec<YoutubeVideoMetadata>,
) -> Option<YoutubeVideoMetadata> {
    let query_tokens = tokenize(query);
    let mut ranked = videos
        .into_iter()
        .enumerate()
        .map(|(index, video)| {
            let score = score_video(&query_tokens, &video, index);
            debug!(
                score,
                title = %video.title,
                channel = %video.channel_title,
                "youtube search candidate"
            );
            (score, video)
        })
        .collect::<Vec<_>>();

    ranked.sort_by_key(|item| std::cmp::Reverse(item.0));
    ranked.into_iter().map(|(_, video)| video).next()
}

fn score_video(query_tokens: &[String], video: &YoutubeVideoMetadata, index: usize) -> i64 {
    let title_tokens = tokenize(&video.title);
    let channel_tokens = tokenize(&video.channel_title);
    let combined_tokens = title_tokens
        .iter()
        .chain(channel_tokens.iter())
        .cloned()
        .collect::<HashSet<_>>();
    let title_set = title_tokens.iter().cloned().collect::<HashSet<_>>();
    let channel_set = channel_tokens.iter().cloned().collect::<HashSet<_>>();

    let mut score = 0;
    for token in query_tokens {
        if title_set.contains(token) {
            score += 14;
        }
        if channel_set.contains(token) {
            score += 8;
        }
        if combined_tokens.contains(token) {
            score += 4;
        } else {
            score -= 20;
        }
    }

    let query_joined = query_tokens.join(" ");
    let title_joined = title_tokens.join(" ");
    let channel_joined = channel_tokens.join(" ");
    let combined_joined = format!("{channel_joined} {title_joined}");

    if title_joined == query_joined {
        score += 90;
    }
    if combined_joined == query_joined {
        score += 120;
    }
    if combined_joined.contains(&query_joined) {
        score += 35;
    }
    if title_joined.starts_with(&query_joined) {
        score += 25;
    }
    if !query_tokens.is_empty()
        && query_tokens
            .iter()
            .all(|token| combined_tokens.contains(token))
    {
        score += 65;
    }

    if video.category_id == MUSIC_CATEGORY_ID {
        score += 35;
    } else {
        score -= 45;
    }

    if channel_joined.contains("vevo") || channel_joined.contains("topic") {
        score += 20;
    }

    for noise in [
        "aovivo", "live", "faustao", "cover", "karaoke", "react", "reaction",
    ] {
        if !query_tokens.iter().any(|token| token == noise)
            && (title_joined.contains(noise) || channel_joined.contains(noise))
        {
            score -= 30;
        }
    }

    score - index as i64
}

fn tokenize(value: &str) -> Vec<String> {
    normalize(value)
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_music_category_above_unrequested_live_clip() {
        let videos = vec![
            YoutubeVideoMetadata {
                video_id: "live".to_string(),
                title: "Mamonas Assassinas- Robocop Gay (Ao vivo no Faustao) HD".to_string(),
                channel_title: "Falando de Mamonas".to_string(),
                duration_seconds: 210,
                category_id: "22".to_string(),
            },
            YoutubeVideoMetadata {
                video_id: "music".to_string(),
                title: "Mamonas Assassinas - Pelados em Santos (Videoclipe)".to_string(),
                channel_title: "Musicalidade".to_string(),
                duration_seconds: 200,
                category_id: MUSIC_CATEGORY_ID.to_string(),
            },
        ];

        let selected = choose_best_video("mamonas assassinas", videos).expect("selected");

        assert_eq!(selected.video_id, "music");
    }

    #[test]
    fn parses_short_youtube_url() {
        let video = YoutubeVideoRef::parse("https://youtu.be/dQw4w9WgXcQ?t=10").expect("video");

        assert_eq!(video.video_id, "dQw4w9WgXcQ");
    }

    #[test]
    fn parses_watch_url() {
        let video =
            YoutubeVideoRef::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").expect("video");

        assert_eq!(video.video_id, "dQw4w9WgXcQ");
    }

    #[test]
    fn rejects_invalid_video_id() {
        assert!(YoutubeVideoRef::parse("https://youtu.be/not-valid").is_none());
    }

    #[test]
    fn parses_youtube_duration() {
        assert_eq!(parse_iso8601_duration("PT6M"), Some(360));
        assert_eq!(parse_iso8601_duration("PT5M32S"), Some(332));
        assert_eq!(parse_iso8601_duration("PT1H2M3S"), Some(3723));
    }

    #[test]
    fn ranking_prefers_requested_title_over_first_result() {
        let videos = vec![
            YoutubeVideoMetadata {
                video_id: "aaaaaaaaaaa".to_string(),
                title: "System Of A Down - Aerials".to_string(),
                channel_title: "systemofadownVEVO".to_string(),
                duration_seconds: 240,
                category_id: MUSIC_CATEGORY_ID.to_string(),
            },
            YoutubeVideoMetadata {
                video_id: "bbbbbbbbbbb".to_string(),
                title: "System Of A Down - Spiders".to_string(),
                channel_title: "System Of A Down".to_string(),
                duration_seconds: 215,
                category_id: MUSIC_CATEGORY_ID.to_string(),
            },
        ];

        let video = choose_best_video("system of a down spiders", videos).expect("video");

        assert_eq!(video.video_id, "bbbbbbbbbbb");
    }
}
