use crate::bencode::BencodeValue;
use crate::error::{BittorrentError, Result};
use super::Peer;
use std::net::IpAddr;

/// Response from a tracker
#[derive(Debug, Clone)]
pub struct TrackerResponse {
    /// Interval in seconds to wait before next request
    pub interval: u64,
    /// Minimum announce interval (optional)
    pub min_interval: Option<u64>,
    /// Tracker ID (optional)
    pub tracker_id: Option<String>,
    /// Number of seeders (optional)
    pub complete: Option<u64>,
    /// Number of leechers (optional)
    pub incomplete: Option<u64>,
    /// List of peers
    pub peers: Vec<Peer>,
}

impl TrackerResponse {
    pub fn from_bencode(value: BencodeValue) -> Result<Self> {
        let dict = value.as_dict().ok_or_else(|| {
            BittorrentError::TrackerError("Response must be a dict".to_string())
        })?;

        // Check for failure reason
        if let Some(failure) = dict.get(b"failure reason".as_ref()) {
            let reason = failure
                .as_str()
                .unwrap_or("Unknown failure")
                .to_string();
            return Err(BittorrentError::TrackerError(reason));
        }

        // Parse interval (required)
        let interval = dict
            .get(b"interval".as_ref())
            .and_then(|v| v.as_integer())
            .ok_or_else(|| {
                BittorrentError::TrackerError("Missing 'interval' field".to_string())
            })? as u64;

        // Parse optional fields
        let min_interval = dict
            .get(b"min interval".as_ref())
            .and_then(|v| v.as_integer())
            .map(|i| i as u64);

        let tracker_id = dict
            .get(b"tracker id".as_ref())
            .and_then(|v| v.as_str())
            .map(String::from);

        let complete = dict
            .get(b"complete".as_ref())
            .and_then(|v| v.as_integer())
            .map(|i| i as u64);

        let incomplete = dict
            .get(b"incomplete".as_ref())
            .and_then(|v| v.as_integer())
            .map(|i| i as u64);

        // Parse peers
        let peers = if let Some(peers_value) = dict.get(b"peers".as_ref()) {
            // Try compact format first (binary string)
            if let Some(compact_peers) = peers_value.as_bytes() {
                Peer::from_compact_list(compact_peers)
            } else if let Some(peer_list) = peers_value.as_list() {
                // Dictionary model
                parse_peer_list(peer_list)?
            } else {
                return Err(BittorrentError::TrackerError(
                    "Invalid 'peers' format".to_string(),
                ));
            }
        } else {
            return Err(BittorrentError::TrackerError(
                "Missing 'peers' field".to_string(),
            ));
        };

        Ok(TrackerResponse {
            interval,
            min_interval,
            tracker_id,
            complete,
            incomplete,
            peers,
        })
    }
}

fn parse_peer_list(list: &[BencodeValue]) -> Result<Vec<Peer>> {
    let mut peers = Vec::new();

    for peer_value in list {
        let peer_dict = peer_value.as_dict().ok_or_else(|| {
            BittorrentError::TrackerError("Peer must be a dict".to_string())
        })?;

        // Parse IP
        let ip_str = peer_dict
            .get(b"ip".as_ref())
            .and_then(|v| v.as_str())
            .ok_or_else(|| BittorrentError::TrackerError("Missing peer 'ip'".to_string()))?;

        let ip: IpAddr = ip_str.parse().map_err(|_| {
            BittorrentError::TrackerError("Invalid peer IP address".to_string())
        })?;

        // Parse port
        let port = peer_dict
            .get(b"port".as_ref())
            .and_then(|v| v.as_integer())
            .ok_or_else(|| BittorrentError::TrackerError("Missing peer 'port'".to_string()))?
            as u16;

        // Parse peer_id (optional)
        let peer_id = peer_dict
            .get(b"peer id".as_ref())
            .and_then(|v| v.as_bytes())
            .map(|b| b.to_vec());

        let peer = if let Some(id) = peer_id {
            Peer::with_peer_id(ip, port, id)
        } else {
            Peer::new(ip, port)
        };

        peers.push(peer);
    }

    Ok(peers)
}
