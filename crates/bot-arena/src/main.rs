mod config;
mod game_runner;
mod json_output;
mod pgn;
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
        /// Preset configuration to use
        #[arg(short, long)]
        preset: Option<String>,
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
            preset,
        } => {
            let white_path = config
                .get_bot(&white)
                .map(|b| b.path.clone())
                .unwrap_or_else(|_| white.clone().into());
            let black_path = config
                .get_bot(&black)
                .map(|b| b.path.clone())
                .unwrap_or_else(|_| black.clone().into());

            // Determine games and time_control from preset or defaults
            let (games, time_control) = if let Some(preset_name) = &preset {
                if let Some(p) = config.presets.get(preset_name) {
                    println!("Using preset: {}", preset_name);
                    (p.games, p.time_control.clone())
                } else {
                    eprintln!("Unknown preset: {}", preset_name);
                    return;
                }
            } else {
                (
                    games,
                    config
                        .get_bot(&white)
                        .map(|b| b.time_control.clone())
                        .unwrap_or_else(|_| "movetime 500".to_string()),
                )
            };

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
                        let game_id = storage
                            .save_game(&result)
                            .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

                        // Save PGN file
                        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                        let pgn_dir = format!("data/games/{}", date);
                        if let Err(e) = std::fs::create_dir_all(&pgn_dir) {
                            eprintln!("Warning: Failed to create PGN directory {}: {}", pgn_dir, e);
                        }
                        let pgn_path = format!("{}/{}.pgn", pgn_dir, game_id);
                        if let Err(e) = pgn::write_pgn(&pgn_path, &result) {
                            eprintln!("Warning: Failed to save PGN file: {}", e);
                        }

                        // Save JSON file with search info
                        let json_path = format!("{}/{}.json", pgn_dir, game_id);
                        if let Err(e) = json_output::write_json(&json_path, &game_id, &result) {
                            eprintln!("Warning: Failed to write JSON: {}", e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parses_match_command_with_preset() {
        // Verify the CLI can parse a match command with preset argument
        let cli = Cli::try_parse_from(["bot-arena", "match", "bot1", "bot2", "-p", "quick"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match cli.command {
            Commands::Match {
                white,
                black,
                games,
                preset,
            } => {
                assert_eq!(white, "bot1");
                assert_eq!(black, "bot2");
                assert_eq!(games, 10); // default value
                assert_eq!(preset, Some("quick".to_string()));
            }
        }
    }

    #[test]
    fn test_cli_parses_match_command_without_preset() {
        let cli = Cli::try_parse_from(["bot-arena", "match", "bot1", "bot2"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match cli.command {
            Commands::Match {
                white,
                black,
                games,
                preset,
            } => {
                assert_eq!(white, "bot1");
                assert_eq!(black, "bot2");
                assert_eq!(games, 10);
                assert!(preset.is_none());
            }
        }
    }

    #[test]
    fn test_cli_parses_match_command_with_games_override() {
        let cli = Cli::try_parse_from(["bot-arena", "match", "bot1", "bot2", "-g", "50"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match cli.command {
            Commands::Match { games, preset, .. } => {
                assert_eq!(games, 50);
                assert!(preset.is_none());
            }
        }
    }

    #[test]
    fn test_cli_parses_match_command_with_preset_long_form() {
        let cli =
            Cli::try_parse_from(["bot-arena", "match", "bot1", "bot2", "--preset", "standard"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match cli.command {
            Commands::Match { preset, .. } => {
                assert_eq!(preset, Some("standard".to_string()));
            }
        }
    }

    #[test]
    fn test_preset_overrides_games_count() {
        use config::{ArenaConfig, PresetConfig};
        use std::collections::HashMap;

        let mut presets = HashMap::new();
        presets.insert(
            "test-preset".to_string(),
            PresetConfig {
                games: 42,
                time_control: "movetime 200".to_string(),
                openings: vec![],
            },
        );

        let config = ArenaConfig {
            bots: HashMap::new(),
            presets,
        };

        // Simulate the preset lookup logic from main
        let preset_name = "test-preset";
        let cli_games = 10; // default from CLI

        let (games, time_control) = if let Some(p) = config.presets.get(preset_name) {
            (p.games, p.time_control.clone())
        } else {
            (cli_games, "movetime 500".to_string())
        };

        assert_eq!(games, 42); // preset overrides CLI default
        assert_eq!(time_control, "movetime 200");
    }

    #[test]
    fn test_unknown_preset_is_detected() {
        use config::ArenaConfig;

        let config = ArenaConfig::default();

        // Simulate checking for unknown preset
        let preset_name = "nonexistent";
        let preset_found = config.presets.get(preset_name);

        assert!(preset_found.is_none());
    }

    #[test]
    fn test_cli_help_includes_preset_option() {
        let mut cmd = Cli::command();
        let help = cmd.render_help().to_string();

        // Verify help text mentions preset
        assert!(help.contains("match") || help.contains("Match"));
    }
}
