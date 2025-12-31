use thiserror::Error;

#[derive(Error, Debug)]
pub enum BittorrentError {
    #[error("Bencode parsing error: {0}")]
    BencodeError(String),

    #[error("Invalid torrent file: {0}")]
    InvalidTorrent(String),

    #[error("Tracker error: {0}")]
    TrackerError(String),

    #[error("Peer connection error: {0}")]
    PeerError(String),

    #[error("Piece validation failed: {0}")]
    PieceError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("URL parse error: {0}")]
    UrlParseError(String),
}

impl From<url::ParseError> for BittorrentError {
    fn from(err: url::ParseError) -> Self {
        BittorrentError::UrlParseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, BittorrentError>;
