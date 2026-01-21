//! Bot Arena Worker - Executes matches from the database.
//!
//! This worker polls the database for pending matches, executes them using
//! UCI chess engines, and writes results back to the database.

mod db;
mod elo;
mod runner;

use bot_arena::game_runner::MatchResult;
use clap::Parser;
use runner::MatchRunner;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

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
    tracing::info!("Bots directory: {:?}", args.bots_dir);

    let db = db::connect(&args.db)?;
    let worker_id = uuid::Uuid::new_v4().to_string();
    tracing::info!("Worker ID: {}", worker_id);

    let runner = MatchRunner::new(&args.bots_dir);

    // Shutdown flag
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    // Spawn signal handler
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        tracing::info!("Shutdown signal received");
        shutdown_clone.store(true, Ordering::SeqCst);
    });

    // Track the currently running match ID for graceful shutdown.
    // Note: The shutdown check only runs between loop iterations, so if a match
    // is in progress during shutdown, it will complete before the worker exits.
    // The match ID is tracked so it can be released if shutdown occurs after
    // claiming but before the match starts (edge case).
    let mut current_match_id: Option<String> = None;

    // Main worker loop
    loop {
        // Check for shutdown
        if shutdown.load(Ordering::SeqCst) {
            if let Some(ref match_id) = current_match_id {
                tracing::info!("Releasing match {} due to shutdown", match_id);
                if let Err(e) = db::release_match(&db, match_id, &worker_id) {
                    tracing::error!("Failed to release match: {}", e);
                }
            }
            break;
        }

        match db::claim_match(&db, &worker_id) {
            Ok(Some(pending)) => {
                // Track current match for graceful shutdown release.
                // This is intentionally set before run_match so it can be released
                // if a shutdown signal is received during match execution.
                // Note: Rust warns about unused assignment because it doesn't track
                // the value being read across loop iterations at shutdown check.
                #[allow(unused_assignments)]
                {
                    current_match_id = Some(pending.id.clone());
                }
                tracing::info!(
                    "Claimed match: {} ({} vs {})",
                    pending.id,
                    pending.white_bot,
                    pending.black_bot
                );

                match runner.run_match(&pending) {
                    Ok(results) => {
                        let mut white_score = 0.0;
                        let mut black_score = 0.0;
                        let mut game_results = Vec::new();

                        for (game_num, (game_id, result)) in results.iter().enumerate() {
                            // Create game record
                            if let Err(e) =
                                db::create_game(&db, game_id, &pending.id, game_num as i32)
                            {
                                tracing::error!("Failed to create game {}: {}", game_id, e);
                                continue;
                            }

                            // Insert all moves
                            for (ply, move_record) in result.moves.iter().enumerate() {
                                let _ = db::insert_move(
                                    &db,
                                    game_id,
                                    ply as i32,
                                    &move_record.uci,
                                    None, // SAN not available from MoveRecord
                                    "",   // FEN not available
                                );
                            }

                            // Calculate scores (considering color alternation)
                            // In even-numbered games, white_bot plays white
                            // In odd-numbered games, black_bot plays white
                            let game_result_str = match result.result {
                                MatchResult::WhiteWins => {
                                    if game_num % 2 == 0 {
                                        white_score += 1.0;
                                    } else {
                                        black_score += 1.0;
                                    }
                                    "1-0"
                                }
                                MatchResult::BlackWins => {
                                    if game_num % 2 == 0 {
                                        black_score += 1.0;
                                    } else {
                                        white_score += 1.0;
                                    }
                                    "0-1"
                                }
                                MatchResult::Draw => {
                                    white_score += 0.5;
                                    black_score += 0.5;
                                    "1/2-1/2"
                                }
                            };

                            // Collect game result for Elo update
                            game_results.push(db::GameResult {
                                game_num: game_num as i32,
                                result: game_result_str.to_string(),
                            });

                            let _ = db::finish_game(&db, game_id, game_result_str);
                            tracing::info!("Game {} finished: {}", game_id, game_result_str);
                        }

                        // Finish the match
                        if let Err(e) = db::finish_match(&db, &pending.id, white_score, black_score)
                        {
                            tracing::error!("Failed to finish match {}: {}", pending.id, e);
                        } else {
                            tracing::info!(
                                "Match {} completed: {} - {}",
                                pending.id,
                                white_score,
                                black_score
                            );
                        }

                        // Update Elo ratings
                        if let Err(e) = db::update_elo_ratings(&db, &pending.id, &game_results) {
                            tracing::error!(
                                "Failed to update Elo ratings for match {}: {}",
                                pending.id,
                                e
                            );
                        } else {
                            tracing::info!("Elo ratings updated for match {}", pending.id);
                        }

                        // Clear current match after successful completion
                        current_match_id = None;
                    }
                    Err(e) => {
                        tracing::error!("Match {} failed: {}", pending.id, e);
                        // Clear current match on failure (match remains in running state)
                        // TODO: Mark match as failed in database
                        current_match_id = None;
                    }
                }
            }
            Ok(None) => {
                // No pending matches, wait before polling again
                tokio::time::sleep(Duration::from_millis(args.poll_interval)).await;
            }
            Err(e) => {
                tracing::error!("Database error: {}", e);
                tokio::time::sleep(Duration::from_millis(args.poll_interval)).await;
            }
        }
    }

    tracing::info!("Worker shutdown complete");
    Ok(())
}
