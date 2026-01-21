//! HTML templates for export functionality.
//!
//! This module contains Askama templates for generating HTML exports of chess games,
//! positions, and bot profiles.

pub mod board;
pub mod bot_export;
pub mod game_export;
pub mod match_export;

pub use board::{BoardTemplate, PieceView};
pub use bot_export::{BotExportTemplate, EloPoint};
pub use game_export::GameExportTemplate;
pub use match_export::{GameSummary, MatchExportTemplate};
