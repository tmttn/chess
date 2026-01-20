//! High-performance chess engine with bitboard representation.
//!
//! This crate provides:
//! - [`Bitboard`] - 64-bit board representation with efficient operations
//! - [`Position`] - Full game state including piece positions, castling rights, etc.
//! - [`Game`] - Complete game management with history tracking
//! - [`RuleSet`] - Trait for implementing chess variants
//! - Move generation and validation
//! - SAN notation parsing and generation
//!
//! # Architecture
//!
//! The engine uses bitboards for piece representation - each piece type/color
//! combination has a 64-bit integer where each bit represents a square.
//! This enables efficient move generation using bitwise operations.
//!
//! # Example
//!
//! ```
//! use chess_engine::{Game, Position, StandardChess};
//! use chess_engine::rules::RuleSet;
//!
//! // Using Position directly (stateless)
//! let position = StandardChess.initial_position();
//! let moves = StandardChess.generate_moves(&position);
//! println!("Legal moves from starting position: {}", moves.len());
//!
//! // Using Game for full game management
//! let mut game = Game::new();
//! game.make_move_san("e4").unwrap();
//! game.make_move_san("e5").unwrap();
//! println!("Position after 1.e4 e5: {}", game.to_fen());
//! ```

mod bitboard;
mod game;
pub mod movegen;
mod position;
pub mod rules;
pub mod san;
mod zobrist;

pub use bitboard::Bitboard;
pub use game::{Game, GameError, GameMove};
pub use movegen::{
    bishop_attacks, generate_moves, is_king_attacked, king_attacks, knight_attacks, make_move,
    pawn_attacks, queen_attacks, rook_attacks, MoveList,
};
pub use position::Position;
pub use rules::{DrawReason, GameResult, RuleSet, StandardChess};
pub use san::{move_to_san, san_to_move, SanError};
