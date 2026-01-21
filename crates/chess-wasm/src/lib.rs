//! WebAssembly bindings for the chess engine.
//!
//! This crate provides a JavaScript-friendly API for the chess engine,
//! allowing it to run in web browsers and Node.js.
//!
//! # Usage
//!
//! ```javascript
//! import init, { Game } from 'chess-wasm';
//!
//! await init();
//!
//! const game = new Game();
//! console.log(game.to_fen());
//!
//! const moves = game.legal_moves();
//! console.log(`Legal moves: ${moves.length}`);
//!
//! game.make_move("e2e4");
//! console.log(game.to_fen());
//! ```

use chess_engine::rules::RuleSet;
use chess_engine::{Position, StandardChess};
use wasm_bindgen::prelude::*;

/// A chess game that can be manipulated from JavaScript.
#[wasm_bindgen]
pub struct Game {
    position: Position,
    rules: StandardChess,
}

#[wasm_bindgen]
impl Game {
    /// Creates a new game with the standard starting position.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Game {
            position: StandardChess.initial_position(),
            rules: StandardChess,
        }
    }

    /// Creates a game from a FEN string.
    ///
    /// Returns an error if the FEN is invalid.
    #[wasm_bindgen(js_name = fromFen)]
    pub fn from_fen(fen: &str) -> Result<Game, JsError> {
        let position = Position::from_fen(fen).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Game {
            position,
            rules: StandardChess,
        })
    }

    /// Returns the current position as a FEN string.
    #[wasm_bindgen(js_name = toFen)]
    pub fn to_fen(&self) -> String {
        self.position.to_fen()
    }

    /// Returns an array of legal moves in UCI format.
    #[wasm_bindgen(js_name = legalMoves)]
    pub fn legal_moves(&self) -> Vec<String> {
        self.rules
            .generate_moves(&self.position)
            .as_slice()
            .iter()
            .map(|m| m.to_uci())
            .collect()
    }

    /// Makes a move given in UCI format (e.g., "e2e4", "e7e8q").
    ///
    /// Returns an error if the move is invalid or illegal.
    #[wasm_bindgen(js_name = makeMove)]
    pub fn make_move(&mut self, uci: &str) -> Result<(), JsError> {
        let m = chess_core::Move::from_uci(uci)
            .ok_or_else(|| JsError::new(&format!("Invalid move format: {}", uci)))?;

        // Find the matching legal move with correct flags (DoublePush, EnPassant, etc.)
        // since from_uci doesn't set these flags properly.
        let legal_move = self
            .rules
            .generate_moves(&self.position)
            .as_slice()
            .iter()
            .find(|legal| {
                legal.from() == m.from()
                    && legal.to() == m.to()
                    && legal.flag().promotion_piece() == m.flag().promotion_piece()
            })
            .copied()
            .ok_or_else(|| JsError::new(&format!("Illegal move: {}", uci)))?;

        self.position = self.rules.make_move(&self.position, legal_move);
        Ok(())
    }

    /// Returns true if the current side to move is in check.
    #[wasm_bindgen(js_name = isCheck)]
    pub fn is_check(&self) -> bool {
        self.rules.is_check(&self.position)
    }

    /// Returns true if the game is over (checkmate, stalemate, or draw).
    #[wasm_bindgen(js_name = isGameOver)]
    pub fn is_game_over(&self) -> bool {
        self.rules.is_game_over(&self.position)
    }

    /// Returns the game result if the game is over.
    ///
    /// Returns one of: "white_wins", "black_wins", "draw", or null if game is ongoing.
    #[wasm_bindgen]
    pub fn result(&self) -> Option<String> {
        self.rules.game_result(&self.position).map(|r| match r {
            chess_engine::GameResult::WhiteWins => "white_wins".to_string(),
            chess_engine::GameResult::BlackWins => "black_wins".to_string(),
            chess_engine::GameResult::Draw(_) => "draw".to_string(),
        })
    }

    /// Returns the side to move ("white" or "black").
    #[wasm_bindgen(js_name = sideToMove)]
    pub fn side_to_move(&self) -> String {
        match self.position.side_to_move {
            chess_core::Color::White => "white".to_string(),
            chess_core::Color::Black => "black".to_string(),
        }
    }

    /// Returns the piece at the given square in algebraic notation.
    ///
    /// Returns null if the square is empty.
    /// Returns a string like "P" (white pawn), "k" (black king), etc.
    #[wasm_bindgen(js_name = pieceAt)]
    pub fn piece_at(&self, square: &str) -> Option<String> {
        let sq = chess_core::Square::from_algebraic(square)?;
        let (piece, color) = self.position.piece_at(sq)?;
        Some(piece.to_fen_char(color).to_string())
    }

    /// Resets the game to the starting position.
    pub fn reset(&mut self) {
        self.position = StandardChess.initial_position();
    }

    /// Converts a UCI move to Standard Algebraic Notation (SAN).
    ///
    /// Must be called before making the move since it needs the current position.
    #[wasm_bindgen(js_name = moveToSan)]
    pub fn move_to_san(&self, uci: &str) -> Result<String, JsError> {
        use chess_core::{Move, MoveFlag, Piece};

        let m = Move::from_uci(uci)
            .ok_or_else(|| JsError::new(&format!("Invalid move format: {}", uci)))?;

        // Find the legal move with correct flags
        let legal_moves = self.rules.generate_moves(&self.position);
        let legal_move = legal_moves
            .as_slice()
            .iter()
            .find(|legal| {
                legal.from() == m.from()
                    && legal.to() == m.to()
                    && legal.flag().promotion_piece() == m.flag().promotion_piece()
            })
            .ok_or_else(|| JsError::new(&format!("Illegal move: {}", uci)))?;

        let from = legal_move.from();
        let to = legal_move.to();
        let flag = legal_move.flag();

        // Get piece at from square
        let (piece, _color) = self
            .position
            .piece_at(from)
            .ok_or_else(|| JsError::new("No piece at from square"))?;

        let mut san = String::new();

        // Handle castling
        if flag == MoveFlag::CastleKingside {
            san.push_str("O-O");
        } else if flag == MoveFlag::CastleQueenside {
            san.push_str("O-O-O");
        } else {
            // Piece letter (except pawns)
            if piece != Piece::Pawn {
                san.push(match piece {
                    Piece::Knight => 'N',
                    Piece::Bishop => 'B',
                    Piece::Rook => 'R',
                    Piece::Queen => 'Q',
                    Piece::King => 'K',
                    Piece::Pawn => unreachable!(),
                });

                // Check for disambiguation - other pieces of same type that can reach the target
                let same_piece_moves: Vec<_> = legal_moves
                    .as_slice()
                    .iter()
                    .filter(|mv| {
                        mv.to() == to
                            && mv.from() != from
                            && self
                                .position
                                .piece_at(mv.from())
                                .map(|(p, _)| p == piece)
                                .unwrap_or(false)
                    })
                    .collect();

                if !same_piece_moves.is_empty() {
                    let same_file = same_piece_moves
                        .iter()
                        .any(|mv| mv.from().file() == from.file());
                    let same_rank = same_piece_moves
                        .iter()
                        .any(|mv| mv.from().rank() == from.rank());

                    if !same_file {
                        san.push(from.to_algebraic().chars().next().unwrap());
                    } else if !same_rank {
                        san.push(from.to_algebraic().chars().nth(1).unwrap());
                    } else {
                        san.push_str(&from.to_algebraic());
                    }
                }
            }

            // Capture indicator
            let is_capture = self.position.piece_at(to).is_some() || flag == MoveFlag::EnPassant;
            if is_capture {
                if piece == Piece::Pawn {
                    san.push(from.to_algebraic().chars().next().unwrap());
                }
                san.push('x');
            }

            // Destination square
            san.push_str(&to.to_algebraic());

            // Promotion
            if let Some(promo_piece) = flag.promotion_piece() {
                san.push('=');
                san.push(match promo_piece {
                    Piece::Queen => 'Q',
                    Piece::Rook => 'R',
                    Piece::Bishop => 'B',
                    Piece::Knight => 'N',
                    _ => 'Q',
                });
            }
        }

        // Check for check or checkmate after the move
        let new_pos = self.rules.make_move(&self.position, *legal_move);
        if self.rules.is_check(&new_pos) {
            if self.rules.is_game_over(&new_pos) {
                san.push('#');
            } else {
                san.push('+');
            }
        }

        Ok(san)
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialization function called when WASM module loads.
#[wasm_bindgen(start)]
pub fn init() {
    // Future: Add console_error_panic_hook for better panic messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_new() {
        let game = Game::new();
        assert_eq!(game.side_to_move(), "white");
        // Note: is_game_over() will return true until move generation is implemented
        // because an empty move list is interpreted as stalemate
    }

    #[test]
    fn game_from_fen() {
        let game =
            Game::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
        assert_eq!(game.side_to_move(), "black");
    }

    #[test]
    fn piece_at() {
        let game = Game::new();
        assert_eq!(game.piece_at("e1"), Some("K".to_string()));
        assert_eq!(game.piece_at("e8"), Some("k".to_string()));
        assert_eq!(game.piece_at("e4"), None);
    }
}
