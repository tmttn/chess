mod config;
mod game_runner;
mod uci_client;

use clap::{Parser, Subcommand};
use config::ArenaConfig;
use game_runner::{GameRunner, MatchResult};
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
                    Ok(result) => {
                        match result.result {
                            MatchResult::WhiteWins => white_wins += 1,
                            MatchResult::BlackWins => black_wins += 1,
                            MatchResult::Draw => draws += 1,
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

            println!("\nResults: W:{} D:{} L:{}", white_wins, draws, black_wins);
        }
    }
}
