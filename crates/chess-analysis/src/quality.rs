//! Move quality classification and game analysis.

use serde::{Deserialize, Serialize};

use crate::Evaluation;

/// Classification of move quality based on evaluation loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveQuality {
    /// Matches top engine choice (0 cp loss)
    Best,
    /// Within 10cp of best
    Excellent,
    /// Within 30cp of best
    Good,
    /// 30-100cp worse than best
    Inaccuracy,
    /// 100-300cp worse than best
    Mistake,
    /// >300cp worse than best
    Blunder,
    /// Only legal move or opening book
    Forced,
}

impl MoveQuality {
    /// Classifies a move based on centipawn loss.
    ///
    /// # Arguments
    ///
    /// * `cp_loss` - The centipawn loss from playing this move instead of the best move.
    /// * `is_forced` - Whether this was a forced move (only legal move or opening book).
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::MoveQuality;
    ///
    /// assert_eq!(MoveQuality::from_cp_loss(0, false), MoveQuality::Best);
    /// assert_eq!(MoveQuality::from_cp_loss(5, false), MoveQuality::Excellent);
    /// assert_eq!(MoveQuality::from_cp_loss(20, false), MoveQuality::Good);
    /// assert_eq!(MoveQuality::from_cp_loss(50, false), MoveQuality::Inaccuracy);
    /// assert_eq!(MoveQuality::from_cp_loss(150, false), MoveQuality::Mistake);
    /// assert_eq!(MoveQuality::from_cp_loss(400, false), MoveQuality::Blunder);
    /// assert_eq!(MoveQuality::from_cp_loss(500, true), MoveQuality::Forced);
    /// ```
    pub fn from_cp_loss(cp_loss: i32, is_forced: bool) -> Self {
        if is_forced {
            return MoveQuality::Forced;
        }

        match cp_loss {
            0 => MoveQuality::Best,
            1..=10 => MoveQuality::Excellent,
            11..=30 => MoveQuality::Good,
            31..=100 => MoveQuality::Inaccuracy,
            101..=300 => MoveQuality::Mistake,
            _ => MoveQuality::Blunder,
        }
    }

    /// Returns true if this is a negative move quality (Inaccuracy, Mistake, or Blunder).
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::MoveQuality;
    ///
    /// assert!(!MoveQuality::Best.is_negative());
    /// assert!(!MoveQuality::Excellent.is_negative());
    /// assert!(!MoveQuality::Good.is_negative());
    /// assert!(MoveQuality::Inaccuracy.is_negative());
    /// assert!(MoveQuality::Mistake.is_negative());
    /// assert!(MoveQuality::Blunder.is_negative());
    /// assert!(!MoveQuality::Forced.is_negative());
    /// ```
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            MoveQuality::Inaccuracy | MoveQuality::Mistake | MoveQuality::Blunder
        )
    }
}

/// Analysis result for a single move.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAnalysis {
    /// The move that was played (in UCI notation)
    pub uci: String,
    /// The move in Standard Algebraic Notation
    pub san: Option<String>,
    /// Quality classification of the move
    pub quality: MoveQuality,
    /// Bot's own evaluation of the position
    pub bot_eval: Option<Evaluation>,
    /// Search depth used by the bot
    pub bot_depth: Option<u32>,
    /// Number of nodes searched by the bot
    pub bot_nodes: Option<u64>,
    /// Time spent by the bot in milliseconds
    pub bot_time_ms: Option<u64>,
    /// Principal variation from the bot's search
    pub bot_pv: Vec<String>,
    /// Engine evaluation before the move
    pub engine_eval_before: Option<Evaluation>,
    /// Engine evaluation after the move
    pub engine_eval_after: Option<Evaluation>,
    /// Best move according to the engine
    pub engine_best_move: Option<String>,
    /// Principal variation from the engine
    pub engine_pv: Vec<String>,
    /// Centipawn loss from playing this move
    pub centipawn_loss: Option<i32>,
}

/// Statistics for a player's performance in a game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Average centipawn loss per move
    pub avg_centipawn_loss: f32,
    /// Number of blunders
    pub blunders: u32,
    /// Number of mistakes
    pub mistakes: u32,
    /// Number of inaccuracies
    pub inaccuracies: u32,
    /// Average search depth
    pub avg_depth: f32,
    /// Average nodes searched
    pub avg_nodes: u64,
    /// Average time per move in milliseconds
    pub avg_time_ms: u64,
    /// Accuracy percentage (0-100)
    pub accuracy_percent: f32,
}

impl PlayerStats {
    /// Computes player statistics from a list of move analyses.
    ///
    /// The accuracy formula is based on the average centipawn loss,
    /// using an exponential decay formula similar to chess.com's accuracy.
    ///
    /// # Arguments
    ///
    /// * `moves` - Slice of move analyses for a single player.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::{PlayerStats, MoveAnalysis, MoveQuality};
    ///
    /// let stats = PlayerStats::from_moves(&[]);
    /// assert_eq!(stats.avg_centipawn_loss, 0.0);
    /// assert_eq!(stats.accuracy_percent, 100.0);
    /// ```
    pub fn from_moves(moves: &[MoveAnalysis]) -> Self {
        if moves.is_empty() {
            return PlayerStats {
                accuracy_percent: 100.0,
                ..Default::default()
            };
        }

        let mut total_cp_loss: i32 = 0;
        let mut cp_loss_count: u32 = 0;
        let mut blunders: u32 = 0;
        let mut mistakes: u32 = 0;
        let mut inaccuracies: u32 = 0;
        let mut total_depth: u32 = 0;
        let mut depth_count: u32 = 0;
        let mut total_nodes: u64 = 0;
        let mut nodes_count: u32 = 0;
        let mut total_time_ms: u64 = 0;
        let mut time_count: u32 = 0;

        for m in moves {
            // Count quality categories
            match m.quality {
                MoveQuality::Blunder => blunders += 1,
                MoveQuality::Mistake => mistakes += 1,
                MoveQuality::Inaccuracy => inaccuracies += 1,
                _ => {}
            }

            // Accumulate centipawn loss
            if let Some(cp) = m.centipawn_loss {
                total_cp_loss += cp;
                cp_loss_count += 1;
            }

            // Accumulate bot metrics
            if let Some(d) = m.bot_depth {
                total_depth += d;
                depth_count += 1;
            }
            if let Some(n) = m.bot_nodes {
                total_nodes += n;
                nodes_count += 1;
            }
            if let Some(t) = m.bot_time_ms {
                total_time_ms += t;
                time_count += 1;
            }
        }

        let avg_centipawn_loss = if cp_loss_count > 0 {
            total_cp_loss as f32 / cp_loss_count as f32
        } else {
            0.0
        };

        let avg_depth = if depth_count > 0 {
            total_depth as f32 / depth_count as f32
        } else {
            0.0
        };

        let avg_nodes = if nodes_count > 0 {
            total_nodes / nodes_count as u64
        } else {
            0
        };

        let avg_time_ms = if time_count > 0 {
            total_time_ms / time_count as u64
        } else {
            0
        };

        // Accuracy formula: exponential decay based on average centipawn loss
        // 0 cp loss = 100% accuracy, ~50 cp loss = ~50% accuracy
        // Formula: 100 * e^(-acpl / 50)
        let accuracy_percent = 100.0 * (-avg_centipawn_loss / 50.0).exp();
        let accuracy_percent = accuracy_percent.clamp(0.0, 100.0);

        PlayerStats {
            avg_centipawn_loss,
            blunders,
            mistakes,
            inaccuracies,
            avg_depth,
            avg_nodes,
            avg_time_ms,
            accuracy_percent,
        }
    }
}

/// Complete analysis of a chess game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalysis {
    /// Unique identifier for the game
    pub game_id: String,
    /// White bot identifier
    pub white_bot: String,
    /// Black bot identifier
    pub black_bot: String,
    /// Opening name if identified
    pub opening: Option<String>,
    /// Game result (e.g., "1-0", "0-1", "1/2-1/2")
    pub result: String,
    /// Analysis of each move
    pub moves: Vec<MoveAnalysis>,
    /// Statistics for white
    pub white_stats: PlayerStats,
    /// Statistics for black
    pub black_stats: PlayerStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_quality_from_cp_loss() {
        // Best: 0 cp loss
        assert_eq!(MoveQuality::from_cp_loss(0, false), MoveQuality::Best);

        // Excellent: 1-10 cp loss
        assert_eq!(MoveQuality::from_cp_loss(1, false), MoveQuality::Excellent);
        assert_eq!(MoveQuality::from_cp_loss(5, false), MoveQuality::Excellent);
        assert_eq!(MoveQuality::from_cp_loss(10, false), MoveQuality::Excellent);

        // Good: 11-30 cp loss
        assert_eq!(MoveQuality::from_cp_loss(11, false), MoveQuality::Good);
        assert_eq!(MoveQuality::from_cp_loss(20, false), MoveQuality::Good);
        assert_eq!(MoveQuality::from_cp_loss(30, false), MoveQuality::Good);

        // Inaccuracy: 31-100 cp loss
        assert_eq!(
            MoveQuality::from_cp_loss(31, false),
            MoveQuality::Inaccuracy
        );
        assert_eq!(
            MoveQuality::from_cp_loss(50, false),
            MoveQuality::Inaccuracy
        );
        assert_eq!(
            MoveQuality::from_cp_loss(100, false),
            MoveQuality::Inaccuracy
        );

        // Mistake: 101-300 cp loss
        assert_eq!(MoveQuality::from_cp_loss(101, false), MoveQuality::Mistake);
        assert_eq!(MoveQuality::from_cp_loss(200, false), MoveQuality::Mistake);
        assert_eq!(MoveQuality::from_cp_loss(300, false), MoveQuality::Mistake);

        // Blunder: >300 cp loss
        assert_eq!(MoveQuality::from_cp_loss(301, false), MoveQuality::Blunder);
        assert_eq!(MoveQuality::from_cp_loss(500, false), MoveQuality::Blunder);
        assert_eq!(MoveQuality::from_cp_loss(1000, false), MoveQuality::Blunder);
    }

    #[test]
    fn test_move_quality_forced() {
        // Forced should override any cp_loss value
        assert_eq!(MoveQuality::from_cp_loss(0, true), MoveQuality::Forced);
        assert_eq!(MoveQuality::from_cp_loss(50, true), MoveQuality::Forced);
        assert_eq!(MoveQuality::from_cp_loss(500, true), MoveQuality::Forced);
    }

    #[test]
    fn test_move_quality_is_negative() {
        // Positive qualities
        assert!(!MoveQuality::Best.is_negative());
        assert!(!MoveQuality::Excellent.is_negative());
        assert!(!MoveQuality::Good.is_negative());
        assert!(!MoveQuality::Forced.is_negative());

        // Negative qualities
        assert!(MoveQuality::Inaccuracy.is_negative());
        assert!(MoveQuality::Mistake.is_negative());
        assert!(MoveQuality::Blunder.is_negative());
    }

    #[test]
    fn test_player_stats_from_empty_moves() {
        let stats = PlayerStats::from_moves(&[]);

        assert_eq!(stats.avg_centipawn_loss, 0.0);
        assert_eq!(stats.blunders, 0);
        assert_eq!(stats.mistakes, 0);
        assert_eq!(stats.inaccuracies, 0);
        assert_eq!(stats.avg_depth, 0.0);
        assert_eq!(stats.avg_nodes, 0);
        assert_eq!(stats.avg_time_ms, 0);
        assert_eq!(stats.accuracy_percent, 100.0);
    }

    #[test]
    fn test_player_stats_accuracy() {
        // Create moves with known cp_loss values
        let moves = vec![
            MoveAnalysis {
                uci: "e2e4".to_string(),
                san: Some("e4".to_string()),
                quality: MoveQuality::Best,
                bot_eval: None,
                bot_depth: Some(20),
                bot_nodes: Some(1000000),
                bot_time_ms: Some(500),
                bot_pv: vec![],
                engine_eval_before: None,
                engine_eval_after: None,
                engine_best_move: Some("e2e4".to_string()),
                engine_pv: vec![],
                centipawn_loss: Some(0),
            },
            MoveAnalysis {
                uci: "d2d4".to_string(),
                san: Some("d4".to_string()),
                quality: MoveQuality::Good,
                bot_eval: None,
                bot_depth: Some(18),
                bot_nodes: Some(800000),
                bot_time_ms: Some(400),
                bot_pv: vec![],
                engine_eval_before: None,
                engine_eval_after: None,
                engine_best_move: Some("c2c4".to_string()),
                engine_pv: vec![],
                centipawn_loss: Some(20),
            },
            MoveAnalysis {
                uci: "a2a4".to_string(),
                san: Some("a4".to_string()),
                quality: MoveQuality::Inaccuracy,
                bot_eval: None,
                bot_depth: Some(22),
                bot_nodes: Some(1200000),
                bot_time_ms: Some(600),
                bot_pv: vec![],
                engine_eval_before: None,
                engine_eval_after: None,
                engine_best_move: Some("b1c3".to_string()),
                engine_pv: vec![],
                centipawn_loss: Some(50),
            },
        ];

        let stats = PlayerStats::from_moves(&moves);

        // Check counts
        assert_eq!(stats.blunders, 0);
        assert_eq!(stats.mistakes, 0);
        assert_eq!(stats.inaccuracies, 1);

        // Check average centipawn loss: (0 + 20 + 50) / 3 = 23.33...
        let expected_acpl = (0.0 + 20.0 + 50.0) / 3.0;
        assert!((stats.avg_centipawn_loss - expected_acpl).abs() < 0.01);

        // Check average depth: (20 + 18 + 22) / 3 = 20
        assert!((stats.avg_depth - 20.0).abs() < 0.01);

        // Check average nodes: (1000000 + 800000 + 1200000) / 3 = 1000000
        assert_eq!(stats.avg_nodes, 1000000);

        // Check average time: (500 + 400 + 600) / 3 = 500
        assert_eq!(stats.avg_time_ms, 500);

        // Accuracy should be 100 * e^(-23.33/50) = ~62.7%
        let expected_accuracy = 100.0 * (-expected_acpl / 50.0).exp();
        assert!((stats.accuracy_percent - expected_accuracy).abs() < 0.1);
    }

    #[test]
    fn test_player_stats_with_all_quality_types() {
        let moves = vec![
            MoveAnalysis {
                uci: "e2e4".to_string(),
                san: None,
                quality: MoveQuality::Blunder,
                bot_eval: None,
                bot_depth: None,
                bot_nodes: None,
                bot_time_ms: None,
                bot_pv: vec![],
                engine_eval_before: None,
                engine_eval_after: None,
                engine_best_move: None,
                engine_pv: vec![],
                centipawn_loss: Some(400),
            },
            MoveAnalysis {
                uci: "d2d4".to_string(),
                san: None,
                quality: MoveQuality::Mistake,
                bot_eval: None,
                bot_depth: None,
                bot_nodes: None,
                bot_time_ms: None,
                bot_pv: vec![],
                engine_eval_before: None,
                engine_eval_after: None,
                engine_best_move: None,
                engine_pv: vec![],
                centipawn_loss: Some(200),
            },
            MoveAnalysis {
                uci: "b1c3".to_string(),
                san: None,
                quality: MoveQuality::Inaccuracy,
                bot_eval: None,
                bot_depth: None,
                bot_nodes: None,
                bot_time_ms: None,
                bot_pv: vec![],
                engine_eval_before: None,
                engine_eval_after: None,
                engine_best_move: None,
                engine_pv: vec![],
                centipawn_loss: Some(60),
            },
        ];

        let stats = PlayerStats::from_moves(&moves);

        assert_eq!(stats.blunders, 1);
        assert_eq!(stats.mistakes, 1);
        assert_eq!(stats.inaccuracies, 1);
    }

    #[test]
    fn test_move_quality_serialization() {
        let quality = MoveQuality::Excellent;
        let json = serde_json::to_string(&quality).unwrap();
        let parsed: MoveQuality = serde_json::from_str(&json).unwrap();
        assert_eq!(quality, parsed);
    }

    #[test]
    fn test_move_analysis_serialization() {
        let analysis = MoveAnalysis {
            uci: "e2e4".to_string(),
            san: Some("e4".to_string()),
            quality: MoveQuality::Best,
            bot_eval: Some(Evaluation::Centipawn(35)),
            bot_depth: Some(20),
            bot_nodes: Some(1000000),
            bot_time_ms: Some(500),
            bot_pv: vec!["e2e4".to_string(), "e7e5".to_string()],
            engine_eval_before: Some(Evaluation::Centipawn(0)),
            engine_eval_after: Some(Evaluation::Centipawn(35)),
            engine_best_move: Some("e2e4".to_string()),
            engine_pv: vec!["e2e4".to_string(), "e7e5".to_string()],
            centipawn_loss: Some(0),
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let parsed: MoveAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.uci, "e2e4");
        assert_eq!(parsed.quality, MoveQuality::Best);
        assert_eq!(parsed.centipawn_loss, Some(0));
    }

    #[test]
    fn test_player_stats_serialization() {
        let stats = PlayerStats {
            avg_centipawn_loss: 25.5,
            blunders: 1,
            mistakes: 2,
            inaccuracies: 3,
            avg_depth: 18.5,
            avg_nodes: 500000,
            avg_time_ms: 300,
            accuracy_percent: 75.5,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let parsed: PlayerStats = serde_json::from_str(&json).unwrap();

        assert!((parsed.avg_centipawn_loss - 25.5).abs() < 0.01);
        assert_eq!(parsed.blunders, 1);
        assert_eq!(parsed.mistakes, 2);
        assert_eq!(parsed.inaccuracies, 3);
    }

    #[test]
    fn test_game_analysis_serialization() {
        let game = GameAnalysis {
            game_id: "game-001".to_string(),
            white_bot: "stockfish-10".to_string(),
            black_bot: "komodo-14".to_string(),
            opening: Some("Sicilian Defense".to_string()),
            result: "1-0".to_string(),
            moves: vec![],
            white_stats: PlayerStats::default(),
            black_stats: PlayerStats::default(),
        };

        let json = serde_json::to_string(&game).unwrap();
        let parsed: GameAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.game_id, "game-001");
        assert_eq!(parsed.white_bot, "stockfish-10");
        assert_eq!(parsed.black_bot, "komodo-14");
        assert_eq!(parsed.opening, Some("Sicilian Defense".to_string()));
        assert_eq!(parsed.result, "1-0");
    }
}
