mod manager;
mod picker;

pub use manager::PieceManager;
pub use picker::PiecePicker;

/// Standard block size (16 KB)
pub const BLOCK_SIZE: u32 = 16 * 1024;

/// State of a piece
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceState {
    /// Not downloaded yet
    Missing,
    /// Currently downloading
    Downloading,
    /// Downloaded and verified
    Complete,
}

/// Information about a piece
#[derive(Debug, Clone)]
pub struct PieceInfo {
    pub index: usize,
    pub length: u64,
    pub state: PieceState,
    pub hash: [u8; 20],
}
