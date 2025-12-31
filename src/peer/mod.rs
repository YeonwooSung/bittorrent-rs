mod connection;
mod message;
mod protocol;

pub use connection::PeerConnection;
pub use message::{PeerMessage, BlockInfo};
pub use protocol::{Handshake, PROTOCOL_STRING};

// Peer connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerState {
    /// Whether we are choking the peer
    pub am_choking: bool,
    /// Whether we are interested in the peer
    pub am_interested: bool,
    /// Whether the peer is choking us
    pub peer_choking: bool,
    /// Whether the peer is interested in us
    pub peer_interested: bool,
}

impl Default for PeerState {
    fn default() -> Self {
        Self {
            am_choking: true,
            am_interested: false,
            peer_choking: true,
            peer_interested: false,
        }
    }
}
