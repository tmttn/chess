//! PGN (Portable Game Notation) file generation for chess games.
//!
//! This module provides functionality to export completed games to the standard
//! PGN format, which can be read by most chess software and databases.

#[cfg(test)]
use crate::game_runner::MoveRecord;
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
    writeln!(file, "[Round \"-\"]")?;
    writeln!(file, "[White \"{}\"]", result.white_name)?;
    writeln!(file, "[Black \"{}\"]", result.black_name)?;
    writeln!(file, "[Result \"{}\"]", result_str)?;

    // Add optional opening headers if detected
    if let Some(opening) = &result.opening {
        writeln!(file, "[Opening \"{}\"]", opening.name)?;
        if let Some(eco) = &opening.eco {
            writeln!(file, "[ECO \"{}\"]", eco)?;
        }
    }

    writeln!(file)?;

    // Write moves in PGN format (UCI for now, SAN conversion later)
    let mut move_text = String::new();
    for (i, record) in result.moves.iter().enumerate() {
        if i % 2 == 0 {
            move_text.push_str(&format!("{}. ", i / 2 + 1));
        }
        move_text.push_str(&record.uci);
        move_text.push(' ');
    }
    move_text.push_str(result_str);

    // Wrap at 80 chars at word boundaries
    let mut line = String::new();
    for word in move_text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > 80 {
            writeln!(file, "{}", line)?;
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        writeln!(file, "{}", line)?;
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
                MoveRecord {
                    uci: "e2e4".to_string(),
                    search_info: None,
                },
                MoveRecord {
                    uci: "e7e5".to_string(),
                    search_info: None,
                },
                MoveRecord {
                    uci: "g1f3".to_string(),
                    search_info: None,
                },
                MoveRecord {
                    uci: "b8c6".to_string(),
                    search_info: None,
                },
                MoveRecord {
                    uci: "f1b5".to_string(),
                    search_info: None,
                },
            ],
            result: MatchResult::WhiteWins,
            white_name: "TestEngineWhite".to_string(),
            black_name: "TestEngineBlack".to_string(),
            opening: None,
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
            contents.contains("[Round \"-\"]"),
            "Should contain Round header"
        );
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

        // Verify word-boundary wrapping doesn't split tokens
        for line in contents.lines() {
            // Skip header lines
            if line.starts_with('[') || line.is_empty() {
                continue;
            }
            // Verify no line exceeds 80 chars (except if a single word is longer)
            assert!(
                line.len() <= 80 || !line.contains(' '),
                "Line should not exceed 80 chars: {} (len: {})",
                line,
                line.len()
            );
        }

        // Cleanup
        fs::remove_file(&pgn_path).ok();
    }

    #[test]
    fn test_write_pgn_wraps_long_lines() {
        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_long_game.pgn");

        // Create a game with many moves to trigger line wrapping
        let moves: Vec<MoveRecord> = (0..100)
            .map(|i| {
                let uci = if i % 2 == 0 {
                    format!("e{}e{}", (i % 8) + 1, (i % 8) + 2)
                } else {
                    format!("d{}d{}", (i % 8) + 1, (i % 8) + 2)
                };
                MoveRecord {
                    uci,
                    search_info: None,
                }
            })
            .collect();

        let result = GameResult {
            moves,
            result: MatchResult::Draw,
            white_name: "LongGameWhite".to_string(),
            black_name: "LongGameBlack".to_string(),
            opening: None,
        };
        write_pgn(&pgn_path, &result).expect("Failed to write PGN file");

        // Read contents
        let mut contents = String::new();
        fs::File::open(&pgn_path)
            .expect("Failed to open PGN file")
            .read_to_string(&mut contents)
            .expect("Failed to read PGN file");

        // Verify no line exceeds 80 chars
        for line in contents.lines() {
            assert!(
                line.len() <= 80,
                "Line exceeds 80 chars: {} (len: {})",
                line,
                line.len()
            );
        }

        // Verify that moves are not split mid-token
        // The draw result "1/2-1/2" should be on a single line, not split
        assert!(
            contents.contains("1/2-1/2"),
            "Draw result should not be split across lines"
        );

        // Verify tokens are not split mid-word (e.g., "1/2-1/2" should not become "1/2-1" and "/2")
        // Note: Move numbers like "7." ending a line is acceptable as they are complete tokens
        for line in contents.lines() {
            if line.starts_with('[') || line.is_empty() {
                continue;
            }
            // Check that no partial results appear (would indicate mid-token split)
            // A properly formatted line should not have partial fractions
            assert!(
                !line.contains("1/2-1\n") && !line.ends_with("1/2-1"),
                "Line should not contain split draw result: {}",
                line
            );
        }

        // Cleanup
        fs::remove_file(&pgn_path).ok();
    }

    #[test]
    fn test_write_pgn_black_wins() {
        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_black_wins.pgn");

        let result = GameResult {
            moves: vec![
                MoveRecord {
                    uci: "e2e4".to_string(),
                    search_info: None,
                },
                MoveRecord {
                    uci: "e7e5".to_string(),
                    search_info: None,
                },
            ],
            result: MatchResult::BlackWins,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
            opening: None,
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
            moves: vec![MoveRecord {
                uci: "e2e4".to_string(),
                search_info: None,
            }],
            result: MatchResult::Draw,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
            opening: None,
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
            opening: None,
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

    #[test]
    fn test_write_pgn_with_opening_headers() {
        use crate::game_runner::DetectedOpening;

        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_with_opening.pgn");

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
        write_pgn(&pgn_path, &result).expect("Failed to write PGN file");

        let mut contents = String::new();
        fs::File::open(&pgn_path)
            .expect("Failed to open PGN file")
            .read_to_string(&mut contents)
            .expect("Failed to read PGN file");

        // Verify opening headers
        assert!(
            contents.contains("[Opening \"Italian Game\"]"),
            "Should contain Opening header"
        );
        assert!(
            contents.contains("[ECO \"C50\"]"),
            "Should contain ECO header"
        );

        fs::remove_file(&pgn_path).ok();
    }

    #[test]
    fn test_write_pgn_with_opening_no_eco() {
        use crate::game_runner::DetectedOpening;

        let temp_dir = std::env::temp_dir();
        let pgn_path = temp_dir.join("test_opening_no_eco.pgn");

        let result = GameResult {
            moves: vec![MoveRecord { uci: "e2e4".to_string(), search_info: None }],
            result: MatchResult::Draw,
            white_name: "Engine1".to_string(),
            black_name: "Engine2".to_string(),
            opening: Some(DetectedOpening {
                id: "custom-opening".to_string(),
                name: "Custom Opening".to_string(),
                eco: None,
            }),
        };
        write_pgn(&pgn_path, &result).expect("Failed to write PGN file");

        let mut contents = String::new();
        fs::File::open(&pgn_path)
            .expect("Failed to open PGN file")
            .read_to_string(&mut contents)
            .expect("Failed to read PGN file");

        // Verify opening header present, ECO absent
        assert!(
            contents.contains("[Opening \"Custom Opening\"]"),
            "Should contain Opening header"
        );
        assert!(
            !contents.contains("[ECO"),
            "Should not contain ECO header when None"
        );

        fs::remove_file(&pgn_path).ok();
    }
}
