use crate::client::{ClientConfig, TorrentClient};
use crate::error::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bittorrent-rs")]
#[command(about = "A BitTorrent client written in Rust", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download a torrent file
    Download {
        /// Path to the .torrent file
        #[arg(short, long)]
        torrent: PathBuf,

        /// Download directory
        #[arg(short, long, default_value = "./downloads")]
        output: String,

        /// Port to listen on
        #[arg(short, long, default_value = "6881")]
        port: u16,

        /// Maximum number of peers to connect to
        #[arg(short, long, default_value = "50")]
        max_peers: usize,
    },

    /// Show information about a torrent file
    Info {
        /// Path to the .torrent file
        torrent: PathBuf,
    },
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Download {
                torrent,
                output,
                port,
                max_peers,
            } => {
                let config = ClientConfig {
                    download_dir: output.clone(),
                    listen_port: *port,
                    max_peers: *max_peers,
                };

                let client = TorrentClient::new(config);
                client.download(torrent).await?;
            }

            Commands::Info { torrent } => {
                self.show_torrent_info(torrent).await?;
            }
        }

        Ok(())
    }

    async fn show_torrent_info(&self, torrent_path: &PathBuf) -> Result<()> {
        let metainfo = crate::torrent::load_torrent_file(torrent_path).await?;

        println!("Torrent Information");
        println!("==================");
        println!("Name: {}", metainfo.info.name);
        println!("Tracker: {}", metainfo.announce);
        println!("Total Size: {} bytes", metainfo.info.total_length);
        println!("Piece Length: {} bytes", metainfo.info.piece_length);
        println!("Number of Pieces: {}", metainfo.info.pieces.len());
        println!("Info Hash: {}", metainfo.info_hash_hex());
        println!("\nFiles:");

        for (i, file) in metainfo.info.files.iter().enumerate() {
            println!(
                "  {}: {} ({} bytes)",
                i + 1,
                file.path.join("/"),
                file.length
            );
        }

        if let Some(announce_list) = &metainfo.announce_list {
            println!("\nAdditional Trackers:");
            for (tier, trackers) in announce_list.iter().enumerate() {
                println!("  Tier {}:", tier + 1);
                for tracker in trackers {
                    println!("    - {}", tracker);
                }
            }
        }

        Ok(())
    }
}
