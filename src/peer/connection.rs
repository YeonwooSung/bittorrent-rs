use super::{Handshake, PeerMessage, PeerState};
use crate::error::{BittorrentError, Result};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

/// Manages a connection to a peer
pub struct PeerConnection {
    addr: SocketAddr,
    stream: TcpStream,
    state: PeerState,
    peer_id: Option<[u8; 20]>,
    bitfield: Option<Vec<u8>>,
}

impl PeerConnection {
    /// Connect to a peer and perform handshake
    pub async fn connect(
        addr: SocketAddr,
        info_hash: [u8; 20],
        our_peer_id: [u8; 20],
    ) -> Result<Self> {
        info!("Connecting to peer: {}", addr);

        // Connect to peer
        let mut stream = TcpStream::connect(addr).await.map_err(|e| {
            BittorrentError::PeerError(format!("Failed to connect to {}: {}", addr, e))
        })?;

        // Send handshake
        let handshake = Handshake::new(info_hash, our_peer_id);
        stream.write_all(&handshake.to_bytes()).await?;

        debug!("Sent handshake to {}", addr);

        // Receive handshake
        let mut handshake_buf = vec![0u8; 68];
        stream.read_exact(&mut handshake_buf).await?;

        let peer_handshake = Handshake::from_bytes(&handshake_buf)?;

        // Verify info hash
        if peer_handshake.info_hash != info_hash {
            return Err(BittorrentError::PeerError("Info hash mismatch".to_string()));
        }

        info!("Successfully connected to peer: {}", addr);

        Ok(Self {
            addr,
            stream,
            state: PeerState::default(),
            peer_id: Some(peer_handshake.peer_id),
            bitfield: None,
        })
    }

    /// Send a message to the peer
    pub async fn send_message(&mut self, message: &PeerMessage) -> Result<()> {
        let bytes = message.to_bytes();
        self.stream.write_all(&bytes).await?;

        // Update our state based on what we sent
        match message {
            PeerMessage::Choke => self.state.am_choking = true,
            PeerMessage::Unchoke => self.state.am_choking = false,
            PeerMessage::Interested => self.state.am_interested = true,
            PeerMessage::NotInterested => self.state.am_interested = false,
            _ => {}
        }

        debug!("Sent message to {}: {:?}", self.addr, message);
        Ok(())
    }

    /// Receive a message from the peer
    pub async fn receive_message(&mut self) -> Result<PeerMessage> {
        // Read length prefix (4 bytes)
        let mut length_buf = [0u8; 4];
        self.stream.read_exact(&mut length_buf).await?;

        let length = u32::from_be_bytes(length_buf) as usize;

        // Handle keep-alive
        if length == 0 {
            return Ok(PeerMessage::KeepAlive);
        }

        // Read message payload
        let mut message_buf = vec![0u8; length];
        self.stream.read_exact(&mut message_buf).await?;

        // Reconstruct full message for parsing
        let mut full_message = Vec::with_capacity(4 + length);
        full_message.extend_from_slice(&length_buf);
        full_message.extend_from_slice(&message_buf);

        let message = PeerMessage::from_bytes(&full_message)?;

        // Update state based on message
        self.handle_message(&message);

        debug!("Received message from {}: {:?}", self.addr, message);

        Ok(message)
    }

    /// Handle incoming message and update state
    fn handle_message(&mut self, message: &PeerMessage) {
        match message {
            PeerMessage::Choke => self.state.peer_choking = true,
            PeerMessage::Unchoke => self.state.peer_choking = false,
            PeerMessage::Interested => self.state.peer_interested = true,
            PeerMessage::NotInterested => self.state.peer_interested = false,
            PeerMessage::Bitfield { bitfield } => {
                self.bitfield = Some(bitfield.clone());
            }
            _ => {}
        }
    }

    /// Check if peer has a specific piece
    pub fn has_piece(&self, piece_index: usize) -> bool {
        if let Some(bitfield) = &self.bitfield {
            let byte_index = piece_index / 8;
            let bit_index = 7 - (piece_index % 8);

            if byte_index < bitfield.len() {
                return (bitfield[byte_index] >> bit_index) & 1 == 1;
            }
        }
        false
    }

    pub fn state(&self) -> &PeerState {
        &self.state
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn peer_id(&self) -> Option<&[u8; 20]> {
        self.peer_id.as_ref()
    }
}
