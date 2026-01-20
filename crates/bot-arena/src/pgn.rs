//! PGN (Portable Game Notation) file generation for chess games.
//!
//! This module provides functionality to export completed games to the standard
//! PGN format, which can be read by most chess software and databases.

use crate::game_runner::{GameResult, MatchResult};
use chrono::Utc;
use std::io::Write;
use std::path::Path;

/// Writes a completed game result to a PGN file.
///
/// Creates a PGN file containing the game metadata (event, site, date, players,
/// result) and the complete move list in a human-readable format.
///
/// # Arguments
///
/// * `path` - The filesystem path where the PGN file should be written.
/// * `result` - The completed game result containing player names, moves, and outcome.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an `std::io::Error` if the file cannot be
/// created or written to.
///
/// # File Format
///
/// The generated PGN file follows the standard format:
/// - Seven Tag Roster headers (Event, Site, Date, White, Black, Result)
/// - Blank line separator
/// - Move text with move numbers (e.g., "1. e2e4 e7e5 2. g1f3 ...")
/// - Result terminator
///
/// Note: Moves are currently written in UCI notation. SAN (Standard Algebraic
/// Notation) conversion may be added in a future version.
///
/// # Example
///
/// ```ignore
/// use bot_arena::pgn::write_pgn;
/// use bot_arena::game_runner::{GameResult, MatchResult};
///
/// let result = GameResult {
///     moves: vec!["e2e4".to_string(), "e7e5".to_string()],
///     result: MatchResult::WhiteWins,
///     white_name: "Engine A".to_string(),
///     black_name: "Engine B".to_string(),
/// };
///
/// write_pgn("game.pgn", &result)?;
/// ```
pub fn write_pgn<P: AsRef<Path>>(path: P, result: &GameResult) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;

    let result_str = match result.result {
        MatchResult::WhiteWins => "1-0",
        MatchResult::BlackWins => "0-1",
        MatchResult::Draw => "1/2-1/2",
    };

    writeln!(file, "[Event \"Bot Arena Match\"]")?;
    writeln!(file, "[Site \"local\"]")?;
    writeln!(file, "[Date \"{}\"]", Utc::now().format("%Y.%m.%d"))?;
    writeln!(file, "[White \"{}\"]", result.white_name)?;
    writeln!(file, "[Black \"{}\"]", result.black_name)?;
    writeln!(file, "[Result \"{}\"]", result_str)?;
    writeln!(file)?;

    // Write moves in PGN format (UCI for now, SAN conversion later)
    let mut move_text = String::new();
    for (i, mv) in result.moves.iter().enumerate() {
        if i % 2 == 0 {
            move_text.push_str(&format!("{}. ", i / 2 + 1));
        }
        move_text.push_str(mv);
        move_text.push(' ');
    }
    move_text.push_str(result_str);

    // Wrap at 80 chars
    for chunk in move_text.as_bytes().chunks(80) {
        file.write_all(chunk)?;
        writeln!(file)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    /// Helper function to create a test game result.
    fn create_test_result() -> GameResult {
        GameResult {
            moves: vec![
                "e2e4".to_string(),
                "e7e5".to_string(),
                "g1f3".to_string(),
                "b8c6".to_string(),
                "f1b5".to_string(),
            ],
            result: MatchResult::WhiteWins,
            white_name: "TestEngineWhite".to_string(),
            black_name: "TestEngineBlack".to_string(),
        }
    }

    #[test]
    fn test_write_pgn_creates_valid_file() {
        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_game.pgn");

        let result = create_test_result();
        write_pgn(&pgn_path, &result).expect("Failed to write PGN file");

        // Verify file exists
        assert!(pgn_path.exists(), "PGN file should be created");

        // Read and verify contents
        let mut contents = String::new();
        fs::File::open(&pgn_path)
            .expect("Failed to open PGN file")
            .read_to_string(&mut contents)
            .expect("Failed to read PGN file");

        // Verify headers
        assert!(
            contents.contains("[Event \"Bot Arena Match\"]"),
            "Should contain Event header"
        );
        assert!(
            contents.contains("[Site \"local\"]"),
            "Should contain Site header"
        );
        assert!(contents.contains("[Date \""), "Should contain Date header");
        assert!(
            contents.contains("[White \"TestEngineWhite\"]"),
            "Should contain White header"
        );
        assert!(
            contents.contains("[Black \"TestEngineBlack\"]"),
            "Should contain Black header"
        );
        assert!(
            contents.contains("[Result \"1-0\"]"),
            "Should contain Result header"
        );

        // Verify result terminator in move text
        assert!(
            contents.contains("1-0"),
            "Should contain result in move text"
        );

        // Cleanup
        fs::remove_file(&pgn_path).ok();
    }

    #[test]
    fn test_write_pgn_formats_moves_correctly() {
        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_moves.pgn");

        let result = create_test_result();
        write_pgn(&pgn_path, &result).expect("Failed to write PGN file");

        // Read contents
        let mut contents = String::new();
        fs::File::open(&pgn_path)
            .expect("Failed to open PGN file")
            .read_to_string(&mut contents)
            .expect("Failed to read PGN file");

        // Verify move numbering
        assert!(
            contents.contains("1. e2e4 e7e5"),
            "Should contain move 1 with correct numbering"
        );
        assert!(
            contents.contains("2. g1f3 b8c6"),
            "Should contain move 2 with correct numbering"
        );
        assert!(
            contents.contains("3. f1b5"),
            "Should contain move 3 with correct numbering"
        );

        // Cleanup
        fs::remove_file(&pgn_path).ok();
    }

    #[test]
    fn test_write_pgn_black_wins() {
        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_black_wins.pgn");

        let result = GameResult {
            moves: vec!["e2e4".to_string(), "e7e5".to_string()],
            result: MatchResult::BlackWins,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
        };
        write_pgn(&pgn_path, &result).expect("Failed to write PGN file");

        let mut contents = String::new();
        fs::File::open(&pgn_path)
            .expect("Failed to open PGN file")
            .read_to_string(&mut contents)
            .expect("Failed to read PGN file");

        assert!(
            contents.contains("[Result \"0-1\"]"),
            "Should contain 0-1 result header"
        );
        assert!(
            contents.contains("0-1"),
            "Should contain 0-1 result terminator"
        );

        fs::remove_file(&pgn_path).ok();
    }

    #[test]
    fn test_write_pgn_draw() {
        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_draw.pgn");

        let result = GameResult {
            moves: vec!["e2e4".to_string()],
            result: MatchResult::Draw,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
        };
        write_pgn(&pgn_path, &result).expect("Failed to write PGN file");

        let mut contents = String::new();
        fs::File::open(&pgn_path)
            .expect("Failed to open PGN file")
            .read_to_string(&mut contents)
            .expect("Failed to read PGN file");

        assert!(
            contents.contains("[Result \"1/2-1/2\"]"),
            "Should contain draw result header"
        );
        assert!(
            contents.contains("1/2-1/2"),
            "Should contain draw result terminator"
        );

        fs::remove_file(&pgn_path).ok();
    }

    #[test]
    fn test_write_pgn_empty_moves() {
        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_empty.pgn");

        let result = GameResult {
            moves: vec![],
            result: MatchResult::Draw,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
        };
        write_pgn(&pgn_path, &result).expect("Failed to write PGN file");

        let mut contents = String::new();
        fs::File::open(&pgn_path)
            .expect("Failed to open PGN file")
            .read_to_string(&mut contents)
            .expect("Failed to read PGN file");

        // Should still have headers and result
        assert!(
            contents.contains("[Result \"1/2-1/2\"]"),
            "Should contain result header"
        );

        fs::remove_file(&pgn_path).ok();
    }
}
