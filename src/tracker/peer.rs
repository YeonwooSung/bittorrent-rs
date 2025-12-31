use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Represents a peer in the swarm
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Peer {
    pub addr: SocketAddr,
    pub peer_id: Option<Vec<u8>>,
}

impl Peer {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            addr: SocketAddr::new(ip, port),
            peer_id: None,
        }
    }

    pub fn with_peer_id(ip: IpAddr, port: u16, peer_id: Vec<u8>) -> Self {
        Self {
            addr: SocketAddr::new(ip, port),
            peer_id: Some(peer_id),
        }
    }

    /// Parse a peer from compact format (6 bytes: 4 IP + 2 port)
    pub fn from_compact(data: &[u8]) -> Option<Self> {
        if data.len() != 6 {
            return None;
        }

        let ip = Ipv4Addr::new(data[0], data[1], data[2], data[3]);
        let port = u16::from_be_bytes([data[4], data[5]]);

        Some(Self::new(IpAddr::V4(ip), port))
    }

    /// Parse multiple peers from compact format
    pub fn from_compact_list(data: &[u8]) -> Vec<Self> {
        data.chunks_exact(6)
            .filter_map(Self::from_compact)
            .collect()
    }
}
