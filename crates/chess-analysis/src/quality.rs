//! Move quality classification and game analysis.

use crate::Evaluation;

/// Classification of move quality based on evaluation loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveQuality {
    /// The best move in the position
    Best,
    /// Excellent move (minimal centipawn loss)
    Excellent,
    /// Good move (small centipawn loss)
    Good,
    /// Inaccuracy (noticeable centipawn loss)
    Inaccuracy,
    /// Mistake (significant centipawn loss)
    Mistake,
    /// Blunder (major centipawn loss)
    Blunder,
}

/// Analysis result for a single move.
#[derive(Debug, Clone)]
pub struct MoveAnalysis {
    /// The move that was played (in UCI notation)
    pub played_move: String,
    /// The best move according to the engine
    pub best_move: String,
    /// Evaluation before the move
    pub eval_before: Evaluation,
    /// Evaluation after the move
    pub eval_after: Evaluation,
    /// Quality classification
    pub quality: MoveQuality,
    /// Centipawn loss from playing this move
    pub cp_loss: i32,
}

/// Statistics for a player's performance in a game.
#[derive(Debug, Clone, Default)]
pub struct PlayerStats {
    /// Total moves analyzed
    pub total_moves: u32,
    /// Number of best moves
    pub best_moves: u32,
    /// Number of excellent moves
    pub excellent_moves: u32,
    /// Number of good moves
    pub good_moves: u32,
    /// Number of inaccuracies
    pub inaccuracies: u32,
    /// Number of mistakes
    pub mistakes: u32,
    /// Number of blunders
    pub blunders: u32,
    /// Average centipawn loss
    pub avg_cp_loss: f64,
    /// Accuracy percentage (0-100)
    pub accuracy_percent: f64,
}

/// Complete analysis of a chess game.
#[derive(Debug, Clone)]
pub struct GameAnalysis {
    /// Unique identifier for the game
    pub game_id: String,
    /// White player identifier
    pub white_player: String,
    /// Black player identifier
    pub black_player: String,
    /// Analysis of each move
    pub moves: Vec<MoveAnalysis>,
    /// Statistics for white
    pub white_stats: PlayerStats,
    /// Statistics for black
    pub black_stats: PlayerStats,
    /// Game result
    pub result: String,
}
