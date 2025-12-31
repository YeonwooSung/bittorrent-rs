mod metainfo;
mod piece;

pub use metainfo::{FileInfo, Metainfo, TorrentInfo};
pub use piece::{PieceHash, Pieces};

use crate::bencode::{decode, BencodeValue};
use crate::error::{BittorrentError, Result};
use std::path::Path;
use tokio::fs;

/// Load and parse a .torrent file
pub async fn load_torrent_file<P: AsRef<Path>>(path: P) -> Result<Metainfo> {
    let data = fs::read(path).await?;
    parse_torrent(&data)
}

/// Parse torrent data from bytes
pub fn parse_torrent(data: &[u8]) -> Result<Metainfo> {
    let value = decode(data)?;
    Metainfo::from_bencode(value, data)
}
