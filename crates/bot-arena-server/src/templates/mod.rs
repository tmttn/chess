//! HTML templates for export functionality.
//!
//! This module contains Askama templates for generating HTML exports of chess games
//! and positions.

pub mod board;
pub mod match_export;

pub use board::{BoardTemplate, PieceView};
pub use match_export::{GameSummary, MatchExportTemplate};
