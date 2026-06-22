#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YoutubeVideoRef {
    pub video_id: String,
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
