use crate::error::{BittorrentError, Result};
use crate::torrent::{FileInfo, TorrentInfo};
use std::path::{Path, PathBuf};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tracing::{debug, info};

/// Manages file I/O for downloaded pieces
pub struct StorageManager {
    /// Base directory for downloads
    download_dir: PathBuf,
    /// Files in the torrent
    files: Vec<FileEntry>,
    /// Total length of all files
    total_length: u64,
    /// Piece length
    piece_length: u64,
}

struct FileEntry {
    path: PathBuf,
    length: u64,
    offset: u64, // Global offset in the torrent
}

impl StorageManager {
    /// Create a new storage manager
    pub async fn new<P: AsRef<Path>>(
        download_dir: P,
        torrent_info: &TorrentInfo,
    ) -> Result<Self> {
        let download_dir = download_dir.as_ref().to_path_buf();

        // Create download directory
        fs::create_dir_all(&download_dir).await?;

        let mut files = Vec::new();
        let mut offset = 0u64;

        for file_info in &torrent_info.files {
            let mut file_path = download_dir.clone();
            for component in &file_info.path {
                file_path.push(component);
            }

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            files.push(FileEntry {
                path: file_path,
                length: file_info.length,
                offset,
            });

            offset += file_info.length;
        }

        info!(
            "Storage initialized: {} files, {} bytes total",
            files.len(),
            torrent_info.total_length
        );

        Ok(Self {
            download_dir,
            files,
            total_length: torrent_info.total_length,
            piece_length: torrent_info.piece_length,
        })
    }

    /// Write a piece to disk
    pub async fn write_piece(&self, piece_index: usize, data: &[u8]) -> Result<()> {
        let global_offset = (piece_index as u64) * self.piece_length;

        debug!(
            "Writing piece {} at global offset {} ({} bytes)",
            piece_index,
            global_offset,
            data.len()
        );

        self.write_at_offset(global_offset, data).await?;

        info!("Piece {} written to disk", piece_index);
        Ok(())
    }

    /// Read a piece from disk
    pub async fn read_piece(&self, piece_index: usize) -> Result<Vec<u8>> {
        let global_offset = (piece_index as u64) * self.piece_length;

        // Calculate piece length (last piece might be smaller)
        let piece_length = if piece_index == self.num_pieces() - 1 {
            let remainder = self.total_length % self.piece_length;
            if remainder == 0 {
                self.piece_length
            } else {
                remainder
            }
        } else {
            self.piece_length
        };

        self.read_at_offset(global_offset, piece_length as usize).await
    }

    /// Write data at a global offset (spans multiple files if needed)
    async fn write_at_offset(&self, mut offset: u64, mut data: &[u8]) -> Result<()> {
        for file_entry in &self.files {
            if offset >= file_entry.offset + file_entry.length {
                continue; // This file is before our offset
            }

            if offset < file_entry.offset {
                break; // We've passed our offset
            }

            let file_offset = offset - file_entry.offset;
            let bytes_to_write = std::cmp::min(
                data.len() as u64,
                file_entry.length - file_offset,
            ) as usize;

            // Open/create file and write
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&file_entry.path)
                .await?;

            file.seek(std::io::SeekFrom::Start(file_offset)).await?;
            file.write_all(&data[..bytes_to_write]).await?;

            debug!(
                "Wrote {} bytes to {:?} at offset {}",
                bytes_to_write, file_entry.path, file_offset
            );

            // Move to next file
            offset += bytes_to_write as u64;
            data = &data[bytes_to_write..];

            if data.is_empty() {
                break;
            }
        }

        Ok(())
    }

    /// Read data from a global offset (spans multiple files if needed)
    async fn read_at_offset(&self, mut offset: u64, mut length: usize) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(length);

        for file_entry in &self.files {
            if offset >= file_entry.offset + file_entry.length {
                continue;
            }

            if offset < file_entry.offset {
                break;
            }

            let file_offset = offset - file_entry.offset;
            let bytes_to_read = std::cmp::min(
                length as u64,
                file_entry.length - file_offset,
            ) as usize;

            // Open file and read
            let mut file = File::open(&file_entry.path).await?;
            file.seek(std::io::SeekFrom::Start(file_offset)).await?;

            let mut buffer = vec![0u8; bytes_to_read];
            file.read_exact(&mut buffer).await?;

            result.extend_from_slice(&buffer);

            offset += bytes_to_read as u64;
            length -= bytes_to_read;

            if length == 0 {
                break;
            }
        }

        Ok(result)
    }

    fn num_pieces(&self) -> usize {
        ((self.total_length + self.piece_length - 1) / self.piece_length) as usize
    }
}
