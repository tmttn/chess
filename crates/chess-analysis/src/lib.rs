//! Chess position analysis with Stockfish integration.
//!
//! This crate provides move quality classification and game analysis
//! by comparing bot moves against Stockfish evaluations.
//!
//! # Overview
//!
//! - [`Evaluation`] - Position evaluation (centipawn or mate score)
//! - [`MoveQuality`] - Classification of move quality (Best, Excellent, Good, etc.)
//! - [`AnalysisEngine`] - Wrapper for UCI analysis engines like Stockfish
//! - [`GameAnalyzer`] - Analyzes complete games with move quality classification
//!
//! # Example
//!
//! ```ignore
//! use chess_analysis::{GameAnalyzer, AnalysisConfig, MoveInput};
//!
//! let config = AnalysisConfig::default();
//! let mut analyzer = GameAnalyzer::new("stockfish", config)?;
//! let analysis = analyzer.analyze_game("game1", "white", "black", &moves, "draw")?;
//! println!("White accuracy: {:.1}%", analysis.white_stats.accuracy_percent);
//! ```

pub mod engine;
pub mod evaluation;
pub mod quality;

pub use engine::AnalysisEngine;
pub use evaluation::Evaluation;
pub use quality::{GameAnalysis, MoveAnalysis, MoveQuality, PlayerStats};
