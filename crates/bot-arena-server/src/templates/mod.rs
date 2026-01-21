//! HTML templates for export functionality.
//!
//! This module contains Askama templates for generating HTML exports of chess games
//! and positions.

pub mod board;

pub use board::{BoardTemplate, PieceView};
