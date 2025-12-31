use bytes::{Buf, BufMut, BytesMut};
use crate::error::{BittorrentError, Result};

/// Information about a block within a piece
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockInfo {
    /// Piece index
    pub piece_index: u32,
    /// Byte offset within the piece
    pub offset: u32,
    /// Length of the block
    pub length: u32,
}

impl BlockInfo {
    pub fn new(piece_index: u32, offset: u32, length: u32) -> Self {
        Self {
            piece_index,
            offset,
            length,
        }
    }
}

/// Messages exchanged between peers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerMessage {
    /// Keep-alive message (no payload)
    KeepAlive,
    /// Choke the peer
    Choke,
    /// Unchoke the peer
    Unchoke,
    /// Indicate interest
    Interested,
    /// Indicate lack of interest
    NotInterested,
    /// Indicate possession of a piece
    Have { piece_index: u32 },
    /// Bitfield of available pieces
    Bitfield { bitfield: Vec<u8> },
    /// Request a block
    Request { block: BlockInfo },
    /// Send a block
    Piece {
        piece_index: u32,
        offset: u32,
        data: Vec<u8>,
    },
    /// Cancel a block request
    Cancel { block: BlockInfo },
}

impl PeerMessage {
    /// Message type IDs
    const CHOKE: u8 = 0;
    const UNCHOKE: u8 = 1;
    const INTERESTED: u8 = 2;
    const NOT_INTERESTED: u8 = 3;
    const HAVE: u8 = 4;
    const BITFIELD: u8 = 5;
    const REQUEST: u8 = 6;
    const PIECE: u8 = 7;
    const CANCEL: u8 = 8;

    /// Serialize message to bytes
    /// Format: <length prefix><message ID><payload>
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();

        match self {
            PeerMessage::KeepAlive => {
                buf.put_u32(0); // length = 0
            }
            PeerMessage::Choke => {
                buf.put_u32(1); // length = 1
                buf.put_u8(Self::CHOKE);
            }
            PeerMessage::Unchoke => {
                buf.put_u32(1);
                buf.put_u8(Self::UNCHOKE);
            }
            PeerMessage::Interested => {
                buf.put_u32(1);
                buf.put_u8(Self::INTERESTED);
            }
            PeerMessage::NotInterested => {
                buf.put_u32(1);
                buf.put_u8(Self::NOT_INTERESTED);
            }
            PeerMessage::Have { piece_index } => {
                buf.put_u32(5); // length = 1 + 4
                buf.put_u8(Self::HAVE);
                buf.put_u32(*piece_index);
            }
            PeerMessage::Bitfield { bitfield } => {
                buf.put_u32((1 + bitfield.len()) as u32);
                buf.put_u8(Self::BITFIELD);
                buf.put_slice(bitfield);
            }
            PeerMessage::Request { block } => {
                buf.put_u32(13); // length = 1 + 4 + 4 + 4
                buf.put_u8(Self::REQUEST);
                buf.put_u32(block.piece_index);
                buf.put_u32(block.offset);
                buf.put_u32(block.length);
            }
            PeerMessage::Piece {
                piece_index,
                offset,
                data,
            } => {
                buf.put_u32((9 + data.len()) as u32);
                buf.put_u8(Self::PIECE);
                buf.put_u32(*piece_index);
                buf.put_u32(*offset);
                buf.put_slice(data);
            }
            PeerMessage::Cancel { block } => {
                buf.put_u32(13);
                buf.put_u8(Self::CANCEL);
                buf.put_u32(block.piece_index);
                buf.put_u32(block.offset);
                buf.put_u32(block.length);
            }
        }

        buf.to_vec()
    }

    /// Deserialize message from bytes
    pub fn from_bytes(mut data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(BittorrentError::PeerError(
                "Message too short".to_string(),
            ));
        }

        let length = data.get_u32() as usize;

        if length == 0 {
            return Ok(PeerMessage::KeepAlive);
        }

        if data.len() < length {
            return Err(BittorrentError::PeerError(
                "Incomplete message".to_string(),
            ));
        }

        let message_id = data.get_u8();

        match message_id {
            Self::CHOKE => Ok(PeerMessage::Choke),
            Self::UNCHOKE => Ok(PeerMessage::Unchoke),
            Self::INTERESTED => Ok(PeerMessage::Interested),
            Self::NOT_INTERESTED => Ok(PeerMessage::NotInterested),
            Self::HAVE => {
                if data.len() < 4 {
                    return Err(BittorrentError::PeerError("Invalid Have message".to_string()));
                }
                let piece_index = data.get_u32();
                Ok(PeerMessage::Have { piece_index })
            }
            Self::BITFIELD => {
                let bitfield = data.to_vec();
                Ok(PeerMessage::Bitfield { bitfield })
            }
            Self::REQUEST => {
                if data.len() < 12 {
                    return Err(BittorrentError::PeerError("Invalid Request message".to_string()));
                }
                let piece_index = data.get_u32();
                let offset = data.get_u32();
                let length = data.get_u32();
                Ok(PeerMessage::Request {
                    block: BlockInfo::new(piece_index, offset, length),
                })
            }
            Self::PIECE => {
                if data.len() < 8 {
                    return Err(BittorrentError::PeerError("Invalid Piece message".to_string()));
                }
                let piece_index = data.get_u32();
                let offset = data.get_u32();
                let piece_data = data.to_vec();
                Ok(PeerMessage::Piece {
                    piece_index,
                    offset,
                    data: piece_data,
                })
            }
            Self::CANCEL => {
                if data.len() < 12 {
                    return Err(BittorrentError::PeerError("Invalid Cancel message".to_string()));
                }
                let piece_index = data.get_u32();
                let offset = data.get_u32();
                let length = data.get_u32();
                Ok(PeerMessage::Cancel {
                    block: BlockInfo::new(piece_index, offset, length),
                })
            }
            _ => Err(BittorrentError::PeerError(format!(
                "Unknown message ID: {}",
                message_id
            ))),
        }
    }
}
