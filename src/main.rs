use clap::Parser;
use tokio::fs;
use tokio::time::Duration;
use anyhow::{Context, Result};
use tracing::{info, error, Level};
use tracing_subscriber;

#[derive(Parser)]
struct Opts {
    #[clap(short, long)]
    file_path: String,
    
    #[clap(short, long)]
    time_in_minutes: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let opts: Opts = Opts::parse();

    let file_path = &opts.file_path;
    let time_in_minutes = opts.time_in_minutes;

    let duration = Duration::from_secs(time_in_minutes * 60);

    info!("Scheduled to delete '{}' in {} minutes.", file_path, time_in_minutes);

    tokio::time::sleep(duration).await;

    fs::remove_file(file_path)
        .await
        .with_context(|| format!("Failed to delete file: '{}'", file_path))?;

    info!("'{}' has been deleted.", file_path);
    Ok(())
}
