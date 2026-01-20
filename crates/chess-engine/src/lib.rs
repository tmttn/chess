//! High-performance chess engine with bitboard representation.
//!
//! This crate provides:
//! - [`Bitboard`] - 64-bit board representation with efficient operations
//! - [`Position`] - Full game state including piece positions, castling rights, etc.
//! - [`RuleSet`] - Trait for implementing chess variants
//! - Move generation and validation
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
//! use chess_engine::{Position, StandardChess};
//! use chess_engine::rules::RuleSet;
//!
//! let mut position = StandardChess.initial_position();
//! let moves = StandardChess.generate_moves(&position);
//! println!("Legal moves from starting position: {}", moves.len());
//! ```

mod bitboard;
mod movegen;
mod position;
pub mod rules;
mod zobrist;

pub use bitboard::Bitboard;
pub use movegen::MoveList;
pub use position::Position;
pub use rules::{GameResult, RuleSet, StandardChess};
