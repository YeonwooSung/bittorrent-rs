use crate::error::Result;
use crate::peer::{BlockInfo, PeerConnection, PeerMessage};
use crate::piece::{PieceManager, PiecePicker};
use crate::storage::StorageManager;
use crate::torrent::Metainfo;
use crate::tracker::{generate_peer_id, Peer, TrackerClient, TrackerRequest};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Configuration for the BitTorrent client
pub struct ClientConfig {
    pub download_dir: String,
    pub listen_port: u16,
    pub max_peers: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            download_dir: "./downloads".to_string(),
            listen_port: 6881,
            max_peers: 50,
        }
    }
}

/// Main BitTorrent client
pub struct TorrentClient {
    config: ClientConfig,
    peer_id: [u8; 20],
}

impl TorrentClient {
    pub fn new(config: ClientConfig) -> Self {
        let peer_id = generate_peer_id();
        info!("Client initialized with peer_id: {}", hex::encode(peer_id));

        Self { config, peer_id }
    }

    /// Download a torrent
    pub async fn download(&self, torrent_path: &Path) -> Result<()> {
        info!("Starting download for: {}", torrent_path.display());

        // Load torrent file
        let metainfo = crate::torrent::load_torrent_file(torrent_path).await?;

        info!("Torrent: {}", metainfo.info.name);
        info!("Total size: {} bytes", metainfo.info.total_length);
        info!("Pieces: {}", metainfo.info.pieces.len());
        info!("Info hash: {}", metainfo.info_hash_hex());

        // Initialize components
        let storage = StorageManager::new(&self.config.download_dir, &metainfo.info).await?;
        let piece_manager = Arc::new(Mutex::new(PieceManager::new(
            metainfo.info.piece_length,
            metainfo.info.total_length,
            &metainfo.info.pieces,
        )));
        let piece_picker = Arc::new(Mutex::new(PiecePicker::new(
            metainfo.info.pieces.len(),
        )));

        // Contact tracker
        let tracker_client = TrackerClient::new();
        let request = TrackerRequest::new(
            metainfo.info_hash,
            self.peer_id,
            self.config.listen_port,
            metainfo.info.total_length,
        );

        let tracker_response = tracker_client.announce(&metainfo.announce, &request).await?;

        info!(
            "Received {} peers from tracker",
            tracker_response.peers.len()
        );

        // TODO: Connect to peers and download pieces
        // This is where the main download logic will go
        // For now, we'll just print the peer list

        for peer in &tracker_response.peers {
            info!("Peer: {}", peer.addr);
        }

        warn!("Download logic not yet implemented - this is a framework");

        Ok(())
    }

    /// Download a piece from a peer
    async fn download_piece_from_peer(
        peer: &mut PeerConnection,
        piece_index: usize,
        piece_manager: Arc<Mutex<PieceManager>>,
        storage: Arc<StorageManager>,
    ) -> Result<()> {
        // Start the piece
        {
            let mut pm = piece_manager.lock().await;
            pm.start_piece(piece_index)?;
        }

        // Send interested message
        peer.send_message(&PeerMessage::Interested).await?;

        // Wait for unchoke
        loop {
            let msg = peer.receive_message().await?;
            if matches!(msg, PeerMessage::Unchoke) {
                break;
            }
        }

        // Request blocks
        let num_blocks = {
            let pm = piece_manager.lock().await;
            pm.blocks_in_piece(piece_index)
        };

        for block_index in 0..num_blocks {
            let (offset, length) = {
                let pm = piece_manager.lock().await;
                pm.get_block_info(piece_index, block_index)
                    .ok_or_else(|| crate::error::BittorrentError::PieceError(
                        "Invalid block".to_string()
                    ))?
            };

            let block = BlockInfo::new(piece_index as u32, offset, length);
            peer.send_message(&PeerMessage::Request { block }).await?;

            // Receive piece
            let msg = peer.receive_message().await?;
            if let PeerMessage::Piece {
                piece_index: received_index,
                offset: received_offset,
                data,
            } = msg
            {
                if received_index as usize == piece_index && received_offset == offset {
                    let mut pm = piece_manager.lock().await;
                    pm.add_block(piece_index, offset, &data)?;
                }
            }
        }

        // Complete and verify piece
        let piece_data = {
            let mut pm = piece_manager.lock().await;
            pm.complete_piece(piece_index)?
        };

        // Write to storage
        storage.write_piece(piece_index, &piece_data).await?;

        Ok(())
    }
}

impl Default for TorrentClient {
    fn default() -> Self {
        Self::new(ClientConfig::default())
    }
}
