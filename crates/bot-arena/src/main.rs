mod config;
mod game_runner;
mod json_output;
mod pgn;
mod storage;
mod uci_client;

use chess_analysis::{AnalysisConfig, GameAnalysis, GameAnalyzer, MoveInput};
use clap::{Parser, Subcommand};
use config::ArenaConfig;
use game_runner::{GameRunner, MatchResult};
use serde::Deserialize;
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
    /// Analyze a game with Stockfish
    Analyze {
        /// Game ID to analyze
        #[arg(long)]
        game_id: String,
        /// Path to Stockfish engine (uses config or default if not specified)
        #[arg(long)]
        engine: Option<String>,
        /// Analysis depth
        #[arg(long, default_value = "15")]
        depth: u32,
        /// Number of opening book moves to skip
        #[arg(long, default_value = "0")]
        book_moves: usize,
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
                    std::process::exit(1);
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
        Commands::Analyze {
            game_id,
            engine,
            depth,
            book_moves,
        } => {
            run_analyze(&config, &game_id, engine, depth, book_moves);
        }
    }
}

/// Structure for deserializing game JSON files.
#[derive(Debug, Deserialize)]
struct GameJson {
    id: String,
    white: String,
    black: String,
    result: String,
    moves: Vec<MoveRecordJson>,
}

/// Structure for deserializing move records from JSON.
#[derive(Debug, Deserialize)]
struct MoveRecordJson {
    uci: String,
    search_info: Option<SearchInfoJson>,
}

/// Structure for deserializing search info from JSON.
#[derive(Debug, Deserialize)]
struct SearchInfoJson {
    depth: Option<u32>,
    score_cp: Option<i32>,
    score_mate: Option<i32>,
    nodes: Option<u64>,
    time_ms: Option<u64>,
    pv: Option<Vec<String>>,
}

/// Finds a game JSON file by ID in the data/games directory.
fn find_game_file(game_id: &str) -> Option<std::path::PathBuf> {
    let pattern = format!("data/games/*/{}.json", game_id);
    glob::glob(&pattern).ok()?.flatten().next()
}

/// Loads a game from its JSON file.
fn load_game(path: &std::path::Path) -> Result<GameJson, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read game file: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse game JSON: {}", e))
}

/// Converts JSON move records to MoveInput format for analysis.
fn convert_moves(moves: &[MoveRecordJson]) -> Vec<MoveInput> {
    moves
        .iter()
        .map(|m| {
            let (bot_eval_cp, bot_eval_mate, bot_depth, bot_nodes, bot_time_ms, bot_pv) =
                if let Some(ref info) = m.search_info {
                    (
                        info.score_cp,
                        info.score_mate,
                        info.depth,
                        info.nodes,
                        info.time_ms,
                        info.pv.clone().unwrap_or_default(),
                    )
                } else {
                    (None, None, None, None, None, vec![])
                };

            MoveInput {
                uci: m.uci.clone(),
                bot_eval_cp,
                bot_eval_mate,
                bot_depth,
                bot_nodes,
                bot_time_ms,
                bot_pv,
            }
        })
        .collect()
}

/// Prints analysis results to stdout.
fn print_analysis_results(analysis: &GameAnalysis) {
    println!("\n=== Game Analysis ===");
    println!("Game ID: {}", analysis.game_id);
    println!(
        "White: {} vs Black: {}",
        analysis.white_bot, analysis.black_bot
    );
    println!("Result: {}", analysis.result);
    println!();

    println!("White ({}):", analysis.white_bot);
    println!("  Accuracy: {:.1}%", analysis.white_stats.accuracy_percent);
    println!(
        "  Avg Centipawn Loss: {:.1}",
        analysis.white_stats.avg_centipawn_loss
    );
    println!("  Blunders: {}", analysis.white_stats.blunders);
    println!("  Mistakes: {}", analysis.white_stats.mistakes);
    println!("  Inaccuracies: {}", analysis.white_stats.inaccuracies);
    println!();

    println!("Black ({}):", analysis.black_bot);
    println!("  Accuracy: {:.1}%", analysis.black_stats.accuracy_percent);
    println!(
        "  Avg Centipawn Loss: {:.1}",
        analysis.black_stats.avg_centipawn_loss
    );
    println!("  Blunders: {}", analysis.black_stats.blunders);
    println!("  Mistakes: {}", analysis.black_stats.mistakes);
    println!("  Inaccuracies: {}", analysis.black_stats.inaccuracies);
}

/// Saves analysis results to JSON file.
fn save_analysis(game_id: &str, analysis: &GameAnalysis) -> Result<(), String> {
    let analysis_dir = "data/analysis";
    std::fs::create_dir_all(analysis_dir)
        .map_err(|e| format!("Failed to create analysis directory: {}", e))?;

    let path = format!("{}/{}.json", analysis_dir, game_id);
    let file = std::fs::File::create(&path)
        .map_err(|e| format!("Failed to create analysis file: {}", e))?;
    serde_json::to_writer_pretty(file, analysis)
        .map_err(|e| format!("Failed to write analysis JSON: {}", e))?;

    println!("\nAnalysis saved to: {}", path);
    Ok(())
}

/// Runs the analyze command.
fn run_analyze(
    config: &ArenaConfig,
    game_id: &str,
    engine_override: Option<String>,
    depth: u32,
    book_moves: usize,
) {
    // Determine engine path
    let engine_path = engine_override.unwrap_or_else(|| config.stockfish_path.clone());

    // Find and load game
    let game_path = match find_game_file(game_id) {
        Some(path) => path,
        None => {
            eprintln!("Error: Game not found: {}", game_id);
            eprintln!("Searched in: data/games/*/");
            std::process::exit(1);
        }
    };

    println!("Loading game from: {:?}", game_path);

    let game = match load_game(&game_path) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "Analyzing game: {} vs {} ({} moves)",
        game.white,
        game.black,
        game.moves.len()
    );
    println!("Using engine: {}", engine_path);
    println!("Depth: {}, Book moves: {}", depth, book_moves);

    // Create analyzer
    let analysis_config = AnalysisConfig {
        depth,
        opening_book_moves: book_moves,
    };

    let mut analyzer = match GameAnalyzer::new(&engine_path, analysis_config) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: Failed to initialize analyzer: {}", e);
            eprintln!(
                "Make sure Stockfish is installed and accessible at: {}",
                engine_path
            );
            std::process::exit(1);
        }
    };

    // Convert moves
    let moves = convert_moves(&game.moves);

    // Run analysis
    println!("\nAnalyzing {} moves...", moves.len());
    let analysis =
        match analyzer.analyze_game(&game.id, &game.white, &game.black, &moves, &game.result) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Error: Analysis failed: {}", e);
                std::process::exit(1);
            }
        };

    // Print results
    print_analysis_results(&analysis);

    // Save analysis
    if let Err(e) = save_analysis(&game.id, &analysis) {
        eprintln!("Warning: {}", e);
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
            _ => panic!("Expected Match command"),
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
            _ => panic!("Expected Match command"),
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
            _ => panic!("Expected Match command"),
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
            _ => panic!("Expected Match command"),
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
            stockfish_path: "stockfish".to_string(),
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
        let cmd = Cli::command();
        let match_cmd = cmd
            .get_subcommands()
            .find(|c| c.get_name() == "match")
            .expect("match subcommand exists");
        let help = match_cmd.clone().render_help().to_string();

        // Verify help text mentions preset option
        assert!(help.contains("preset") || help.contains("-p"));
    }

    #[test]
    fn test_cli_parses_analyze_command() {
        let cli = Cli::try_parse_from(["bot-arena", "analyze", "--game-id", "test-game-123"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match cli.command {
            Commands::Analyze {
                game_id,
                engine,
                depth,
                book_moves,
            } => {
                assert_eq!(game_id, "test-game-123");
                assert!(engine.is_none());
                assert_eq!(depth, 15); // default
                assert_eq!(book_moves, 0); // default
            }
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn test_cli_parses_analyze_command_with_all_options() {
        let cli = Cli::try_parse_from([
            "bot-arena",
            "analyze",
            "--game-id",
            "game-456",
            "--engine",
            "/usr/bin/stockfish",
            "--depth",
            "20",
            "--book-moves",
            "10",
        ]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match cli.command {
            Commands::Analyze {
                game_id,
                engine,
                depth,
                book_moves,
            } => {
                assert_eq!(game_id, "game-456");
                assert_eq!(engine, Some("/usr/bin/stockfish".to_string()));
                assert_eq!(depth, 20);
                assert_eq!(book_moves, 10);
            }
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn test_cli_analyze_help_includes_options() {
        let cmd = Cli::command();
        let analyze_cmd = cmd
            .get_subcommands()
            .find(|c| c.get_name() == "analyze")
            .expect("analyze subcommand exists");
        let help = analyze_cmd.clone().render_help().to_string();

        assert!(help.contains("game-id"));
        assert!(help.contains("engine"));
        assert!(help.contains("depth"));
        assert!(help.contains("book-moves"));
    }

    #[test]
    fn test_convert_moves_with_search_info() {
        let moves = vec![
            MoveRecordJson {
                uci: "e2e4".to_string(),
                search_info: Some(SearchInfoJson {
                    depth: Some(15),
                    score_cp: Some(35),
                    score_mate: None,
                    nodes: Some(100000),
                    time_ms: Some(500),
                    pv: Some(vec!["e2e4".to_string(), "e7e5".to_string()]),
                }),
            },
            MoveRecordJson {
                uci: "e7e5".to_string(),
                search_info: None,
            },
        ];

        let converted = convert_moves(&moves);

        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].uci, "e2e4");
        assert_eq!(converted[0].bot_eval_cp, Some(35));
        assert_eq!(converted[0].bot_depth, Some(15));
        assert_eq!(converted[0].bot_nodes, Some(100000));
        assert_eq!(converted[0].bot_time_ms, Some(500));
        assert_eq!(
            converted[0].bot_pv,
            vec!["e2e4".to_string(), "e7e5".to_string()]
        );

        assert_eq!(converted[1].uci, "e7e5");
        assert!(converted[1].bot_eval_cp.is_none());
        assert!(converted[1].bot_depth.is_none());
        assert!(converted[1].bot_pv.is_empty());
    }

    #[test]
    fn test_convert_moves_empty() {
        let moves: Vec<MoveRecordJson> = vec![];
        let converted = convert_moves(&moves);
        assert!(converted.is_empty());
    }

    #[test]
    fn test_find_game_file_not_found() {
        // This should return None for a non-existent game
        let result = find_game_file("non-existent-game-id-12345");
        assert!(result.is_none());
    }
}
