//! Game execution logic for running matches between UCI chess engines.
//!
//! This module provides the [`GameRunner`] struct for executing single games
//! between two UCI-compatible chess engines, handling the complete game loop
//! from initialization to result determination.

use chess_core::Color;
use chess_engine::{Game, GameResult as EngineResult};

use crate::uci_client::{SearchInfo, UciClient, UciError};
use thiserror::Error;

/// Errors that can occur during game execution.
///
/// This enum covers UCI communication errors and invalid move errors
/// that can happen while running a game between two engines.
#[derive(Error, Debug)]
pub enum GameError {
    /// An error occurred while communicating with a UCI engine.
    #[error("UCI error: {0}")]
    Uci(#[from] UciError),
    /// An engine returned an invalid or illegal move.
    #[error("Invalid move: {0}")]
    InvalidMove(String),
}

/// A single move with its associated search information.
///
/// This struct captures a move in UCI notation along with the optional
/// search metrics reported by the engine when calculating the move.
///
/// # Example
///
/// ```
/// use bot_arena::game_runner::MoveRecord;
/// use bot_arena::uci_client::SearchInfo;
///
/// let record = MoveRecord {
///     uci: "e2e4".to_string(),
///     search_info: Some(SearchInfo {
///         depth: Some(20),
///         score_cp: Some(35),
///         ..Default::default()
///     }),
/// };
/// ```
#[derive(Debug, Clone, serde::Serialize)]
pub struct MoveRecord {
    /// The move in UCI notation (e.g., "e2e4", "g1f3").
    pub uci: String,
    /// Search information from the engine when calculating this move.
    pub search_info: Option<SearchInfo>,
}

/// Detected opening information for a game.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DetectedOpening {
    /// The opening ID (e.g., "italian-game").
    pub id: String,
    /// The human-readable name (e.g., "Italian Game").
    pub name: String,
    /// The ECO code, if available (e.g., "C50").
    pub eco: Option<String>,
}

/// The result of a completed game, containing move history and outcome.
///
/// This struct captures all relevant information about a finished game,
/// including the sequence of moves played and the final result.
#[derive(Debug, Clone)]
pub struct GameResult {
    /// The sequence of moves played with their search information.
    pub moves: Vec<MoveRecord>,
    /// The outcome of the game.
    pub result: MatchResult,
    /// The name of the engine playing white.
    pub white_name: String,
    /// The name of the engine playing black.
    pub black_name: String,
    /// The detected opening, if any was recognized.
    pub opening: Option<DetectedOpening>,
}

/// The outcome of a chess game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchResult {
    /// White won the game (by checkmate or opponent resignation/error).
    WhiteWins,
    /// Black won the game (by checkmate or opponent resignation/error).
    BlackWins,
    /// The game ended in a draw.
    Draw,
}

/// Executes games between two UCI chess engines.
///
/// `GameRunner` manages two UCI clients and coordinates game play between them,
/// handling position synchronization, move requests, and result determination.
///
/// # Example
///
/// ```ignore
/// let white = UciClient::spawn("./white_engine")?;
/// let black = UciClient::spawn("./black_engine")?;
/// let mut runner = GameRunner::new(white, black, "movetime 500".to_string(), vec![])?;
/// let result = runner.play_game()?;
/// println!("Game result: {:?}", result.result);
/// ```
pub struct GameRunner {
    /// The UCI client for the white player.
    white: UciClient,
    /// The UCI client for the black player.
    black: UciClient,
    /// The time control string to use for move requests.
    time_control: String,
    /// Opening moves to play before the game starts (in UCI notation).
    opening_moves: Vec<String>,
}

impl GameRunner {
    /// Creates a new game runner with the given engines, time control, and optional opening moves.
    ///
    /// Initializes both UCI engines and prepares them for play.
    ///
    /// # Arguments
    ///
    /// * `white` - The UCI client for the white player
    /// * `black` - The UCI client for the black player
    /// * `time_control` - The time control string (e.g., "movetime 500")
    /// * `opening_moves` - Optional opening moves to play at start (in UCI notation)
    ///
    /// # Errors
    ///
    /// Returns an error if either engine fails to initialize.
    pub fn new(
        mut white: UciClient,
        mut black: UciClient,
        time_control: String,
        opening_moves: Vec<String>,
    ) -> Result<Self, GameError> {
        white.init()?;
        black.init()?;
        Ok(Self {
            white,
            black,
            time_control,
            opening_moves,
        })
    }

    /// Plays a complete game between the two engines.
    ///
    /// Executes the game loop, alternating moves between white and black
    /// until the game ends (checkmate, stalemate, draw, or error).
    /// If opening moves were specified, they are played first before
    /// engines start making their own moves.
    ///
    /// # Returns
    ///
    /// Returns a [`GameResult`] containing the move history and outcome.
    ///
    /// # Errors
    ///
    /// Returns an error if an engine produces an invalid move or if
    /// UCI communication fails.
    ///
    /// # Testing
    ///
    /// Integration tests for this method require real UCI engines (e.g., Stockfish).
    /// Unit tests cover the supporting types ([`MoveRecord`], [`GameResult`], [`MatchResult`]).
    pub fn play_game(&mut self) -> Result<GameResult, GameError> {
        let mut game = Game::new();
        let mut moves: Vec<MoveRecord> = Vec::new();
        let white_name = self.white.name.clone();
        let black_name = self.black.name.clone();

        // Play opening moves first
        for opening_move in &self.opening_moves {
            if game.make_move_uci(opening_move).is_err() {
                return Err(GameError::InvalidMove(format!(
                    "Invalid opening move: {}",
                    opening_move
                )));
            }
            moves.push(MoveRecord {
                uci: opening_move.clone(),
                search_info: None,
            });
        }

        loop {
            if game.is_game_over() {
                break;
            }

            let current = if game.position().side_to_move == Color::White {
                &mut self.white
            } else {
                &mut self.black
            };

            // Extract UCI moves for position command
            let uci_moves: Vec<String> = moves.iter().map(|m| m.uci.clone()).collect();
            current.set_position(&uci_moves)?;
            let (bestmove, search_info) = current.go(&self.time_control)?;

            if bestmove.is_empty() || bestmove == "(none)" || bestmove == "0000" {
                break;
            }

            if game.make_move_uci(&bestmove).is_err() {
                return Err(GameError::InvalidMove(bestmove));
            }

            moves.push(MoveRecord {
                uci: bestmove,
                search_info,
            });

            // Safety limit to prevent infinite games
            if moves.len() > 500 {
                break;
            }
        }

        let result = match game.result() {
            Some(EngineResult::WhiteWins) => MatchResult::WhiteWins,
            Some(EngineResult::BlackWins) => MatchResult::BlackWins,
            Some(EngineResult::Draw(_)) | None => MatchResult::Draw,
        };

        Ok(GameResult {
            moves,
            result,
            white_name,
            black_name,
            opening: None, // Opening detection is done separately after game creation
        })
    }
}

/// Detects the opening from a game result using the provided database.
///
/// This function analyzes the move sequence and returns the longest matching
/// opening from the database.
///
/// # Arguments
///
/// * `result` - The game result to analyze
/// * `db` - The opening database to search
///
/// # Returns
///
/// Returns `Some(DetectedOpening)` if an opening was recognized, `None` otherwise.
pub fn detect_opening(
    moves: &[MoveRecord],
    db: &chess_openings::OpeningDatabase,
) -> Option<DetectedOpening> {
    let uci_moves: Vec<String> = moves.iter().map(|m| m.uci.clone()).collect();
    db.find_by_moves(&uci_moves).map(|opening| DetectedOpening {
        id: opening.id.clone(),
        name: opening.name.clone(),
        eco: opening.eco.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_result_debug() {
        assert_eq!(format!("{:?}", MatchResult::WhiteWins), "WhiteWins");
        assert_eq!(format!("{:?}", MatchResult::BlackWins), "BlackWins");
        assert_eq!(format!("{:?}", MatchResult::Draw), "Draw");
    }

    #[test]
    fn test_match_result_equality() {
        assert_eq!(MatchResult::WhiteWins, MatchResult::WhiteWins);
        assert_eq!(MatchResult::BlackWins, MatchResult::BlackWins);
        assert_eq!(MatchResult::Draw, MatchResult::Draw);
        assert_ne!(MatchResult::WhiteWins, MatchResult::BlackWins);
        assert_ne!(MatchResult::WhiteWins, MatchResult::Draw);
    }

    #[test]
    fn test_game_result_clone() {
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
            result: MatchResult::Draw,
            white_name: "Engine A".to_string(),
            black_name: "Engine B".to_string(),
            opening: None,
        };
        let cloned = result.clone();
        assert_eq!(cloned.moves.len(), result.moves.len());
        assert_eq!(cloned.moves[0].uci, result.moves[0].uci);
        assert_eq!(cloned.result, result.result);
        assert_eq!(cloned.white_name, result.white_name);
        assert_eq!(cloned.black_name, result.black_name);
    }

    #[test]
    fn test_move_record_with_search_info() {
        let record = MoveRecord {
            uci: "e2e4".to_string(),
            search_info: Some(SearchInfo {
                depth: Some(20),
                score_cp: Some(35),
                score_mate: None,
                nodes: Some(1234567),
                time_ms: Some(1500),
                pv: vec!["e2e4".to_string(), "e7e5".to_string()],
            }),
        };

        assert_eq!(record.uci, "e2e4");
        assert!(record.search_info.is_some());
        let info = record.search_info.unwrap();
        assert_eq!(info.depth, Some(20));
        assert_eq!(info.score_cp, Some(35));
    }

    #[test]
    fn test_move_record_serialize() {
        let record = MoveRecord {
            uci: "g1f3".to_string(),
            search_info: Some(SearchInfo {
                depth: Some(10),
                score_cp: Some(-15),
                score_mate: None,
                nodes: None,
                time_ms: None,
                pv: vec![],
            }),
        };

        let json = serde_json::to_string(&record).expect("Failed to serialize");
        assert!(json.contains("\"uci\":\"g1f3\""));
        assert!(json.contains("\"depth\":10"));
        assert!(json.contains("\"score_cp\":-15"));
    }

    // Additional tests for MoveRecord

    #[test]
    fn test_move_record_creation_without_search_info() {
        let record = MoveRecord {
            uci: "e2e4".to_string(),
            search_info: None,
        };
        assert_eq!(record.uci, "e2e4");
        assert!(record.search_info.is_none());
    }

    #[test]
    fn test_move_record_clone() {
        let record = MoveRecord {
            uci: "d2d4".to_string(),
            search_info: Some(SearchInfo {
                depth: Some(15),
                score_cp: Some(50),
                score_mate: None,
                nodes: Some(100000),
                time_ms: Some(200),
                pv: vec!["d2d4".to_string(), "d7d5".to_string()],
            }),
        };
        let cloned = record.clone();
        assert_eq!(cloned.uci, record.uci);
        assert!(cloned.search_info.is_some());
        assert_eq!(cloned.search_info.as_ref().unwrap().depth, Some(15));
    }

    #[test]
    fn test_move_record_serialize_without_search_info() {
        let record = MoveRecord {
            uci: "a2a4".to_string(),
            search_info: None,
        };
        let json = serde_json::to_string(&record).expect("Failed to serialize");
        assert!(json.contains("\"uci\":\"a2a4\""));
        assert!(json.contains("\"search_info\":null"));
    }

    // Additional tests for GameResult

    #[test]
    fn test_game_result_with_white_wins() {
        let result = GameResult {
            moves: vec![MoveRecord {
                uci: "e2e4".to_string(),
                search_info: None,
            }],
            result: MatchResult::WhiteWins,
            white_name: "Stockfish".to_string(),
            black_name: "Komodo".to_string(),
            opening: None,
        };
        assert_eq!(result.result, MatchResult::WhiteWins);
        assert_eq!(result.white_name, "Stockfish");
        assert_eq!(result.black_name, "Komodo");
    }

    #[test]
    fn test_game_result_with_black_wins() {
        let result = GameResult {
            moves: vec![],
            result: MatchResult::BlackWins,
            white_name: "Engine1".to_string(),
            black_name: "Engine2".to_string(),
            opening: None,
        };
        assert_eq!(result.result, MatchResult::BlackWins);
    }

    #[test]
    fn test_game_result_empty_moves() {
        let result = GameResult {
            moves: vec![],
            result: MatchResult::Draw,
            white_name: "A".to_string(),
            black_name: "B".to_string(),
            opening: None,
        };
        assert!(result.moves.is_empty());
        assert_eq!(result.result, MatchResult::Draw);
    }

    #[test]
    fn test_game_result_debug_format() {
        let result = GameResult {
            moves: vec![MoveRecord {
                uci: "e2e4".to_string(),
                search_info: None,
            }],
            result: MatchResult::Draw,
            white_name: "W".to_string(),
            black_name: "B".to_string(),
            opening: None,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("GameResult"));
        assert!(debug.contains("moves"));
        assert!(debug.contains("Draw"));
    }

    // Additional tests for MatchResult

    #[test]
    fn test_match_result_copy() {
        let result = MatchResult::WhiteWins;
        let copied = result;
        assert_eq!(result, copied);
    }

    #[test]
    fn test_match_result_all_variants() {
        let variants = [
            MatchResult::WhiteWins,
            MatchResult::BlackWins,
            MatchResult::Draw,
        ];

        // Each variant should be equal to itself
        for v in &variants {
            assert_eq!(*v, *v);
        }

        // All variants should be different from each other
        assert_ne!(variants[0], variants[1]);
        assert_ne!(variants[1], variants[2]);
        assert_ne!(variants[0], variants[2]);
    }

    #[test]
    fn test_game_error_uci_variant() {
        use crate::uci_client::UciError;
        let uci_err = UciError::SpawnError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "engine not found",
        ));
        let game_err = GameError::Uci(uci_err);
        assert!(game_err.to_string().contains("UCI error"));
    }

    #[test]
    fn test_game_error_invalid_move_variant() {
        let err = GameError::InvalidMove("x9x9".to_string());
        assert_eq!(err.to_string(), "Invalid move: x9x9");
    }

    #[test]
    fn test_game_error_from_uci_error() {
        use crate::uci_client::UciError;
        let uci_err = UciError::NotReady;
        let game_err: GameError = uci_err.into();
        match game_err {
            GameError::Uci(_) => {}
            _ => panic!("Expected Uci variant"),
        }
    }

    #[test]
    fn test_game_error_debug_format() {
        let err = GameError::InvalidMove("invalid".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("InvalidMove"));
        assert!(debug.contains("invalid"));
    }

    // ===== Opening Detection Tests =====

    #[test]
    fn test_detected_opening_default() {
        let opening = DetectedOpening::default();
        assert!(opening.id.is_empty());
        assert!(opening.name.is_empty());
        assert!(opening.eco.is_none());
    }

    #[test]
    fn test_detected_opening_clone() {
        let opening = DetectedOpening {
            id: "italian-game".to_string(),
            name: "Italian Game".to_string(),
            eco: Some("C50".to_string()),
        };
        let cloned = opening.clone();
        assert_eq!(cloned.id, "italian-game");
        assert_eq!(cloned.name, "Italian Game");
        assert_eq!(cloned.eco, Some("C50".to_string()));
    }

    #[test]
    fn test_detected_opening_serialize() {
        let opening = DetectedOpening {
            id: "sicilian-defense".to_string(),
            name: "Sicilian Defense".to_string(),
            eco: Some("B20".to_string()),
        };
        let json = serde_json::to_string(&opening).expect("Failed to serialize");
        assert!(json.contains("\"id\":\"sicilian-defense\""));
        assert!(json.contains("\"name\":\"Sicilian Defense\""));
        assert!(json.contains("\"eco\":\"B20\""));
    }

    #[test]
    fn test_detected_opening_serialize_without_eco() {
        let opening = DetectedOpening {
            id: "custom-opening".to_string(),
            name: "Custom Opening".to_string(),
            eco: None,
        };
        let json = serde_json::to_string(&opening).expect("Failed to serialize");
        assert!(json.contains("\"eco\":null"));
    }

    #[test]
    fn test_detect_opening_finds_italian_game() {
        use chess_openings::{builtin::builtin_openings, OpeningDatabase};

        let db = OpeningDatabase::with_openings(builtin_openings());
        let moves = vec![
            MoveRecord { uci: "e2e4".to_string(), search_info: None },
            MoveRecord { uci: "e7e5".to_string(), search_info: None },
            MoveRecord { uci: "g1f3".to_string(), search_info: None },
            MoveRecord { uci: "b8c6".to_string(), search_info: None },
            MoveRecord { uci: "f1c4".to_string(), search_info: None },
        ];

        let detected = detect_opening(&moves, &db);
        assert!(detected.is_some());
        let opening = detected.unwrap();
        assert_eq!(opening.id, "italian-game");
        assert_eq!(opening.name, "Italian Game");
        assert_eq!(opening.eco, Some("C50".to_string()));
    }

    #[test]
    fn test_detect_opening_finds_sicilian() {
        use chess_openings::{builtin::builtin_openings, OpeningDatabase};

        let db = OpeningDatabase::with_openings(builtin_openings());
        let moves = vec![
            MoveRecord { uci: "e2e4".to_string(), search_info: None },
            MoveRecord { uci: "c7c5".to_string(), search_info: None },
        ];

        let detected = detect_opening(&moves, &db);
        assert!(detected.is_some());
        let opening = detected.unwrap();
        assert_eq!(opening.id, "sicilian-defense");
    }

    #[test]
    fn test_detect_opening_returns_none_for_unknown() {
        use chess_openings::{builtin::builtin_openings, OpeningDatabase};

        let db = OpeningDatabase::with_openings(builtin_openings());
        // Start with an unusual move that's not in the database
        let moves = vec![
            MoveRecord { uci: "a2a3".to_string(), search_info: None },
            MoveRecord { uci: "a7a6".to_string(), search_info: None },
        ];

        let detected = detect_opening(&moves, &db);
        assert!(detected.is_none());
    }

    #[test]
    fn test_detect_opening_empty_moves() {
        use chess_openings::{builtin::builtin_openings, OpeningDatabase};

        let db = OpeningDatabase::with_openings(builtin_openings());
        let moves: Vec<MoveRecord> = vec![];

        let detected = detect_opening(&moves, &db);
        assert!(detected.is_none());
    }

    #[test]
    fn test_game_result_with_detected_opening() {
        let result = GameResult {
            moves: vec![MoveRecord { uci: "e2e4".to_string(), search_info: None }],
            result: MatchResult::Draw,
            white_name: "White".to_string(),
            black_name: "Black".to_string(),
            opening: Some(DetectedOpening {
                id: "french-defense".to_string(),
                name: "French Defense".to_string(),
                eco: Some("C00".to_string()),
            }),
        };

        assert!(result.opening.is_some());
        let opening = result.opening.as_ref().unwrap();
        assert_eq!(opening.id, "french-defense");
    }
}
