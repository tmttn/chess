//! API models for serialization.

use serde::{Deserialize, Serialize};

/// Bot information with statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    /// Unique bot name/identifier.
    pub name: String,
    /// Current Elo rating.
    pub elo_rating: i32,
    /// Total number of games played.
    pub games_played: i32,
    /// Number of games won.
    pub wins: i32,
    /// Number of games lost.
    pub losses: i32,
    /// Number of games drawn.
    pub draws: i32,
}

/// Bot profile with detailed statistics and Elo history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotProfile {
    /// Unique bot name/identifier.
    pub name: String,
    /// Current Elo rating.
    pub elo_rating: i32,
    /// Total number of games played.
    pub games_played: i32,
    /// Number of games won.
    pub wins: i32,
    /// Number of games drawn.
    pub draws: i32,
    /// Number of games lost.
    pub losses: i32,
    /// Historical Elo rating data points.
    pub elo_history: Vec<EloHistoryPoint>,
}

/// A single point in the Elo history timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EloHistoryPoint {
    /// Elo rating at this point in time.
    pub elo: i32,
    /// Timestamp when this rating was recorded.
    pub timestamp: String,
}

impl Bot {
    /// Calculate win rate as a value between 0.0 and 1.0.
    ///
    /// Draws count as 0.5 wins for this calculation.
    /// Returns 0.0 if no games have been played.
    // Justification: Will be used in frontend statistics display (Phase 5, later tasks)
    #[allow(dead_code)]
    pub fn win_rate(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            (self.wins as f64 + self.draws as f64 * 0.5) / self.games_played as f64
        }
    }
}

/// A match (series of games) between two bots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    /// Unique match identifier.
    pub id: String,
    /// Name of the bot playing white.
    pub white_bot: String,
    /// Name of the bot playing black.
    pub black_bot: String,
    /// Total number of games in this match.
    pub games_total: i32,
    /// Score for the white bot (wins + draws * 0.5).
    pub white_score: f64,
    /// Score for the black bot (wins + draws * 0.5).
    pub black_score: f64,
    /// Optional opening database identifier.
    pub opening_id: Option<String>,
    /// Time per move in milliseconds.
    pub movetime_ms: i32,
    /// When the match started.
    pub started_at: String,
    /// When the match finished (if complete).
    pub finished_at: Option<String>,
    /// Match status (pending, running, completed, failed).
    pub status: String,
    /// Worker ID processing this match (if assigned).
    pub worker_id: Option<String>,
}

/// A single game within a match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// Unique game identifier.
    pub id: String,
    /// Match this game belongs to.
    pub match_id: String,
    /// Game number within the match (1-indexed).
    pub game_number: i32,
    /// Game result (1-0, 0-1, 1/2-1/2, or None if in progress).
    pub result: Option<String>,
    /// Name of the opening played.
    pub opening_name: Option<String>,
    /// Full PGN of the game.
    pub pgn: Option<String>,
}

/// A single move in a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    /// Ply number (half-move count, 1-indexed).
    pub ply: i32,
    /// Move in UCI notation (e.g., "e2e4").
    pub uci: String,
    /// Move in SAN notation (e.g., "e4").
    pub san: Option<String>,
    /// FEN position after this move.
    pub fen_after: String,
    /// Bot's evaluation in centipawns.
    pub bot_eval: Option<i32>,
    /// Stockfish's evaluation in centipawns.
    pub stockfish_eval: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_rate_no_games() {
        let bot = Bot {
            name: "test".to_string(),
            elo_rating: 1500,
            games_played: 0,
            wins: 0,
            losses: 0,
            draws: 0,
        };
        assert_eq!(bot.win_rate(), 0.0);
    }

    #[test]
    fn test_win_rate_all_wins() {
        let bot = Bot {
            name: "test".to_string(),
            elo_rating: 1500,
            games_played: 10,
            wins: 10,
            losses: 0,
            draws: 0,
        };
        assert_eq!(bot.win_rate(), 1.0);
    }

    #[test]
    fn test_win_rate_all_losses() {
        let bot = Bot {
            name: "test".to_string(),
            elo_rating: 1500,
            games_played: 10,
            wins: 0,
            losses: 10,
            draws: 0,
        };
        assert_eq!(bot.win_rate(), 0.0);
    }

    #[test]
    fn test_win_rate_mixed() {
        let bot = Bot {
            name: "test".to_string(),
            elo_rating: 1500,
            games_played: 10,
            wins: 5,
            losses: 3,
            draws: 2,
        };
        // 5 wins + 2 * 0.5 draws = 6.0 points out of 10 games
        assert_eq!(bot.win_rate(), 0.6);
    }

    #[test]
    fn test_win_rate_all_draws() {
        let bot = Bot {
            name: "test".to_string(),
            elo_rating: 1500,
            games_played: 10,
            wins: 0,
            losses: 0,
            draws: 10,
        };
        // 10 * 0.5 = 5.0 points out of 10 games
        assert_eq!(bot.win_rate(), 0.5);
    }
}
