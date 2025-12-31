use super::PieceState;
use std::collections::HashMap;

/// Selects which pieces to download next
pub struct PiecePicker {
    total_pieces: usize,
    piece_states: Vec<PieceState>,
    /// Tracks how many peers have each piece (for rarest-first)
    piece_availability: Vec<u32>,
}

impl PiecePicker {
    pub fn new(total_pieces: usize) -> Self {
        Self {
            total_pieces,
            piece_states: vec![PieceState::Missing; total_pieces],
            piece_availability: vec![0; total_pieces],
        }
    }

    /// Update peer's bitfield
    pub fn update_peer_pieces(&mut self, bitfield: &[u8]) {
        for piece_index in 0..self.total_pieces {
            if self.has_piece_in_bitfield(bitfield, piece_index) {
                self.piece_availability[piece_index] += 1;
            }
        }
    }

    /// Mark a piece as being downloaded
    pub fn mark_downloading(&mut self, piece_index: usize) {
        if piece_index < self.total_pieces {
            self.piece_states[piece_index] = PieceState::Downloading;
        }
    }

    /// Mark a piece as complete
    pub fn mark_complete(&mut self, piece_index: usize) {
        if piece_index < self.total_pieces {
            self.piece_states[piece_index] = PieceState::Complete;
        }
    }

    /// Mark a piece as missing (e.g., after failed verification)
    pub fn mark_missing(&mut self, piece_index: usize) {
        if piece_index < self.total_pieces {
            self.piece_states[piece_index] = PieceState::Missing;
        }
    }

    /// Pick the next piece to download using rarest-first strategy
    pub fn pick_piece(&self, peer_bitfield: &[u8]) -> Option<usize> {
        let mut best_piece = None;
        let mut lowest_availability = u32::MAX;

        for piece_index in 0..self.total_pieces {
            // Skip if we already have it or are downloading it
            if self.piece_states[piece_index] != PieceState::Missing {
                continue;
            }

            // Skip if peer doesn't have it
            if !self.has_piece_in_bitfield(peer_bitfield, piece_index) {
                continue;
            }

            // Select rarest piece
            let availability = self.piece_availability[piece_index];
            if availability < lowest_availability {
                lowest_availability = availability;
                best_piece = Some(piece_index);
            }
        }

        best_piece
    }

    /// Check if a bitfield indicates the peer has a specific piece
    fn has_piece_in_bitfield(&self, bitfield: &[u8], piece_index: usize) -> bool {
        let byte_index = piece_index / 8;
        let bit_index = 7 - (piece_index % 8);

        if byte_index < bitfield.len() {
            (bitfield[byte_index] >> bit_index) & 1 == 1
        } else {
            false
        }
    }

    /// Get the number of complete pieces
    pub fn complete_count(&self) -> usize {
        self.piece_states
            .iter()
            .filter(|&&s| s == PieceState::Complete)
            .count()
    }

    /// Check if all pieces are complete
    pub fn is_complete(&self) -> bool {
        self.complete_count() == self.total_pieces
    }

    /// Get progress as a percentage
    pub fn progress(&self) -> f64 {
        (self.complete_count() as f64 / self.total_pieces as f64) * 100.0
    }
}
