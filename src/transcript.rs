use crate::error::{Error, Result};
use yt_transcript_rs::api::YouTubeTranscriptApi;

pub async fn fetch_transcript(url: &str) -> Result<String> {
    let video_id = extract_video_id(url)?;

    let api = YouTubeTranscriptApi::new(None, None, None)
        .map_err(|e| Error::TranscriptFetch(format!("Failed to create API client: {}", e)))?;

    let transcripts = api
        .fetch_transcript(&video_id, &["en"], true)
        .await
        .map_err(|e| Error::TranscriptFetch(format!("Failed to fetch transcript: {}", e)))?;

    // Combine all transcript segments into a single string
    let text: String = transcripts
        .snippets
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    if text.is_empty() {
        return Err(Error::TranscriptFetch("Transcript is empty".to_string()));
    }

    Ok(text)
}

fn extract_video_id(url: &str) -> Result<String> {
    // Handle various YouTube URL formats:
    // - https://www.youtube.com/watch?v=VIDEO_ID
    // - https://youtube.com/watch?v=VIDEO_ID
    // - https://youtu.be/VIDEO_ID
    // - https://www.youtube.com/embed/VIDEO_ID
    // - VIDEO_ID (direct ID)

    let url = url.trim();

    // Check if it's already just a video ID (11 characters, alphanumeric + - + _)
    if url.len() == 11
        && url
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Ok(url.to_string());
    }

    // Try to extract from various URL formats
    if let Some(id) = extract_from_watch_url(url) {
        return Ok(id);
    }

    if let Some(id) = extract_from_short_url(url) {
        return Ok(id);
    }

    if let Some(id) = extract_from_embed_url(url) {
        return Ok(id);
    }

    Err(Error::InvalidYoutubeUrl(format!(
        "Could not extract video ID from: {}",
        url
    )))
}

fn extract_from_watch_url(url: &str) -> Option<String> {
    // https://www.youtube.com/watch?v=VIDEO_ID&other=params
    if url.contains("youtube.com/watch")
        && let Some(v_pos) = url.find("v=")
    {
        let start = v_pos + 2;
        let id: String = url[start..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    None
}

fn extract_from_short_url(url: &str) -> Option<String> {
    // https://youtu.be/VIDEO_ID
    if url.contains("youtu.be/")
        && let Some(pos) = url.find("youtu.be/")
    {
        let start = pos + 9;
        let id: String = url[start..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    None
}

fn extract_from_embed_url(url: &str) -> Option<String> {
    // https://www.youtube.com/embed/VIDEO_ID
    if url.contains("youtube.com/embed/")
        && let Some(pos) = url.find("/embed/")
    {
        let start = pos + 7;
        let id: String = url[start..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    None
}
