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
/// let mut runner = GameRunner::new(white, black, "movetime 500".to_string())?;
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
}

impl GameRunner {
    /// Creates a new game runner with the given engines and time control.
    ///
    /// Initializes both UCI engines and prepares them for play.
    ///
    /// # Arguments
    ///
    /// * `white` - The UCI client for the white player
    /// * `black` - The UCI client for the black player
    /// * `time_control` - The time control string (e.g., "movetime 500")
    ///
    /// # Errors
    ///
    /// Returns an error if either engine fails to initialize.
    pub fn new(
        mut white: UciClient,
        mut black: UciClient,
        time_control: String,
    ) -> Result<Self, GameError> {
        white.init()?;
        black.init()?;
        Ok(Self {
            white,
            black,
            time_control,
        })
    }

    /// Plays a complete game between the two engines.
    ///
    /// Executes the game loop, alternating moves between white and black
    /// until the game ends (checkmate, stalemate, draw, or error).
    ///
    /// # Returns
    ///
    /// Returns a [`GameResult`] containing the move history and outcome.
    ///
    /// # Errors
    ///
    /// Returns an error if an engine produces an invalid move or if
    /// UCI communication fails.
    pub fn play_game(&mut self) -> Result<GameResult, GameError> {
        let mut game = Game::new();
        let mut moves: Vec<MoveRecord> = Vec::new();
        let white_name = self.white.name.clone();
        let black_name = self.black.name.clone();

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
        })
    }
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
}
