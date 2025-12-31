use super::{TrackerRequest, TrackerResponse};
use crate::bencode::decode;
use crate::error::Result;
use reqwest::Client;
use tracing::{debug, info};

/// Client for communicating with BitTorrent trackers
pub struct TrackerClient {
    client: Client,
}

impl TrackerClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Send a request to a tracker and get the peer list
    pub async fn announce(&self, tracker_url: &str, request: &TrackerRequest) -> Result<TrackerResponse> {
        info!("Announcing to tracker: {}", tracker_url);

        // Build URL with query parameters
        let url = reqwest::Url::parse_with_params(
            tracker_url,
            &request.to_query_params(),
        )?;

        debug!("Tracker request URL: {}", url);

        // Send GET request
        let response = self.client.get(url).send().await?;

        let status = response.status();
        let body = response.bytes().await?;

        debug!("Tracker response status: {}, body length: {}", status, body.len());

        if !status.is_success() {
            return Err(crate::error::BittorrentError::TrackerError(
                format!("HTTP error: {}", status)
            ));
        }

        // Decode bencoded response
        let decoded = decode(&body)?;
        let tracker_response = TrackerResponse::from_bencode(decoded)?;

        info!(
            "Received {} peers from tracker (interval: {}s)",
            tracker_response.peers.len(),
            tracker_response.interval
        );

        Ok(tracker_response)
    }
}

impl Default for TrackerClient {
    fn default() -> Self {
        Self::new()
    }
}
