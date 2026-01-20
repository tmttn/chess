//! Rule set abstraction for chess variants.
//!
//! This module provides the [`RuleSet`] trait which abstracts over different
//! chess variants. The engine is rule-agnostic - it delegates game-specific
//! logic to the active rule set.

mod standard;

pub use standard::StandardChess;

use crate::{MoveList, Position};
use chess_core::Move;

/// Result of a finished game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    /// White wins (checkmate or resignation).
    WhiteWins,
    /// Black wins (checkmate or resignation).
    BlackWins,
    /// Draw with a specific reason.
    Draw(DrawReason),
}

/// Reason for a draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawReason {
    /// Stalemate - no legal moves but not in check.
    Stalemate,
    /// Insufficient material to checkmate.
    InsufficientMaterial,
    /// 50-move rule (100 half-moves without pawn move or capture) - claimable.
    FiftyMoveRule,
    /// 75-move rule (150 half-moves) - automatic draw.
    SeventyFiveMoveRule,
    /// Threefold repetition - claimable.
    ThreefoldRepetition,
    /// Fivefold repetition - automatic draw.
    FivefoldRepetition,
    /// Draw by agreement.
    Agreement,
}

/// Trait for implementing chess variants.
///
/// The engine uses this trait to delegate all game-specific logic, making it
/// easy to support different chess variants (standard, Chess960, etc.) without
/// changing the core engine code.
///
/// # Example
///
/// ```
/// use chess_engine::{Position, StandardChess};
/// use chess_engine::rules::RuleSet;
///
/// let position = StandardChess.initial_position();
/// let moves = StandardChess.generate_moves(&position);
/// ```
pub trait RuleSet {
    /// Returns the initial position for this variant.
    fn initial_position(&self) -> Position;

    /// Generates all legal moves for the given position.
    fn generate_moves(&self, position: &Position) -> MoveList;

    /// Returns true if the given move is legal in the position.
    fn is_legal(&self, position: &Position, m: Move) -> bool;

    /// Makes a move on the position, returning the new position.
    ///
    /// # Panics
    ///
    /// May panic if the move is not legal. Use [`is_legal`](RuleSet::is_legal)
    /// to check first, or use [`try_make_move`](RuleSet::try_make_move).
    fn make_move(&self, position: &Position, m: Move) -> Position;

    /// Attempts to make a move, returning `None` if illegal.
    fn try_make_move(&self, position: &Position, m: Move) -> Option<Position> {
        if self.is_legal(position, m) {
            Some(self.make_move(position, m))
        } else {
            None
        }
    }

    /// Returns true if the side to move is in check.
    fn is_check(&self, position: &Position) -> bool;

    /// Returns the game result if the game is over, otherwise `None`.
    ///
    /// Note: This only checks conditions detectable from a single position
    /// (checkmate, stalemate, 75-move rule, insufficient material).
    /// For repetition draws, use the [`Game`](crate::Game) struct which tracks history.
    fn game_result(&self, position: &Position) -> Option<GameResult>;

    /// Returns true if the game is over.
    fn is_game_over(&self, position: &Position) -> bool {
        self.game_result(position).is_some()
    }

    /// Returns true if neither side has sufficient material to checkmate.
    fn is_insufficient_material(&self, position: &Position) -> bool;
}
