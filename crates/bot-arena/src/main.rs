mod config;
mod game_runner;
mod storage;
mod uci_client;

use clap::{Parser, Subcommand};
use config::ArenaConfig;
use game_runner::{GameRunner, MatchResult};
use storage::Storage;
use uci_client::UciClient;

#[derive(Parser)]
#[command(name = "bot-arena")]
#[command(about = "Chess bot comparison tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a match between two bots
    Match {
        /// White bot name
        white: String,
        /// Black bot name
        black: String,
        /// Number of games to play
        #[arg(short, long, default_value = "10")]
        games: u32,
    },
}

fn main() {
    let cli = Cli::parse();
    let config = ArenaConfig::load().unwrap_or_default();

    // Create data directory and open storage
    std::fs::create_dir_all("data").ok();
    let storage = Storage::open("data/arena.db").expect("Failed to open database");

    match cli.command {
        Commands::Match {
            white,
            black,
            games,
        } => {
            let white_path = config
                .get_bot(&white)
                .map(|b| b.path.clone())
                .unwrap_or_else(|_| white.clone().into());
            let black_path = config
                .get_bot(&black)
                .map(|b| b.path.clone())
                .unwrap_or_else(|_| black.clone().into());
            let time_control = config
                .get_bot(&white)
                .map(|b| b.time_control.clone())
                .unwrap_or_else(|_| "movetime 500".to_string());

            // Ensure bots are registered in database
            storage
                .ensure_bot(&white, Some(white_path.to_str().unwrap_or("")))
                .ok();
            storage
                .ensure_bot(&black, Some(black_path.to_str().unwrap_or("")))
                .ok();

            println!("Running {} games: {} vs {}", games, white, black);

            let mut white_wins = 0;
            let mut black_wins = 0;
            let mut draws = 0;

            for i in 1..=games {
                let white_client =
                    UciClient::spawn(&white_path).expect("Failed to spawn white engine");
                let black_client =
                    UciClient::spawn(&black_path).expect("Failed to spawn black engine");

                let mut runner = GameRunner::new(white_client, black_client, time_control.clone())
                    .expect("Failed to initialize game");

                match runner.play_game() {
                    Ok(mut result) => {
                        // Set bot names from config
                        result.white_name = white.clone();
                        result.black_name = black.clone();

                        match result.result {
                            MatchResult::WhiteWins => white_wins += 1,
                            MatchResult::BlackWins => black_wins += 1,
                            MatchResult::Draw => draws += 1,
                        }

                        // Save game to database
                        if let Err(e) = storage.save_game(&result) {
                            eprintln!("Warning: Failed to save game to database: {}", e);
                        }

                        println!(
                            "Game {}: {:?} ({} moves)",
                            i,
                            result.result,
                            result.moves.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("Game {} error: {}", i, e);
                    }
                }
            }

            // Print session results
            println!(
                "\nSession Results: W:{} D:{} L:{}",
                white_wins, draws, black_wins
            );

            // Print cumulative stats from database
            if let Ok((total_games, wins, db_draws, losses)) = storage.get_stats(&white) {
                println!(
                    "\n{} all-time stats: {} games, {} wins, {} draws, {} losses",
                    white, total_games, wins, db_draws, losses
                );
            }
            if let Ok((total_games, wins, db_draws, losses)) = storage.get_stats(&black) {
                println!(
                    "{} all-time stats: {} games, {} wins, {} draws, {} losses",
                    black, total_games, wins, db_draws, losses
                );
            }
        }
    }
}
