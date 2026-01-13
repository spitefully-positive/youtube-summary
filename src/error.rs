use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidYoutubeUrl(String),
    TranscriptFetch(String),
    ApiRequest(String),
    Config(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidYoutubeUrl(msg) => write!(f, "Invalid YouTube URL: {}", msg),
            Error::TranscriptFetch(msg) => write!(f, "Failed to fetch transcript: {}", msg),
            Error::ApiRequest(msg) => write!(f, "API request failed: {}", msg),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
