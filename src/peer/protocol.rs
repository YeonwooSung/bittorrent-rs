use crate::error::{BittorrentError, Result};

pub const PROTOCOL_STRING: &[u8] = b"BitTorrent protocol";

/// Handshake message for peer wire protocol
/// Format: <pstrlen><pstr><reserved><info_hash><peer_id>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handshake {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            info_hash,
            peer_id,
        }
    }

    /// Serialize handshake to bytes
    /// Format: <pstrlen><pstr><reserved><info_hash><peer_id>
    /// Total: 1 + 19 + 8 + 20 + 20 = 68 bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(68);

        // Protocol string length
        buf.push(PROTOCOL_STRING.len() as u8);

        // Protocol string
        buf.extend_from_slice(PROTOCOL_STRING);

        // Reserved bytes (8 bytes, all zeros)
        buf.extend_from_slice(&[0u8; 8]);

        // Info hash
        buf.extend_from_slice(&self.info_hash);

        // Peer ID
        buf.extend_from_slice(&self.peer_id);

        buf
    }

    /// Deserialize handshake from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 68 {
            return Err(BittorrentError::PeerError(
                "Handshake too short".to_string(),
            ));
        }

        // Check protocol string length
        let pstrlen = data[0] as usize;
        if pstrlen != PROTOCOL_STRING.len() {
            return Err(BittorrentError::PeerError(
                "Invalid protocol string length".to_string(),
            ));
        }

        // Check protocol string
        if &data[1..1 + pstrlen] != PROTOCOL_STRING {
            return Err(BittorrentError::PeerError(
                "Invalid protocol string".to_string(),
            ));
        }

        // Extract info hash
        let mut info_hash = [0u8; 20];
        info_hash.copy_from_slice(&data[28..48]);

        // Extract peer ID
        let mut peer_id = [0u8; 20];
        peer_id.copy_from_slice(&data[48..68]);

        Ok(Handshake {
            info_hash,
            peer_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_serialization() {
        let info_hash = [1u8; 20];
        let peer_id = [2u8; 20];

        let handshake = Handshake::new(info_hash, peer_id);
        let bytes = handshake.to_bytes();

        assert_eq!(bytes.len(), 68);
        assert_eq!(bytes[0], 19); // pstrlen
        assert_eq!(&bytes[1..20], PROTOCOL_STRING);

        let decoded = Handshake::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, handshake);
    }
}
