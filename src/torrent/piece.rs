use crate::error::{BittorrentError, Result};

/// A 20-byte SHA1 hash representing a piece
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceHash([u8; 20]);

impl PieceHash {
    pub fn new(hash: [u8; 20]) -> Self {
        Self(hash)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 20 {
            return Err(BittorrentError::InvalidTorrent(
                "Piece hash must be 20 bytes".to_string(),
            ));
        }
        let mut hash = [0u8; 20];
        hash.copy_from_slice(slice);
        Ok(Self(hash))
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl AsRef<[u8]> for PieceHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Collection of piece hashes
#[derive(Debug, Clone)]
pub struct Pieces {
    hashes: Vec<PieceHash>,
}

impl Pieces {
    /// Parse pieces from concatenated SHA1 hashes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() % 20 != 0 {
            return Err(BittorrentError::InvalidTorrent(
                "Pieces length must be multiple of 20".to_string(),
            ));
        }

        let hashes = data
            .chunks_exact(20)
            .map(PieceHash::from_slice)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { hashes })
    }

    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&PieceHash> {
        self.hashes.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &PieceHash> {
        self.hashes.iter()
    }
}
