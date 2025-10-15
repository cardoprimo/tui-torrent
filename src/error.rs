use std::fmt;

#[derive(Debug)]
pub enum TorrentError {
    Network(reqwest::Error),
    Database(rusqlite::Error),
    Parse(String),
    NotFound(String),
    Aria2(String),
}

impl fmt::Display for TorrentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TorrentError::Network(e) => write!(f, "Network error: {}", e),
            TorrentError::Database(e) => write!(f, "Database error: {}", e),
            TorrentError::Parse(msg) => write!(f, "Parse error: {}", msg),
            TorrentError::NotFound(msg) => write!(f, "Not found: {}", msg),
            TorrentError::Aria2(msg) => write!(f, "Aria2 error: {}", msg),
        }
    }
}

impl std::error::Error for TorrentError {}

impl From<reqwest::Error> for TorrentError {
    fn from(error: reqwest::Error) -> Self {
        TorrentError::Network(error)
    }
}

impl From<rusqlite::Error> for TorrentError {
    fn from(error: rusqlite::Error) -> Self {
        TorrentError::Database(error)
    }
}

pub type Result<T> = std::result::Result<T, TorrentError>;
