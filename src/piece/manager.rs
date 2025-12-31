use super::{PieceInfo, PieceState, BLOCK_SIZE};
use crate::error::{BittorrentError, Result};
use crate::torrent::Pieces;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Manages piece download and verification
pub struct PieceManager {
    piece_length: u64,
    total_length: u64,
    pieces: Vec<PieceInfo>,
    /// In-progress piece data
    downloading: HashMap<usize, Vec<u8>>,
}

impl PieceManager {
    pub fn new(piece_length: u64, total_length: u64, piece_hashes: &Pieces) -> Self {
        let num_pieces = piece_hashes.len();
        let mut pieces = Vec::with_capacity(num_pieces);

        for (index, hash) in piece_hashes.iter().enumerate() {
            let length = if index == num_pieces - 1 {
                // Last piece might be smaller
                let remainder = total_length % piece_length;
                if remainder == 0 {
                    piece_length
                } else {
                    remainder
                }
            } else {
                piece_length
            };

            pieces.push(PieceInfo {
                index,
                length,
                state: PieceState::Missing,
                hash: *hash.as_bytes(),
            });
        }

        Self {
            piece_length,
            total_length,
            pieces,
            downloading: HashMap::new(),
        }
    }

    /// Start downloading a piece
    pub fn start_piece(&mut self, piece_index: usize) -> Result<()> {
        if piece_index >= self.pieces.len() {
            return Err(BittorrentError::PieceError("Invalid piece index".to_string()));
        }

        let piece = &mut self.pieces[piece_index];
        if piece.state != PieceState::Missing {
            return Err(BittorrentError::PieceError(
                "Piece already downloading or complete".to_string(),
            ));
        }

        piece.state = PieceState::Downloading;
        self.downloading.insert(piece_index, vec![0u8; piece.length as usize]);

        debug!("Started downloading piece {}", piece_index);
        Ok(())
    }

    /// Add a block to a piece
    pub fn add_block(&mut self, piece_index: usize, offset: u32, data: &[u8]) -> Result<()> {
        let piece_data = self.downloading.get_mut(&piece_index).ok_or_else(|| {
            BittorrentError::PieceError("Piece not being downloaded".to_string())
        })?;

        let offset = offset as usize;
        if offset + data.len() > piece_data.len() {
            return Err(BittorrentError::PieceError("Block exceeds piece size".to_string()));
        }

        piece_data[offset..offset + data.len()].copy_from_slice(data);

        debug!(
            "Added block to piece {} at offset {} ({} bytes)",
            piece_index,
            offset,
            data.len()
        );

        Ok(())
    }

    /// Verify and complete a piece
    pub fn complete_piece(&mut self, piece_index: usize) -> Result<Vec<u8>> {
        let piece_data = self.downloading.remove(&piece_index).ok_or_else(|| {
            BittorrentError::PieceError("Piece not being downloaded".to_string())
        })?;

        let piece = &self.pieces[piece_index];

        // Verify SHA1 hash
        let mut hasher = Sha1::new();
        hasher.update(&piece_data);
        let hash = hasher.finalize();

        if hash.as_slice() != piece.hash {
            warn!("Piece {} failed verification", piece_index);
            self.pieces[piece_index].state = PieceState::Missing;
            return Err(BittorrentError::PieceError(
                "Piece hash verification failed".to_string(),
            ));
        }

        self.pieces[piece_index].state = PieceState::Complete;
        info!("Piece {} verified and complete", piece_index);

        Ok(piece_data)
    }

    /// Get the number of blocks in a piece
    pub fn blocks_in_piece(&self, piece_index: usize) -> usize {
        if piece_index >= self.pieces.len() {
            return 0;
        }

        let piece_length = self.pieces[piece_index].length;
        ((piece_length + BLOCK_SIZE as u64 - 1) / BLOCK_SIZE as u64) as usize
    }

    /// Get block info for a piece
    pub fn get_block_info(&self, piece_index: usize, block_index: usize) -> Option<(u32, u32)> {
        if piece_index >= self.pieces.len() {
            return None;
        }

        let piece = &self.pieces[piece_index];
        let offset = (block_index as u32) * BLOCK_SIZE;

        if offset >= piece.length as u32 {
            return None;
        }

        let length = std::cmp::min(
            BLOCK_SIZE,
            piece.length as u32 - offset,
        );

        Some((offset, length))
    }

    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    pub fn complete_count(&self) -> usize {
        self.pieces.iter().filter(|p| p.state == PieceState::Complete).count()
    }

    pub fn progress(&self) -> f64 {
        (self.complete_count() as f64 / self.piece_count() as f64) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.complete_count() == self.piece_count()
    }

    pub fn get_piece_state(&self, piece_index: usize) -> Option<PieceState> {
        self.pieces.get(piece_index).map(|p| p.state)
    }
}
