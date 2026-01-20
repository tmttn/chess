//! Core types for chess.
//!
//! This crate provides the fundamental types used across the chess engine:
//! - [`Piece`] and [`Color`] for piece representation
//! - [`Square`], [`File`], and [`Rank`] for board coordinates
//! - [`Move`] for move representation
//! - FEN parsing and serialization

mod color;
mod fen;
mod mov;
mod piece;
mod square;

pub use color::Color;
pub use fen::{FenError, FenParser};
pub use mov::{Move, MoveFlag};
pub use piece::Piece;
pub use square::{File, Rank, Square};
