mod client;
mod peer;
mod request;
mod response;

pub use client::TrackerClient;
pub use peer::Peer;
pub use request::{TrackerEvent, TrackerRequest};
pub use response::TrackerResponse;

use crate::error::Result;
use rand::Rng;

/// Generate a random peer ID
/// Format: -RS0001-<12 random chars>
pub fn generate_peer_id() -> [u8; 20] {
    let mut peer_id = [0u8; 20];
    peer_id[0..8].copy_from_slice(b"-RS0001-");

    let mut rng = rand::thread_rng();
    for byte in &mut peer_id[8..] {
        *byte = rng.gen_range(b'0'..=b'z');
    }

    peer_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_peer_id() {
        let peer_id = generate_peer_id();
        assert_eq!(peer_id.len(), 20);
        assert_eq!(&peer_id[0..8], b"-RS0001-");
    }
}
