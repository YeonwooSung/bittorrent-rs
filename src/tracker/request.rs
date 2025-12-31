/// Events sent to the tracker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerEvent {
    Started,
    Stopped,
    Completed,
}

impl TrackerEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrackerEvent::Started => "started",
            TrackerEvent::Stopped => "stopped",
            TrackerEvent::Completed => "completed",
        }
    }
}

/// Request parameters for tracker communication
#[derive(Debug, Clone)]
pub struct TrackerRequest {
    /// SHA1 hash of the info dictionary
    pub info_hash: [u8; 20],
    /// Unique peer ID
    pub peer_id: [u8; 20],
    /// Port this peer is listening on
    pub port: u16,
    /// Total amount uploaded
    pub uploaded: u64,
    /// Total amount downloaded
    pub downloaded: u64,
    /// Number of bytes left to download
    pub left: u64,
    /// Event (optional)
    pub event: Option<TrackerEvent>,
    /// Request compact peer list format
    pub compact: bool,
}

impl TrackerRequest {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20], port: u16, left: u64) -> Self {
        Self {
            info_hash,
            peer_id,
            port,
            uploaded: 0,
            downloaded: 0,
            left,
            event: Some(TrackerEvent::Started),
            compact: true,
        }
    }

    /// Build query parameters for HTTP request
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("info_hash".to_string(), urlencoded_hash(&self.info_hash)),
            ("peer_id".to_string(), urlencoded_hash(&self.peer_id)),
            ("port".to_string(), self.port.to_string()),
            ("uploaded".to_string(), self.uploaded.to_string()),
            ("downloaded".to_string(), self.downloaded.to_string()),
            ("left".to_string(), self.left.to_string()),
            ("compact".to_string(), if self.compact { "1" } else { "0" }.to_string()),
        ];

        if let Some(event) = &self.event {
            params.push(("event".to_string(), event.as_str().to_string()));
        }

        params
    }
}

/// URL-encode a hash for tracker requests
fn urlencoded_hash(hash: &[u8; 20]) -> String {
    hash.iter()
        .map(|b| format!("%{:02x}", b))
        .collect()
}
