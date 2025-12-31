use crate::error::{BittorrentError, Result};
use crate::peer::{BlockInfo, PeerConnection, PeerMessage};
use crate::piece::{PieceManager, PiecePicker};
use crate::storage::StorageManager;
use crate::tracker::{generate_peer_id, TrackerClient, TrackerRequest};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

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
        let piece_picker = Arc::new(Mutex::new(PiecePicker::new(metainfo.info.pieces.len())));

        // Contact tracker
        let tracker_client = TrackerClient::new();
        let request = TrackerRequest::new(
            metainfo.info_hash,
            self.peer_id,
            self.config.listen_port,
            metainfo.info.total_length,
        );

        let tracker_response = tracker_client
            .announce(&metainfo.announce, &request)
            .await?;

        info!(
            "Received {} peers from tracker",
            tracker_response.peers.len()
        );

        // Try to connect to peers and download
        if tracker_response.peers.is_empty() {
            return Err(BittorrentError::TrackerError(
                "No peers available".to_string(),
            ));
        }

        // Storage를 Arc로 감싸서 공유
        let storage = Arc::new(storage);

        // Try to connect to multiple peers
        let mut peer_connections = Vec::new();
        let max_connections = std::cmp::min(self.config.max_peers, tracker_response.peers.len());

        info!("Attempting to connect to up to {} peers", max_connections);

        for peer_info in tracker_response.peers.iter().take(max_connections * 2) {
            if peer_connections.len() >= max_connections {
                break;
            }

            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                PeerConnection::connect(peer_info.addr, metainfo.info_hash, self.peer_id),
            )
            .await
            {
                Ok(Ok(conn)) => {
                    info!("Successfully connected to peer: {}", peer_info.addr);
                    peer_connections.push(conn);
                }
                Ok(Err(e)) => {
                    warn!("Failed to connect to peer {}: {}", peer_info.addr, e);
                }
                Err(_) => {
                    warn!("Connection timeout to peer: {}", peer_info.addr);
                }
            }
        }

        if peer_connections.is_empty() {
            return Err(BittorrentError::PeerError(
                "Could not connect to any peers".to_string(),
            ));
        }

        info!(
            "Connected to {} peers, starting download",
            peer_connections.len()
        );

        // Download pieces concurrently using multiple peers
        let peer_connections = Arc::new(Mutex::new(peer_connections));

        // Create progress monitoring task
        let progress_piece_manager = piece_manager.clone();
        let progress_task = tokio::spawn(async move {
            let mut last_progress = 0.0;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                let (complete, progress, complete_count, total) = {
                    let pm = progress_piece_manager.lock().await;
                    (
                        pm.is_complete(),
                        pm.progress(),
                        pm.complete_count(),
                        pm.piece_count(),
                    )
                };

                if complete {
                    break;
                }

                if (progress - last_progress).abs() > 0.1 {
                    info!(
                        "Download progress: {:.1}% ({}/{})",
                        progress, complete_count, total
                    );
                    last_progress = progress;
                }
            }
        });

        // Create tasks for each peer
        let mut tasks = Vec::new();
        let num_peers = {
            let conns = peer_connections.lock().await;
            conns.len()
        };

        for _ in 0..num_peers {
            let piece_picker_clone = piece_picker.clone();
            let piece_manager_clone = piece_manager.clone();
            let storage_clone = storage.clone();
            let peer_connections_clone = peer_connections.clone();
            let total_pieces = metainfo.info.pieces.len();

            let task = tokio::spawn(async move {
                loop {
                    // Get next piece to download
                    let piece_index = {
                        let mut picker = piece_picker_clone.lock().await;
                        let pm = piece_manager_clone.lock().await;
                        picker.pick_piece(&pm)
                    };

                    let piece_index = match piece_index {
                        Some(idx) => idx,
                        None => {
                            // No more pieces to download
                            break;
                        }
                    };

                    // Get a peer connection
                    let mut peer = {
                        let mut conns = peer_connections_clone.lock().await;
                        if conns.is_empty() {
                            break;
                        }
                        conns.pop().unwrap()
                    };

                    // Check if peer has this piece
                    if !peer.has_piece(piece_index) {
                        // Return peer to pool and skip
                        let mut conns = peer_connections_clone.lock().await;
                        conns.push(peer);
                        continue;
                    }

                    info!(
                        "Downloading piece {}/{} from peer {}",
                        piece_index + 1,
                        total_pieces,
                        peer.addr()
                    );

                    // Download the piece
                    let result = Self::download_piece_from_peer(
                        &mut peer,
                        piece_index,
                        piece_manager_clone.clone(),
                        storage_clone.clone(),
                    )
                    .await;

                    // Return peer to pool
                    {
                        let mut conns = peer_connections_clone.lock().await;
                        conns.push(peer);
                    }

                    match result {
                        Ok(_) => {
                            info!("Successfully downloaded piece {}", piece_index);
                        }
                        Err(e) => {
                            warn!("Failed to download piece {}: {}", piece_index, e);
                            // Mark piece as available again
                            let mut picker = piece_picker_clone.lock().await;
                            picker.mark_missing(piece_index);
                        }
                    }
                }
            });

            tasks.push(task);
        }

        // Wait for all download tasks to complete
        for task in tasks {
            let _ = task.await;
        }

        // Stop progress monitoring
        progress_task.abort();

        // Check if download is complete
        let (complete, progress) = {
            let pm = piece_manager.lock().await;
            (pm.is_complete(), pm.progress())
        };

        if complete {
            info!("Download complete! All pieces downloaded and verified.");
        } else {
            warn!(
                "Download incomplete. Progress: {:.1}%. Some pieces may be missing.",
                progress
            );
        }

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

        // Send interested message if we're not already interested
        if !peer.state().am_interested {
            peer.send_message(&PeerMessage::Interested).await?;
        }

        // Wait for unchoke (with timeout)
        let unchoke_result = tokio::time::timeout(tokio::time::Duration::from_secs(30), async {
            loop {
                let msg = peer.receive_message().await?;
                match msg {
                    PeerMessage::Unchoke => {
                        info!("Peer unchoked us, ready to download piece {}", piece_index);
                        break;
                    }
                    PeerMessage::Choke => {
                        warn!("Peer choked us while waiting for unchoke");
                        return Err(BittorrentError::PeerError("Peer choked us".to_string()));
                    }
                    PeerMessage::KeepAlive => {
                        // Just continue waiting
                    }
                    _ => {
                        // Handle other messages but keep waiting
                    }
                }
            }
            Ok::<(), BittorrentError>(())
        })
        .await;

        match unchoke_result {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(BittorrentError::PeerError(
                    "Timeout waiting for unchoke".to_string(),
                ))
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
                    .ok_or_else(|| BittorrentError::PieceError("Invalid block".to_string()))?
            };

            let block = BlockInfo::new(piece_index as u32, offset, length);
            peer.send_message(&PeerMessage::Request { block }).await?;

            // Receive piece (with timeout)
            let receive_result =
                tokio::time::timeout(tokio::time::Duration::from_secs(30), peer.receive_message())
                    .await;

            match receive_result {
                Ok(Ok(PeerMessage::Piece {
                    piece_index: received_index,
                    offset: received_offset,
                    data,
                })) => {
                    if received_index as usize == piece_index && received_offset == offset {
                        let mut pm = piece_manager.lock().await;
                        pm.add_block(piece_index, offset, &data)?;
                    } else {
                        warn!(
                            "Received unexpected piece data: expected piece {}, offset {}, got piece {}, offset {}",
                            piece_index, offset, received_index, received_offset
                        );
                    }
                }
                Ok(Ok(other_msg)) => {
                    return Err(BittorrentError::PeerError(format!(
                        "Expected Piece message, got {:?}",
                        other_msg
                    )));
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    return Err(BittorrentError::PeerError(
                        "Timeout receiving block".to_string(),
                    ))
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
