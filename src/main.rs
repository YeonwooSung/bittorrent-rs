mod bencode;
mod cli;
mod client;
mod error;
mod peer;
mod piece;
mod storage;
mod torrent;
mod tracker;

use anyhow::Result;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Parse CLI arguments and run
    let cli = Cli::parse();
    cli.run().await?;

    Ok(())
}
