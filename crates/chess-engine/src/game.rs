//! Full game management with history tracking.
//!
//! The [`Game`] struct provides complete game state management including:
//! - Position history for repetition detection
//! - Move history with SAN notation
//! - All FIDE draw conditions
//! - Draw claiming

use crate::movegen::{generate_moves, is_king_attacked, make_move};
use crate::rules::{DrawReason, GameResult, RuleSet, StandardChess};
use crate::san::{move_to_san, san_to_move, SanError};
use crate::{MoveList, Position};
use chess_core::Move;
use std::fmt;

/// A recorded move in game history.
#[derive(Debug, Clone)]
pub struct GameMove {
    /// The move in internal format.
    pub mov: Move,
    /// SAN notation for the move.
    pub san: String,
    /// Zobrist hash of the position before the move.
    pub hash_before: u64,
}

/// Error type for game operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    /// The move is not legal in the current position.
    IllegalMove(String),
    /// The SAN string could not be parsed.
    InvalidSan(SanError),
    /// The game has already ended.
    GameAlreadyOver,
    /// Cannot claim draw (conditions not met).
    CannotClaimDraw,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::IllegalMove(s) => write!(f, "illegal move: {}", s),
            GameError::InvalidSan(e) => write!(f, "invalid SAN: {}", e),
            GameError::GameAlreadyOver => write!(f, "game has already ended"),
            GameError::CannotClaimDraw => write!(f, "cannot claim draw: conditions not met"),
        }
    }
}

impl std::error::Error for GameError {}

impl From<SanError> for GameError {
    fn from(e: SanError) -> Self {
        GameError::InvalidSan(e)
    }
}

/// A complete chess game with history tracking.
///
/// Unlike [`Position`], which represents a single board state, `Game` tracks
/// the full game history needed for repetition detection and provides
/// methods for all FIDE draw conditions.
#[derive(Debug, Clone)]
pub struct Game {
    /// Current position.
    position: Position,
    /// Position hashes for repetition detection.
    history: Vec<u64>,
    /// Move history with SAN notation.
    moves: Vec<GameMove>,
    /// Starting position.
    start_pos: Position,
    /// Game result if the game has ended.
    result: Option<GameResult>,
    /// Whether a draw has been claimed.
    draw_claimed: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Creates a new game with the standard starting position.
    pub fn new() -> Self {
        let position = StandardChess.initial_position();
        let hash = position.zobrist_hash();
        Game {
            position: position.clone(),
            history: vec![hash],
            moves: Vec::new(),
            start_pos: position,
            result: None,
            draw_claimed: false,
        }
    }

    /// Creates a game from a custom starting position.
    pub fn from_position(position: Position) -> Self {
        let hash = position.zobrist_hash();
        let mut game = Game {
            position: position.clone(),
            history: vec![hash],
            moves: Vec::new(),
            start_pos: position,
            result: None,
            draw_claimed: false,
        };
        // Check if the game is already over
        game.check_game_end();
        game
    }

    /// Creates a game from a FEN string.
    pub fn from_fen(fen: &str) -> Result<Self, chess_core::FenError> {
        let position = Position::from_fen(fen)?;
        Ok(Self::from_position(position))
    }

    /// Returns a reference to the current position.
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Returns the starting position.
    pub fn start_position(&self) -> &Position {
        &self.start_pos
    }

    /// Returns all legal moves in the current position.
    pub fn legal_moves(&self) -> MoveList {
        generate_moves(&self.position)
    }

    /// Returns true if the side to move is in check.
    pub fn is_check(&self) -> bool {
        is_king_attacked(&self.position, self.position.side_to_move)
    }

    /// Returns the game result if the game is over.
    pub fn result(&self) -> Option<GameResult> {
        self.result
    }

    /// Returns true if the game has ended.
    pub fn is_game_over(&self) -> bool {
        self.result.is_some()
    }

    /// Returns the move history.
    pub fn move_history(&self) -> &[GameMove] {
        &self.moves
    }

    /// Returns the number of half-moves (plies) played.
    pub fn ply_count(&self) -> usize {
        self.moves.len()
    }

    /// Returns the current full move number.
    pub fn fullmove_number(&self) -> u32 {
        self.position.fullmove_number
    }

    /// Makes a move given in internal format.
    pub fn make_move(&mut self, m: Move) -> Result<(), GameError> {
        if self.result.is_some() {
            return Err(GameError::GameAlreadyOver);
        }

        // Check if move is legal
        let legal_moves = self.legal_moves();
        if !legal_moves.as_slice().contains(&m) {
            return Err(GameError::IllegalMove(m.to_uci()));
        }

        self.apply_move(m);
        Ok(())
    }

    /// Makes a move given in SAN notation.
    pub fn make_move_san(&mut self, san: &str) -> Result<(), GameError> {
        if self.result.is_some() {
            return Err(GameError::GameAlreadyOver);
        }

        let m = san_to_move(&self.position, san)?;
        self.apply_move(m);
        Ok(())
    }

    /// Makes a move given in UCI notation.
    pub fn make_move_uci(&mut self, uci: &str) -> Result<(), GameError> {
        if self.result.is_some() {
            return Err(GameError::GameAlreadyOver);
        }

        let m = Move::from_uci(uci)
            .ok_or_else(|| GameError::IllegalMove(format!("invalid UCI: {}", uci)))?;

        // Find the matching legal move (to get correct flags)
        let legal_moves = self.legal_moves();
        let matching = legal_moves.as_slice().iter().find(|lm| {
            lm.from() == m.from() && lm.to() == m.to() && {
                // For promotions, also match the promotion piece
                if m.flag().is_promotion() {
                    lm.flag() == m.flag()
                } else {
                    true
                }
            }
        });

        match matching {
            Some(&legal_move) => {
                self.apply_move(legal_move);
                Ok(())
            }
            None => Err(GameError::IllegalMove(uci.to_string())),
        }
    }

    /// Internal method to apply a legal move.
    fn apply_move(&mut self, m: Move) {
        let san = move_to_san(&self.position, m);
        let hash_before = self.position.zobrist_hash();

        // Record the move
        self.moves.push(GameMove {
            mov: m,
            san,
            hash_before,
        });

        // Apply the move
        self.position = make_move(&self.position, m);

        // Record position hash for repetition detection
        let new_hash = self.position.zobrist_hash();
        self.history.push(new_hash);

        // Check for game end
        self.check_game_end();
    }

    /// Checks if the game has ended and updates the result.
    fn check_game_end(&mut self) {
        // Check for fivefold repetition (automatic draw)
        if self.position_count() >= 5 {
            self.result = Some(GameResult::Draw(DrawReason::FivefoldRepetition));
            return;
        }

        // Check for 75-move rule (automatic draw)
        if self.position.halfmove_clock >= 150 {
            self.result = Some(GameResult::Draw(DrawReason::SeventyFiveMoveRule));
            return;
        }

        // Check for insufficient material
        if StandardChess.is_insufficient_material(&self.position) {
            self.result = Some(GameResult::Draw(DrawReason::InsufficientMaterial));
            return;
        }

        // Check for checkmate or stalemate
        let moves = self.legal_moves();
        if moves.is_empty() {
            if self.is_check() {
                // Checkmate
                self.result = Some(match self.position.side_to_move {
                    chess_core::Color::White => GameResult::BlackWins,
                    chess_core::Color::Black => GameResult::WhiteWins,
                });
            } else {
                // Stalemate
                self.result = Some(GameResult::Draw(DrawReason::Stalemate));
            }
        }
    }

    /// Counts how many times the current position has occurred.
    pub fn position_count(&self) -> usize {
        let current_hash = self.position.zobrist_hash();
        self.history.iter().filter(|&&h| h == current_hash).count()
    }

    /// Returns true if a draw can be claimed (threefold repetition or 50-move rule).
    pub fn can_claim_draw(&self) -> bool {
        if self.result.is_some() {
            return false;
        }
        self.position_count() >= 3 || self.position.halfmove_clock >= 100
    }

    /// Claims a draw if conditions are met.
    pub fn claim_draw(&mut self) -> Result<(), GameError> {
        if self.result.is_some() {
            return Err(GameError::GameAlreadyOver);
        }

        if self.position_count() >= 3 {
            self.result = Some(GameResult::Draw(DrawReason::ThreefoldRepetition));
            self.draw_claimed = true;
            return Ok(());
        }

        if self.position.halfmove_clock >= 100 {
            self.result = Some(GameResult::Draw(DrawReason::FiftyMoveRule));
            self.draw_claimed = true;
            return Ok(());
        }

        Err(GameError::CannotClaimDraw)
    }

    /// Agrees to a draw (both players must agree in real chess).
    pub fn agree_draw(&mut self) -> Result<(), GameError> {
        if self.result.is_some() {
            return Err(GameError::GameAlreadyOver);
        }

        self.result = Some(GameResult::Draw(DrawReason::Agreement));
        self.draw_claimed = true;
        Ok(())
    }

    /// Resigns the game for the side to move.
    pub fn resign(&mut self) -> Result<(), GameError> {
        if self.result.is_some() {
            return Err(GameError::GameAlreadyOver);
        }

        self.result = Some(match self.position.side_to_move {
            chess_core::Color::White => GameResult::BlackWins,
            chess_core::Color::Black => GameResult::WhiteWins,
        });
        Ok(())
    }

    /// Returns the current position as a FEN string.
    pub fn to_fen(&self) -> String {
        self.position.to_fen()
    }

    /// Generates SAN for a move in the current position.
    pub fn move_to_san(&self, m: Move) -> String {
        move_to_san(&self.position, m)
    }

    /// Parses SAN and returns the corresponding move.
    pub fn san_to_move(&self, san: &str) -> Result<Move, SanError> {
        san_to_move(&self.position, san)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::{File, MoveFlag, Rank, Square};

    #[test]
    fn new_game() {
        let game = Game::new();
        assert_eq!(game.ply_count(), 0);
        assert!(!game.is_game_over());
        assert!(!game.is_check());
    }

    #[test]
    fn make_move_uci() {
        let mut game = Game::new();
        game.make_move_uci("e2e4").unwrap();
        assert_eq!(game.ply_count(), 1);
        assert_eq!(game.move_history()[0].san, "e4");
    }

    #[test]
    fn make_move_san() {
        let mut game = Game::new();
        game.make_move_san("e4").unwrap();
        assert_eq!(game.ply_count(), 1);
        game.make_move_san("e5").unwrap();
        assert_eq!(game.ply_count(), 2);
    }

    #[test]
    fn illegal_move() {
        let mut game = Game::new();
        let result = game.make_move_uci("e2e5"); // Can't move pawn 3 squares
        assert!(result.is_err());
    }

    #[test]
    fn checkmate_fools_mate() {
        let mut game = Game::new();
        game.make_move_san("f3").unwrap();
        game.make_move_san("e5").unwrap();
        game.make_move_san("g4").unwrap();
        game.make_move_san("Qh4").unwrap();
        assert!(game.is_game_over());
        assert_eq!(game.result(), Some(GameResult::BlackWins));
    }

    #[test]
    fn stalemate() {
        let game = Game::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(game.is_game_over());
        assert_eq!(game.result(), Some(GameResult::Draw(DrawReason::Stalemate)));
    }

    #[test]
    fn threefold_repetition() {
        let mut game = Game::new();
        // Move knights back and forth to create repetition
        game.make_move_san("Nf3").unwrap();
        game.make_move_san("Nf6").unwrap();
        game.make_move_san("Ng1").unwrap();
        game.make_move_san("Ng8").unwrap();
        // Position repeated twice now
        assert_eq!(game.position_count(), 2);
        assert!(!game.can_claim_draw());

        game.make_move_san("Nf3").unwrap();
        game.make_move_san("Nf6").unwrap();
        game.make_move_san("Ng1").unwrap();
        game.make_move_san("Ng8").unwrap();
        // Position repeated three times
        assert_eq!(game.position_count(), 3);
        assert!(game.can_claim_draw());

        // Claim the draw
        game.claim_draw().unwrap();
        assert!(game.is_game_over());
        assert_eq!(
            game.result(),
            Some(GameResult::Draw(DrawReason::ThreefoldRepetition))
        );
    }

    #[test]
    fn fifty_move_rule() {
        // Position with rook so it's not insufficient material
        let mut game = Game::from_fen("8/8/8/8/8/8/8/R3K2k w Q - 99 1").unwrap();
        // Make one more move to reach 100 half-moves
        let a1 = Square::new(File::A, Rank::R1);
        let a2 = Square::new(File::A, Rank::R2);
        let m = Move::normal(a1, a2);
        game.make_move(m).unwrap();
        assert!(game.can_claim_draw());

        game.claim_draw().unwrap();
        assert_eq!(
            game.result(),
            Some(GameResult::Draw(DrawReason::FiftyMoveRule))
        );
    }

    #[test]
    fn seventy_five_move_rule_automatic() {
        // Position with rook so it's not insufficient material, 75-move rule takes precedence
        let game = Game::from_fen("8/8/8/8/8/8/8/R3K2k w Q - 150 1").unwrap();
        assert!(game.is_game_over());
        assert_eq!(
            game.result(),
            Some(GameResult::Draw(DrawReason::SeventyFiveMoveRule))
        );
    }

    #[test]
    fn insufficient_material() {
        let game = Game::from_fen("8/8/8/8/8/8/8/4K2k w - - 0 1").unwrap();
        assert!(game.is_game_over());
        assert_eq!(
            game.result(),
            Some(GameResult::Draw(DrawReason::InsufficientMaterial))
        );
    }

    #[test]
    fn resign() {
        let mut game = Game::new();
        game.resign().unwrap();
        assert!(game.is_game_over());
        assert_eq!(game.result(), Some(GameResult::BlackWins)); // White resigned
    }

    #[test]
    fn agree_draw() {
        let mut game = Game::new();
        game.agree_draw().unwrap();
        assert!(game.is_game_over());
        assert_eq!(game.result(), Some(GameResult::Draw(DrawReason::Agreement)));
    }

    #[test]
    fn cannot_move_after_game_over() {
        let mut game = Game::new();
        game.resign().unwrap();
        let result = game.make_move_san("e4");
        assert!(matches!(result, Err(GameError::GameAlreadyOver)));
    }

    #[test]
    fn move_history() {
        let mut game = Game::new();
        game.make_move_san("e4").unwrap();
        game.make_move_san("e5").unwrap();
        game.make_move_san("Nf3").unwrap();

        let history = game.move_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].san, "e4");
        assert_eq!(history[1].san, "e5");
        assert_eq!(history[2].san, "Nf3");
    }
}
