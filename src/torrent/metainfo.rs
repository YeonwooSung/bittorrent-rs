use crate::bencode::{encode, BencodeValue};
use crate::error::{BittorrentError, Result};
use super::Pieces;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;

/// Represents a file in a multi-file torrent
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: Vec<String>,
    pub length: u64,
}

/// Information about the torrent contents
#[derive(Debug, Clone)]
pub struct TorrentInfo {
    /// Suggested name for the file or directory
    pub name: String,
    /// Number of bytes in each piece
    pub piece_length: u64,
    /// SHA1 hashes of all pieces
    pub pieces: Pieces,
    /// Files in the torrent
    pub files: Vec<FileInfo>,
    /// Total length of all files
    pub total_length: u64,
}

impl TorrentInfo {
    fn from_bencode(value: &BencodeValue) -> Result<Self> {
        let dict = value
            .as_dict()
            .ok_or_else(|| BittorrentError::InvalidTorrent("Info must be a dict".to_string()))?;

        // Parse name
        let name = dict
            .get(b"name".as_ref())
            .and_then(|v| v.as_str())
            .ok_or_else(|| BittorrentError::InvalidTorrent("Missing 'name' field".to_string()))?
            .to_string();

        // Parse piece length
        let piece_length = dict
            .get(b"piece length".as_ref())
            .and_then(|v| v.as_integer())
            .ok_or_else(|| {
                BittorrentError::InvalidTorrent("Missing 'piece length' field".to_string())
            })? as u64;

        // Parse pieces
        let pieces_bytes = dict
            .get(b"pieces".as_ref())
            .and_then(|v| v.as_bytes())
            .ok_or_else(|| BittorrentError::InvalidTorrent("Missing 'pieces' field".to_string()))?;

        let pieces = Pieces::from_bytes(pieces_bytes)?;

        // Parse files (single-file or multi-file mode)
        let (files, total_length) = if let Some(length_value) = dict.get(b"length".as_ref()) {
            // Single-file mode
            let length = length_value.as_integer().ok_or_else(|| {
                BittorrentError::InvalidTorrent("Invalid 'length' field".to_string())
            })? as u64;

            let file = FileInfo {
                path: vec![name.clone()],
                length,
            };

            (vec![file], length)
        } else if let Some(files_value) = dict.get(b"files".as_ref()) {
            // Multi-file mode
            let files_list = files_value.as_list().ok_or_else(|| {
                BittorrentError::InvalidTorrent("Invalid 'files' field".to_string())
            })?;

            let mut files = Vec::new();
            let mut total = 0u64;

            for file_value in files_list {
                let file_dict = file_value.as_dict().ok_or_else(|| {
                    BittorrentError::InvalidTorrent("File entry must be a dict".to_string())
                })?;

                let length = file_dict
                    .get(b"length".as_ref())
                    .and_then(|v| v.as_integer())
                    .ok_or_else(|| {
                        BittorrentError::InvalidTorrent("Missing file 'length'".to_string())
                    })? as u64;

                let path_list = file_dict
                    .get(b"path".as_ref())
                    .and_then(|v| v.as_list())
                    .ok_or_else(|| {
                        BittorrentError::InvalidTorrent("Missing file 'path'".to_string())
                    })?;

                let path = path_list
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .ok_or_else(|| {
                                BittorrentError::InvalidTorrent("Invalid path component".to_string())
                            })
                            .map(String::from)
                    })
                    .collect::<Result<Vec<_>>>()?;

                total += length;
                files.push(FileInfo { path, length });
            }

            (files, total)
        } else {
            return Err(BittorrentError::InvalidTorrent(
                "Missing 'length' or 'files' field".to_string(),
            ));
        };

        Ok(TorrentInfo {
            name,
            piece_length,
            pieces,
            files,
            total_length,
        })
    }
}

/// Top-level metainfo structure from a .torrent file
#[derive(Debug, Clone)]
pub struct Metainfo {
    /// URL of the tracker
    pub announce: String,
    /// Additional tracker URLs (optional)
    pub announce_list: Option<Vec<Vec<String>>>,
    /// Information about the torrent contents
    pub info: TorrentInfo,
    /// SHA1 hash of the bencoded info dictionary
    pub info_hash: [u8; 20],
}

impl Metainfo {
    pub fn from_bencode(value: BencodeValue, raw_data: &[u8]) -> Result<Self> {
        let dict = value.as_dict().ok_or_else(|| {
            BittorrentError::InvalidTorrent("Torrent must be a dict".to_string())
        })?;

        // Parse announce
        let announce = dict
            .get(b"announce".as_ref())
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                BittorrentError::InvalidTorrent("Missing 'announce' field".to_string())
            })?
            .to_string();

        // Parse announce-list (optional)
        let announce_list = dict.get(b"announce-list".as_ref()).and_then(|v| {
            v.as_list().map(|list| {
                list.iter()
                    .filter_map(|tier| {
                        tier.as_list().map(|urls| {
                            urls.iter()
                                .filter_map(|u| u.as_str().map(String::from))
                                .collect()
                        })
                    })
                    .collect()
            })
        });

        // Parse info
        let info_value = dict
            .get(b"info".as_ref())
            .ok_or_else(|| BittorrentError::InvalidTorrent("Missing 'info' field".to_string()))?;

        let info = TorrentInfo::from_bencode(info_value)?;

        // Calculate info_hash from raw bencoded info dict
        let info_hash = calculate_info_hash(raw_data)?;

        Ok(Metainfo {
            announce,
            announce_list,
            info,
            info_hash,
        })
    }

    /// Get the info hash as a hex string
    pub fn info_hash_hex(&self) -> String {
        hex::encode(self.info_hash)
    }

    /// Get the info hash as a URL-encoded string for tracker requests
    pub fn info_hash_urlencoded(&self) -> String {
        self.info_hash
            .iter()
            .map(|b| format!("%{:02x}", b))
            .collect()
    }
}

/// Calculate the info_hash from the raw torrent data
fn calculate_info_hash(raw_data: &[u8]) -> Result<[u8; 20]> {
    // Find the info dictionary in the raw data
    // We need to find "4:info" and then extract the bencoded dict that follows
    let info_key = b"4:info";
    let info_start = raw_data
        .windows(info_key.len())
        .position(|window| window == info_key)
        .ok_or_else(|| BittorrentError::InvalidTorrent("Info dict not found".to_string()))?
        + info_key.len();

    // Parse the info dict to find its end
    let info_dict_bytes = extract_info_dict(&raw_data[info_start..])?;

    // Calculate SHA1 hash
    let mut hasher = Sha1::new();
    hasher.update(info_dict_bytes);
    let hash = hasher.finalize();

    let mut result = [0u8; 20];
    result.copy_from_slice(&hash);
    Ok(result)
}

/// Extract the bencoded info dictionary bytes
fn extract_info_dict(data: &[u8]) -> Result<&[u8]> {
    if data.is_empty() || data[0] != b'd' {
        return Err(BittorrentError::InvalidTorrent(
            "Info dict must start with 'd'".to_string(),
        ));
    }

    let mut pos = 0;
    let mut depth = 0;

    for (i, &byte) in data.iter().enumerate() {
        match byte {
            b'd' | b'l' => depth += 1,
            b'e' => {
                depth -= 1;
                if depth == 0 {
                    pos = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if pos == 0 {
        return Err(BittorrentError::InvalidTorrent(
            "Unterminated info dict".to_string(),
        ));
    }

    Ok(&data[..pos])
}
