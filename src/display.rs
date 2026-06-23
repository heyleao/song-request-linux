use crate::song_requests::{RequestSource, SongRequest};

const GENERIC_ARTISTS: &[&str] = &["spotify", "youtube", "spotify search", "youtube search"];

pub fn chat_song_title(song: &SongRequest) -> String {
    if matches!(song.source, RequestSource::Spotify { .. }) {
        return song.title.trim().to_string();
    }

    compact_track(&song.title, &song.artist)
}

fn compact_track(title: &str, artist: &str) -> String {
    let title = clean_part(title);
    let artist = clean_part(artist);
    let artist_is_generic = is_generic_artist(&artist);

    if let Some((parsed_artist, parsed_title)) = split_title(&title) {
        if !artist_is_generic && same_text(&parsed_title, &artist) {
            return clean_part(&format!("{artist} - {parsed_artist}"));
        }
        if artist_is_generic || parsed_artist.chars().count() <= 42 {
            return clean_part(&format!("{parsed_artist} - {parsed_title}"));
        }
    }

    if !artist.is_empty() && !artist_is_generic {
        clean_part(&format!("{artist} - {title}"))
    } else {
        title
    }
}

fn split_title(value: &str) -> Option<(String, String)> {
    for separator in [" - ", " – ", " — ", " | "] {
        let parts = value
            .split(separator)
            .map(clean_part)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.len() >= 2 {
            return Some((parts[0].clone(), parts[1..].join(" - ")));
        }
    }
    None
}

fn clean_part(value: &str) -> String {
    let text = remove_noise_parentheses(&remove_bracketed(value, '[', ']'));
    let mut words = Vec::new();
    for word in text.split_whitespace() {
        let normalized = normalize_word(word);
        if is_noise_word(&normalized) {
            continue;
        }
        words.push(word.trim_matches(['(', ')', '[', ']']));
    }

    words
        .into_iter()
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(['-', '–', '—', '|', ' '])
        .trim()
        .to_string()
}

fn remove_bracketed(value: &str, open: char, close: char) -> String {
    let mut output = String::with_capacity(value.len());
    let mut depth = 0u32;
    for ch in value.chars() {
        if ch == open {
            depth = depth.saturating_add(1);
            output.push(' ');
        } else if ch == close && depth > 0 {
            depth -= 1;
            output.push(' ');
        } else if depth == 0 {
            output.push(ch);
        }
    }
    output
}

fn remove_noise_parentheses(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut buffer = String::new();
    let mut in_parentheses = false;

    for ch in value.chars() {
        if ch == '(' && !in_parentheses {
            in_parentheses = true;
            buffer.clear();
            continue;
        }

        if ch == ')' && in_parentheses {
            in_parentheses = false;
            if contains_noise(&buffer) {
                output.push(' ');
            } else {
                output.push('(');
                output.push_str(&buffer);
                output.push(')');
            }
            continue;
        }

        if in_parentheses {
            buffer.push(ch);
        } else {
            output.push(ch);
        }
    }

    if in_parentheses {
        output.push('(');
        output.push_str(&buffer);
    }

    output
}

fn contains_noise(value: &str) -> bool {
    value
        .split_whitespace()
        .map(normalize_word)
        .any(|word| is_noise_word(&word))
}

fn normalize_word(value: &str) -> String {
    value
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
        .to_ascii_lowercase()
}

fn is_noise_word(value: &str) -> bool {
    matches!(
        value,
        "official"
            | "video"
            | "audio"
            | "lyrics"
            | "lyric"
            | "visualizer"
            | "remastered"
            | "remaster"
            | "hd"
            | "4k"
            | "upgrade"
    )
}

fn is_generic_artist(value: &str) -> bool {
    GENERIC_ARTISTS
        .iter()
        .any(|artist| value.eq_ignore_ascii_case(artist))
}

fn same_text(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song_requests::{MusicProvider, RequestSource};

    #[test]
    fn cleans_youtube_video_title_for_chat() {
        let song = SongRequest {
            id: 1,
            requester: "viewer".to_string(),
            query: "test".to_string(),
            source: RequestSource::Youtube {
                video_id: "abc123".to_string(),
            },
            title: "Breaking the Habit (Official Music Video) [HD UPGRADE] – Linkin Park"
                .to_string(),
            artist: "Linkin Park".to_string(),
        };

        assert_eq!(chat_song_title(&song), "Linkin Park - Breaking the Habit");
    }

    #[test]
    fn keeps_spotify_title_unchanged() {
        let song = SongRequest {
            id: 1,
            requester: "viewer".to_string(),
            query: "test".to_string(),
            source: RequestSource::Spotify {
                uri: "spotify:track:test".to_string(),
            },
            title: "One More Time".to_string(),
            artist: "Daft Punk".to_string(),
        };

        assert_eq!(chat_song_title(&song), "One More Time");
    }

    #[test]
    fn uses_search_title_when_artist_is_generic() {
        let song = SongRequest {
            id: 1,
            requester: "viewer".to_string(),
            query: "test".to_string(),
            source: RequestSource::Search {
                provider: MusicProvider::Youtube,
            },
            title: "System Of A Down - Spiders".to_string(),
            artist: "YouTube search".to_string(),
        };

        assert_eq!(chat_song_title(&song), "System Of A Down - Spiders");
    }
}
