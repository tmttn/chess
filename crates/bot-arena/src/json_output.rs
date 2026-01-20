//! JSON file generation for chess game results with UCI search information.
//!
//! This module provides functionality to export completed games to JSON format,
//! including detailed search information from the UCI engines for each move.
//! This is useful for analysis, machine learning, and detailed game review.

use crate::game_runner::{DetectedOpening, GameResult, MatchResult, MoveRecord};
use chrono::Utc;
use serde::Serialize;
use std::path::Path;

/// JSON representation of a complete game.
///
/// This struct is used for serialization and contains all game metadata
/// along with the full move list including engine search information.
#[derive(Serialize)]
struct GameJson<'a> {
    /// Unique identifier for the game.
    id: &'a str,
    /// Name of the engine playing white.
    white: &'a str,
    /// Name of the engine playing black.
    black: &'a str,
    /// Game result: "white", "black", or "draw".
    result: &'a str,
    /// Detected opening information, if recognized.
    #[serde(skip_serializing_if = "Option::is_none")]
    opening: Option<&'a DetectedOpening>,
    /// Complete move list with search information.
    moves: &'a [MoveRecord],
    /// ISO 8601 timestamp when the file was created.
    created_at: String,
}

/// Writes a completed game result to a JSON file with full search information.
///
/// Creates a JSON file containing the game metadata (id, players, result) and
/// the complete move list with UCI search information (depth, score, nodes, PV)
/// for each move.
///
/// # Arguments
///
/// * `path` - The filesystem path where the JSON file should be written.
/// * `id` - A unique identifier for this game (typically a UUID).
/// * `result` - The completed game result containing player names, moves, and outcome.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an `std::io::Error` if the file cannot be
/// created or written to.
///
/// # File Format
///
/// The generated JSON file has the following structure:
/// ```json
/// {
///   "id": "game-uuid",
///   "white": "Engine A",
///   "black": "Engine B",
///   "result": "white",
///   "moves": [
///     {
///       "uci": "e2e4",
///       "search_info": {
///         "depth": 20,
///         "score_cp": 35,
///         "score_mate": null,
///         "nodes": 1234567,
///         "time_ms": 1000,
///         "pv": ["e2e4", "e7e5", "g1f3"]
///       }
///     }
///   ],
///   "created_at": "2024-01-15T12:00:00Z"
/// }
/// ```
///
/// # Example
///
/// ```ignore
/// use bot_arena::json_output::write_json;
/// use bot_arena::game_runner::{GameResult, MatchResult, MoveRecord};
///
/// let result = GameResult {
///     moves: vec![MoveRecord { uci: "e2e4".to_string(), search_info: None }],
///     result: MatchResult::WhiteWins,
///     white_name: "Engine A".to_string(),
///     black_name: "Engine B".to_string(),
/// };
///
/// write_json("game.json", "unique-id", &result)?;
/// ```
pub fn write_json<P: AsRef<Path>>(path: P, id: &str, result: &GameResult) -> std::io::Result<()> {
    let result_str = match result.result {
        MatchResult::WhiteWins => "white",
        MatchResult::BlackWins => "black",
        MatchResult::Draw => "draw",
    };

    let json = GameJson {
        id,
        white: &result.white_name,
        black: &result.black_name,
        result: result_str,
        opening: result.opening.as_ref(),
        moves: &result.moves,
        created_at: Utc::now().to_rfc3339(),
    };

    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uci_client::SearchInfo;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_write_json_creates_valid_file() {
        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_game.json");

        let result = GameResult {
            moves: vec![
                MoveRecord {
                    uci: "e2e4".to_string(),
                    search_info: Some(SearchInfo {
                        depth: Some(20),
                        score_cp: Some(35),
                        score_mate: None,
                        nodes: Some(1234567),
                        time_ms: Some(1000),
                        pv: vec!["e2e4".to_string(), "e7e5".to_string()],
                    }),
                },
                MoveRecord {
                    uci: "e7e5".to_string(),
                    search_info: Some(SearchInfo {
                        depth: Some(18),
                        score_cp: Some(-30),
                        score_mate: None,
                        nodes: Some(987654),
                        time_ms: Some(950),
                        pv: vec!["e7e5".to_string(), "g1f3".to_string()],
                    }),
                },
            ],
            result: MatchResult::WhiteWins,
            white_name: "TestWhite".to_string(),
            black_name: "TestBlack".to_string(),
            opening: None,
        };

        write_json(&json_path, "test-game-id", &result).expect("Failed to write JSON file");

        // Verify file exists
        assert!(json_path.exists(), "JSON file should be created");

        // Read and verify contents
        let mut contents = String::new();
        fs::File::open(&json_path)
            .expect("Failed to open JSON file")
            .read_to_string(&mut contents)
            .expect("Failed to read JSON file");

        // Verify JSON structure
        assert!(contents.contains("\"id\":"), "Should contain id field");
        assert!(
            contents.contains("\"test-game-id\""),
            "Should contain correct id value"
        );
        assert!(
            contents.contains("\"white\":"),
            "Should contain white field"
        );
        assert!(
            contents.contains("\"TestWhite\""),
            "Should contain white player name"
        );
        assert!(
            contents.contains("\"black\":"),
            "Should contain black field"
        );
        assert!(
            contents.contains("\"TestBlack\""),
            "Should contain black player name"
        );
        assert!(
            contents.contains("\"result\":"),
            "Should contain result field"
        );
        assert!(
            contents.contains("\"white\""),
            "Should contain white as result"
        );
        assert!(
            contents.contains("\"moves\":"),
            "Should contain moves field"
        );
        assert!(
            contents.contains("\"created_at\":"),
            "Should contain created_at field"
        );

        // Verify search info is included
        assert!(
            contents.contains("\"depth\":"),
            "Should contain depth in search info"
        );
        assert!(
            contents.contains("\"score_cp\":"),
            "Should contain score_cp in search info"
        );
        assert!(
            contents.contains("\"nodes\":"),
            "Should contain nodes in search info"
        );
        assert!(
            contents.contains("\"pv\":"),
            "Should contain pv in search info"
        );

        // Verify it's valid JSON by parsing it
        let parsed: serde_json::Value =
            serde_json::from_str(&contents).expect("Should be valid JSON");
        assert_eq!(parsed["id"], "test-game-id");
        assert_eq!(parsed["white"], "TestWhite");
        assert_eq!(parsed["black"], "TestBlack");
        assert_eq!(parsed["result"], "white");
        assert_eq!(parsed["moves"].as_array().unwrap().len(), 2);

        // Cleanup
        fs::remove_file(&json_path).ok();
    }

    #[test]
    fn test_write_json_black_wins() {
        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_black_wins.json");

        let result = GameResult {
            moves: vec![MoveRecord {
                uci: "e2e4".to_string(),
                search_info: None,
            }],
            result: MatchResult::BlackWins,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
            opening: None,
        };

        write_json(&json_path, "black-wins-id", &result).expect("Failed to write JSON file");

        let mut contents = String::new();
        fs::File::open(&json_path)
            .expect("Failed to open JSON file")
            .read_to_string(&mut contents)
            .expect("Failed to read JSON file");

        let parsed: serde_json::Value =
            serde_json::from_str(&contents).expect("Should be valid JSON");
        assert_eq!(parsed["result"], "black");

        fs::remove_file(&json_path).ok();
    }

    #[test]
    fn test_write_json_draw() {
        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_draw.json");

        let result = GameResult {
            moves: vec![],
            result: MatchResult::Draw,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
            opening: None,
        };

        write_json(&json_path, "draw-id", &result).expect("Failed to write JSON file");

        let mut contents = String::new();
        fs::File::open(&json_path)
            .expect("Failed to open JSON file")
            .read_to_string(&mut contents)
            .expect("Failed to read JSON file");

        let parsed: serde_json::Value =
            serde_json::from_str(&contents).expect("Should be valid JSON");
        assert_eq!(parsed["result"], "draw");
        assert_eq!(parsed["moves"].as_array().unwrap().len(), 0);

        fs::remove_file(&json_path).ok();
    }

    #[test]
    fn test_write_json_with_null_search_info() {
        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_null_info.json");

        let result = GameResult {
            moves: vec![MoveRecord {
                uci: "g1f3".to_string(),
                search_info: None,
            }],
            result: MatchResult::WhiteWins,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
            opening: None,
        };

        write_json(&json_path, "null-info-id", &result).expect("Failed to write JSON file");

        let mut contents = String::new();
        fs::File::open(&json_path)
            .expect("Failed to open JSON file")
            .read_to_string(&mut contents)
            .expect("Failed to read JSON file");

        let parsed: serde_json::Value =
            serde_json::from_str(&contents).expect("Should be valid JSON");
        assert!(parsed["moves"][0]["search_info"].is_null());

        fs::remove_file(&json_path).ok();
    }

    #[test]
    fn test_write_json_with_mate_score() {
        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_mate_score.json");

        let result = GameResult {
            moves: vec![MoveRecord {
                uci: "d1h5".to_string(),
                search_info: Some(SearchInfo {
                    depth: Some(25),
                    score_cp: None,
                    score_mate: Some(3),
                    nodes: Some(500000),
                    time_ms: Some(2000),
                    pv: vec!["d1h5".to_string(), "g7g6".to_string(), "h5f7".to_string()],
                }),
            }],
            result: MatchResult::WhiteWins,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
            opening: None,
        };

        write_json(&json_path, "mate-score-id", &result).expect("Failed to write JSON file");

        let mut contents = String::new();
        fs::File::open(&json_path)
            .expect("Failed to open JSON file")
            .read_to_string(&mut contents)
            .expect("Failed to read JSON file");

        let parsed: serde_json::Value =
            serde_json::from_str(&contents).expect("Should be valid JSON");
        assert_eq!(parsed["moves"][0]["search_info"]["score_mate"], 3);
        assert!(parsed["moves"][0]["search_info"]["score_cp"].is_null());

        fs::remove_file(&json_path).ok();
    }

    #[test]
    fn test_write_json_with_opening() {
        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_with_opening.json");

        let result = GameResult {
            moves: vec![
                MoveRecord { uci: "e2e4".to_string(), search_info: None },
                MoveRecord { uci: "e7e5".to_string(), search_info: None },
                MoveRecord { uci: "g1f3".to_string(), search_info: None },
                MoveRecord { uci: "b8c6".to_string(), search_info: None },
                MoveRecord { uci: "f1c4".to_string(), search_info: None },
            ],
            result: MatchResult::WhiteWins,
            white_name: "Minimax".to_string(),
            black_name: "Random".to_string(),
            opening: Some(DetectedOpening {
                id: "italian-game".to_string(),
                name: "Italian Game".to_string(),
                eco: Some("C50".to_string()),
            }),
        };

        write_json(&json_path, "opening-test-id", &result).expect("Failed to write JSON file");

        let mut contents = String::new();
        fs::File::open(&json_path)
            .expect("Failed to open JSON file")
            .read_to_string(&mut contents)
            .expect("Failed to read JSON file");

        let parsed: serde_json::Value =
            serde_json::from_str(&contents).expect("Should be valid JSON");

        // Verify opening is included
        assert!(!parsed["opening"].is_null(), "Opening should be present");
        assert_eq!(parsed["opening"]["id"], "italian-game");
        assert_eq!(parsed["opening"]["name"], "Italian Game");
        assert_eq!(parsed["opening"]["eco"], "C50");

        fs::remove_file(&json_path).ok();
    }

    #[test]
    fn test_write_json_opening_is_omitted_when_none() {
        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_no_opening.json");

        let result = GameResult {
            moves: vec![MoveRecord { uci: "e2e4".to_string(), search_info: None }],
            result: MatchResult::Draw,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
            opening: None,
        };

        write_json(&json_path, "no-opening-id", &result).expect("Failed to write JSON file");

        let mut contents = String::new();
        fs::File::open(&json_path)
            .expect("Failed to open JSON file")
            .read_to_string(&mut contents)
            .expect("Failed to read JSON file");

        // Opening field should be omitted (skip_serializing_if)
        assert!(
            !contents.contains("\"opening\":"),
            "Opening should not be present when None"
        );

        fs::remove_file(&json_path).ok();
    }
}
