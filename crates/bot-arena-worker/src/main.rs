//! Bot Arena Worker - Executes matches from the database.

mod db;

use clap::Parser;
use std::path::PathBuf;

/// Bot Arena Worker - Executes bot matches from the database.
#[derive(Parser)]
#[command(name = "bot-arena-worker")]
#[command(about = "Executes bot matches from the database")]
struct Args {
    /// Path to SQLite database
    #[arg(long, default_value = "data/arena.db")]
    db: PathBuf,

    /// Poll interval in milliseconds
    #[arg(long, default_value = "1000")]
    poll_interval: u64,

    /// Directory containing bot executables
    #[arg(long, default_value = "bots")]
    bots_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    tracing::info!("Starting bot-arena-worker");
    tracing::info!("Database: {:?}", args.db);
    tracing::info!("Poll interval: {}ms", args.poll_interval);

    let db = db::connect(&args.db)?;
    let worker_id = uuid::Uuid::new_v4().to_string();
    tracing::info!("Worker ID: {}", worker_id);

    // TODO: Implement worker loop
    let _ = (db, worker_id, args.bots_dir, args.poll_interval);
    Ok(())
}
