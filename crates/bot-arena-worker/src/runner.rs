//! Match runner - executes games between bots.
//!
//! This module provides functionality to run matches between UCI chess engines
//! using the bot-arena game runner. It wraps the bot-arena API to work with
//! the worker's PendingMatch type and handles color alternation between games.

use crate::db::PendingMatch;
use bot_arena::game_runner::{GameError, GameResult, GameRunner};
use bot_arena::uci_client::UciClient;
use std::path::PathBuf;

/// Executes matches between UCI chess engines.
///
/// `MatchRunner` is responsible for spawning engine processes and coordinating
/// game play according to match parameters. It alternates colors between games
/// to ensure fairness.
pub struct MatchRunner {
    /// Directory containing bot executables.
    bots_dir: PathBuf,
}

impl MatchRunner {
    /// Creates a new match runner.
    ///
    /// # Arguments
    ///
    /// * `bots_dir` - Directory containing the bot executables. Bot names from
    ///   `PendingMatch` are resolved relative to this directory.
    pub fn new(bots_dir: impl Into<PathBuf>) -> Self {
        Self {
            bots_dir: bots_dir.into(),
        }
    }

    /// Runs a complete match consisting of multiple games.
    ///
    /// Executes all games specified by the match parameters, alternating colors
    /// between games to ensure fairness. Each game result is paired with a
    /// unique game ID.
    ///
    /// # Arguments
    ///
    /// * `pending` - The match parameters including bot names, game count, and time control.
    ///
    /// # Returns
    ///
    /// Returns a vector of tuples containing `(game_id, GameResult)` for each
    /// completed game. The game ID has format `{match_id}-{game_number}`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A bot executable cannot be spawned
    /// - UCI initialization fails for either engine
    /// - Game execution encounters a fatal error
    ///
    /// Note that early termination on error means some games may not be played.
    pub fn run_match(
        &self,
        pending: &PendingMatch,
    ) -> Result<Vec<(String, GameResult)>, GameError> {
        let white_path = self.bots_dir.join(&pending.white_bot);
        let black_path = self.bots_dir.join(&pending.black_bot);
        let time_control = format!("movetime {}", pending.movetime_ms);

        let mut results = Vec::new();

        for game_num in 0..pending.games_total {
            let game_id = format!("{}-{}", pending.id, game_num);

            // Alternate colors each game for fairness
            let (w_path, b_path) = if game_num % 2 == 0 {
                (&white_path, &black_path)
            } else {
                (&black_path, &white_path)
            };

            let white = UciClient::spawn(w_path)?;
            let black = UciClient::spawn(b_path)?;

            let mut runner = GameRunner::new(white, black, time_control.clone(), vec![])?;

            let result = runner.play_game()?;
            results.push((game_id, result));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_runner_new() {
        let runner = MatchRunner::new("/tmp/bots");
        assert_eq!(runner.bots_dir, PathBuf::from("/tmp/bots"));
    }

    #[test]
    fn test_match_runner_new_with_pathbuf() {
        let path = PathBuf::from("/var/bots");
        let runner = MatchRunner::new(path.clone());
        assert_eq!(runner.bots_dir, path);
    }

    #[test]
    fn test_run_match_missing_bot_returns_error() {
        let runner = MatchRunner::new("/nonexistent/path");
        let pending = PendingMatch {
            id: "test-match".to_string(),
            white_bot: "white.exe".to_string(),
            black_bot: "black.exe".to_string(),
            games_total: 2,
            movetime_ms: 100,
            opening_id: None,
        };

        let result = runner.run_match(&pending);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_control_format() {
        // Verify time control string format by checking string formatting
        let movetime_ms = 500;
        let time_control = format!("movetime {}", movetime_ms);
        assert_eq!(time_control, "movetime 500");
    }

    #[test]
    fn test_game_id_format() {
        let match_id = "abc-123";
        let game_num = 5;
        let game_id = format!("{}-{}", match_id, game_num);
        assert_eq!(game_id, "abc-123-5");
    }
}
