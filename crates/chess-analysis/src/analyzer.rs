//! Game analysis with move quality classification.
//!
//! This module provides the [`GameAnalyzer`] for analyzing complete chess games
//! and classifying each move's quality.

use thiserror::Error;

use crate::engine::{AnalysisEngine, EngineError};
use crate::evaluation::Evaluation;
use crate::quality::{GameAnalysis, MoveAnalysis, MoveQuality, PlayerStats};

/// Errors that can occur during game analysis.
#[derive(Error, Debug)]
pub enum AnalyzerError {
    /// Error from the analysis engine.
    #[error("Engine error: {0}")]
    Engine(#[from] EngineError),
    /// Invalid game data was provided.
    #[error("Invalid game data: {0}")]
    InvalidGame(String),
}

/// Input data for a single move to be analyzed.
#[derive(Debug, Clone)]
pub struct MoveInput {
    /// The move in UCI notation (e.g., "e2e4").
    pub uci: String,
    /// Bot's evaluation in centipawns (positive = white advantage).
    pub bot_eval_cp: Option<i32>,
    /// Bot's evaluation as mate-in-N moves.
    pub bot_eval_mate: Option<i32>,
    /// Search depth used by the bot.
    pub bot_depth: Option<u32>,
    /// Number of nodes searched by the bot.
    pub bot_nodes: Option<u64>,
    /// Time spent by the bot in milliseconds.
    pub bot_time_ms: Option<u64>,
    /// Principal variation from the bot's search.
    pub bot_pv: Vec<String>,
}

/// Configuration for game analysis.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Maximum search depth for position analysis.
    pub depth: u32,
    /// Number of opening book moves to mark as forced.
    pub opening_book_moves: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            depth: 15,
            opening_book_moves: 0,
        }
    }
}

/// Analyzes chess games to classify move quality.
///
/// Uses a UCI-compatible engine (like Stockfish) to evaluate positions
/// and compare bot moves against optimal play.
pub struct GameAnalyzer {
    /// The analysis engine instance.
    engine: AnalysisEngine,
    /// Configuration for analysis.
    config: AnalysisConfig,
}

impl GameAnalyzer {
    /// Creates a new game analyzer with the specified engine and configuration.
    ///
    /// # Arguments
    ///
    /// * `stockfish_path` - Path to the UCI engine executable.
    /// * `config` - Analysis configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the engine cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chess_analysis::{GameAnalyzer, AnalysisConfig};
    ///
    /// let config = AnalysisConfig::default();
    /// let analyzer = GameAnalyzer::new("stockfish", config)?;
    /// ```
    pub fn new(stockfish_path: &str, config: AnalysisConfig) -> Result<Self, AnalyzerError> {
        let engine = AnalysisEngine::new(stockfish_path)?;
        Ok(Self { engine, config })
    }

    /// Analyzes a complete chess game.
    ///
    /// For each move in the game:
    /// 1. Analyzes the position before the move at the configured depth.
    /// 2. Calculates centipawn loss by comparing the best move's evaluation
    ///    to the actual move's evaluation.
    /// 3. Classifies move quality based on centipawn loss.
    ///
    /// Opening book moves (if configured) are marked as [`MoveQuality::Forced`].
    ///
    /// # Arguments
    ///
    /// * `game_id` - Unique identifier for the game.
    /// * `white_name` - Name of the white player/bot.
    /// * `black_name` - Name of the black player/bot.
    /// * `moves` - List of moves with optional bot metadata.
    /// * `result` - Game result (e.g., "1-0", "0-1", "1/2-1/2").
    ///
    /// # Errors
    ///
    /// Returns an error if engine analysis fails or game data is invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chess_analysis::{GameAnalyzer, AnalysisConfig, MoveInput};
    ///
    /// let moves = vec![
    ///     MoveInput {
    ///         uci: "e2e4".to_string(),
    ///         bot_eval_cp: Some(20),
    ///         bot_eval_mate: None,
    ///         bot_depth: Some(10),
    ///         bot_nodes: Some(50000),
    ///         bot_time_ms: Some(100),
    ///         bot_pv: vec!["e2e4".to_string(), "e7e5".to_string()],
    ///     },
    /// ];
    ///
    /// let config = AnalysisConfig::default();
    /// let mut analyzer = GameAnalyzer::new("stockfish", config)?;
    /// let analysis = analyzer.analyze_game("game1", "bot1", "bot2", &moves, "1-0")?;
    /// ```
    pub fn analyze_game(
        &mut self,
        game_id: &str,
        white_name: &str,
        black_name: &str,
        moves: &[MoveInput],
        result: &str,
    ) -> Result<GameAnalysis, AnalyzerError> {
        // Clear engine hash tables for fresh analysis
        self.engine.clear_hash()?;

        let mut analyzed_moves: Vec<MoveAnalysis> = Vec::with_capacity(moves.len());
        let mut move_history: Vec<String> = Vec::new();

        for (move_idx, move_input) in moves.iter().enumerate() {
            let is_opening_book = move_idx < self.config.opening_book_moves;

            // Analyze position before the move
            let analysis_before = self
                .engine
                .analyze_moves(&move_history, self.config.depth)?;

            // Add the move to history for next iteration
            move_history.push(move_input.uci.clone());

            // Analyze position after the move
            let analysis_after = self
                .engine
                .analyze_moves(&move_history, self.config.depth)?;

            // Determine if this is white's move (even index = white, odd = black)
            let is_white_move = move_idx % 2 == 0;

            // Calculate centipawn loss
            // The evaluation before is from the side to move's perspective
            // If best move was played, eval_after (negated for opponent) should equal eval_before
            let best_eval_cp = analysis_before.evaluation.to_centipawns();
            let actual_eval_cp = analysis_after.evaluation.flip().to_centipawns();

            // CP loss: how much worse was the actual move compared to best
            // For white: higher is better, so loss = best - actual
            // For black: lower is better, so loss = actual - best
            let cp_loss = if is_white_move {
                (best_eval_cp - actual_eval_cp).max(0)
            } else {
                (actual_eval_cp - best_eval_cp).max(0)
            };

            // Classify move quality
            let quality = MoveQuality::from_cp_loss(cp_loss, is_opening_book);

            // Build bot evaluation from input
            let bot_eval =
                Evaluation::from_uci_score(move_input.bot_eval_cp, move_input.bot_eval_mate);

            // Create MoveAnalysis
            let move_analysis = MoveAnalysis {
                uci: move_input.uci.clone(),
                san: None, // SAN conversion not implemented here
                quality,
                bot_eval,
                bot_depth: move_input.bot_depth,
                bot_nodes: move_input.bot_nodes,
                bot_time_ms: move_input.bot_time_ms,
                bot_pv: move_input.bot_pv.clone(),
                engine_eval_before: Some(analysis_before.evaluation),
                engine_eval_after: Some(analysis_after.evaluation),
                engine_best_move: Some(analysis_before.best_move.clone()),
                engine_pv: analysis_before.pv.clone(),
                centipawn_loss: Some(cp_loss),
            };

            analyzed_moves.push(move_analysis);
        }

        // Separate moves for white and black
        let white_moves: Vec<&MoveAnalysis> = analyzed_moves.iter().step_by(2).collect();
        let black_moves: Vec<&MoveAnalysis> = analyzed_moves.iter().skip(1).step_by(2).collect();

        // Convert references to owned values for PlayerStats::from_moves
        let white_moves_owned: Vec<MoveAnalysis> = white_moves.into_iter().cloned().collect();
        let black_moves_owned: Vec<MoveAnalysis> = black_moves.into_iter().cloned().collect();

        let white_stats = PlayerStats::from_moves(&white_moves_owned);
        let black_stats = PlayerStats::from_moves(&black_moves_owned);

        Ok(GameAnalysis {
            game_id: game_id.to_string(),
            white_bot: white_name.to_string(),
            black_bot: black_name.to_string(),
            opening: None, // Opening detection not implemented
            result: result.to_string(),
            moves: analyzed_moves,
            white_stats,
            black_stats,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert_eq!(config.depth, 15);
        assert_eq!(config.opening_book_moves, 0);
    }

    #[test]
    fn test_move_input_clone() {
        let input = MoveInput {
            uci: "e2e4".to_string(),
            bot_eval_cp: Some(20),
            bot_eval_mate: None,
            bot_depth: Some(10),
            bot_nodes: Some(50000),
            bot_time_ms: Some(100),
            bot_pv: vec!["e2e4".to_string(), "e7e5".to_string()],
        };

        let cloned = input.clone();

        assert_eq!(cloned.uci, "e2e4");
        assert_eq!(cloned.bot_eval_cp, Some(20));
        assert_eq!(cloned.bot_eval_mate, None);
        assert_eq!(cloned.bot_depth, Some(10));
        assert_eq!(cloned.bot_nodes, Some(50000));
        assert_eq!(cloned.bot_time_ms, Some(100));
        assert_eq!(cloned.bot_pv, vec!["e2e4".to_string(), "e7e5".to_string()]);
    }

    #[test]
    fn test_analyzer_error_display() {
        // Test Engine error variant
        let engine_err = AnalyzerError::Engine(EngineError::NotFound("stockfish".to_string()));
        let display = format!("{}", engine_err);
        assert!(display.contains("Engine error"));
        assert!(display.contains("stockfish"));

        // Test InvalidGame error variant
        let invalid_err = AnalyzerError::InvalidGame("no moves provided".to_string());
        let display = format!("{}", invalid_err);
        assert!(display.contains("Invalid game data"));
        assert!(display.contains("no moves provided"));
    }

    #[test]
    fn test_move_input_debug() {
        let input = MoveInput {
            uci: "d2d4".to_string(),
            bot_eval_cp: None,
            bot_eval_mate: Some(3),
            bot_depth: None,
            bot_nodes: None,
            bot_time_ms: None,
            bot_pv: vec![],
        };

        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("d2d4"));
        assert!(debug_str.contains("MoveInput"));
    }

    #[test]
    fn test_analysis_config_clone() {
        let config = AnalysisConfig {
            depth: 20,
            opening_book_moves: 10,
        };

        let cloned = config.clone();
        assert_eq!(cloned.depth, 20);
        assert_eq!(cloned.opening_book_moves, 10);
    }
}
