//! Bot repository for database operations.

use crate::db::DbPool;
use crate::elo;
use crate::models::{Bot, BotProfile, EloHistoryPoint};
use rusqlite::OptionalExtension;
use rusqlite::Result as SqliteResult;

/// Repository for bot database operations.
pub struct BotRepo {
    db: DbPool,
}

impl BotRepo {
    /// Create a new bot repository with the given database pool.
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// List all bots, ordered by Elo rating (descending).
    pub fn list(&self) -> SqliteResult<Vec<Bot>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, elo_rating, games_played, wins, losses, draws
             FROM bots ORDER BY elo_rating DESC",
        )?;

        let bots = stmt
            .query_map([], |row| {
                Ok(Bot {
                    name: row.get(0)?,
                    elo_rating: row.get(1)?,
                    games_played: row.get(2)?,
                    wins: row.get(3)?,
                    losses: row.get(4)?,
                    draws: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(bots)
    }

    /// Get a bot by name.
    ///
    /// Returns `None` if the bot doesn't exist.
    pub fn get(&self, name: &str) -> SqliteResult<Option<Bot>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, elo_rating, games_played, wins, losses, draws
             FROM bots WHERE name = ?1",
        )?;

        stmt.query_row([name], |row| {
            Ok(Bot {
                name: row.get(0)?,
                elo_rating: row.get(1)?,
                games_played: row.get(2)?,
                wins: row.get(3)?,
                losses: row.get(4)?,
                draws: row.get(5)?,
            })
        })
        .optional()
    }

    /// Ensure a bot exists in the database.
    ///
    /// If the bot doesn't exist, it will be created with default values.
    /// If it already exists, this is a no-op.
    // Justification: Will be used by match handlers when registering bots (Phase 5 tasks).
    #[allow(dead_code)]
    pub fn ensure(&self, name: &str) -> SqliteResult<()> {
        let conn = self.db.lock().unwrap();
        conn.execute("INSERT OR IGNORE INTO bots (name) VALUES (?1)", [name])?;
        Ok(())
    }

    /// Update bot stats and Elo after a game.
    ///
    /// # Arguments
    /// * `name` - Bot name
    /// * `opponent_rating` - Opponent's Elo rating
    /// * `result` - 1.0 = win, 0.5 = draw, 0.0 = loss
    // Justification: Will be used by match handlers to update ratings after games (Phase 5 tasks).
    #[allow(dead_code)]
    pub fn update_after_game(
        &self,
        name: &str,
        opponent_rating: i32,
        result: f64,
    ) -> SqliteResult<i32> {
        let conn = self.db.lock().unwrap();

        // Get current rating
        let current_rating: i32 = conn.query_row(
            "SELECT elo_rating FROM bots WHERE name = ?1",
            [name],
            |row| row.get(0),
        )?;

        let new_rating = elo::new_rating(current_rating, opponent_rating, result);

        let (wins, draws, losses) = match result {
            r if r > 0.9 => (1, 0, 0),
            r if r > 0.1 => (0, 1, 0),
            _ => (0, 0, 1),
        };

        conn.execute(
            "UPDATE bots SET
                elo_rating = ?1,
                games_played = games_played + 1,
                wins = wins + ?2,
                draws = draws + ?3,
                losses = losses + ?4
             WHERE name = ?5",
            (new_rating, wins, draws, losses, name),
        )?;

        Ok(new_rating)
    }

    /// Get Elo history for a bot, ordered by timestamp ascending.
    pub fn get_elo_history(&self, name: &str) -> SqliteResult<Vec<EloHistoryPoint>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT elo_rating, recorded_at FROM elo_history
             WHERE bot_name = ?1 ORDER BY recorded_at ASC",
        )?;

        let history = stmt
            .query_map([name], |row| {
                Ok(EloHistoryPoint {
                    elo: row.get(0)?,
                    timestamp: row.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(history)
    }

    /// Get a bot profile with Elo history.
    ///
    /// Returns `None` if the bot doesn't exist.
    pub fn get_profile(&self, name: &str) -> SqliteResult<Option<BotProfile>> {
        let bot = self.get(name)?;
        match bot {
            None => Ok(None),
            Some(bot) => {
                let elo_history = self.get_elo_history(name)?;
                Ok(Some(BotProfile {
                    name: bot.name,
                    elo_rating: bot.elo_rating,
                    games_played: bot.games_played,
                    wins: bot.wins,
                    draws: bot.draws,
                    losses: bot.losses,
                    elo_history,
                }))
            }
        }
    }

    /// Record an Elo rating change in history.
    ///
    /// This should be called after each game to track rating progression.
    // Justification: Will be used by match handlers to record rating history (Phase 5 tasks).
    #[allow(dead_code)]
    pub fn record_elo_history(
        &self,
        name: &str,
        elo_rating: i32,
        match_id: Option<&str>,
    ) -> SqliteResult<()> {
        let conn = self.db.lock().unwrap();
        conn.execute(
            "INSERT INTO elo_history (bot_name, elo_rating, recorded_at, match_id)
             VALUES (?1, ?2, datetime('now'), ?3)",
            (name, elo_rating, match_id),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;

    #[test]
    fn test_list_bots_empty() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db);
        let bots = repo.list().unwrap();
        assert!(bots.is_empty());
    }

    #[test]
    fn test_ensure_and_get_bot() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db);

        repo.ensure("stockfish").unwrap();

        let bot = repo.get("stockfish").unwrap();
        assert!(bot.is_some());
        let bot = bot.unwrap();
        assert_eq!(bot.name, "stockfish");
        assert_eq!(bot.elo_rating, 1500);
    }

    #[test]
    fn test_get_nonexistent_bot() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db);

        let bot = repo.get("nonexistent").unwrap();
        assert!(bot.is_none());
    }

    #[test]
    fn test_ensure_idempotent() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db);

        // Ensure the same bot twice - should not fail
        repo.ensure("stockfish").unwrap();
        repo.ensure("stockfish").unwrap();

        let bots = repo.list().unwrap();
        assert_eq!(bots.len(), 1);
    }

    #[test]
    fn test_list_bots_ordered_by_elo() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db.clone());

        // Insert bots with different Elo ratings
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO bots (name, elo_rating) VALUES ('weak_bot', 1200)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO bots (name, elo_rating) VALUES ('strong_bot', 1800)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO bots (name, elo_rating) VALUES ('medium_bot', 1500)",
                [],
            )
            .unwrap();
        }

        let bots = repo.list().unwrap();
        assert_eq!(bots.len(), 3);
        assert_eq!(bots[0].name, "strong_bot");
        assert_eq!(bots[1].name, "medium_bot");
        assert_eq!(bots[2].name, "weak_bot");
    }

    #[test]
    fn test_bot_default_stats() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db);

        repo.ensure("new_bot").unwrap();
        let bot = repo.get("new_bot").unwrap().unwrap();

        assert_eq!(bot.elo_rating, 1500);
        assert_eq!(bot.games_played, 0);
        assert_eq!(bot.wins, 0);
        assert_eq!(bot.losses, 0);
        assert_eq!(bot.draws, 0);
    }

    #[test]
    fn test_update_after_game() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db);

        repo.ensure("bot_a").unwrap();
        repo.ensure("bot_b").unwrap();

        // bot_a wins against bot_b (both start at 1500)
        let new_rating = repo.update_after_game("bot_a", 1500, 1.0).unwrap();
        assert_eq!(new_rating, 1516);

        let bot = repo.get("bot_a").unwrap().unwrap();
        assert_eq!(bot.elo_rating, 1516);
        assert_eq!(bot.games_played, 1);
        assert_eq!(bot.wins, 1);
    }

    #[test]
    fn test_get_elo_history_empty() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db);

        repo.ensure("test_bot").unwrap();
        let history = repo.get_elo_history("test_bot").unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_record_and_get_elo_history() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db.clone());

        repo.ensure("test_bot").unwrap();
        repo.ensure("opponent_bot").unwrap();
        repo.record_elo_history("test_bot", 1500, None).unwrap();

        // Create a match to reference for the second history entry
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at) VALUES ('match1', 'test_bot', 'opponent_bot', 1, '2025-01-21')",
                [],
            )
            .unwrap();
        }
        repo.record_elo_history("test_bot", 1520, Some("match1"))
            .unwrap();

        let history = repo.get_elo_history("test_bot").unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].elo, 1500);
        assert_eq!(history[1].elo, 1520);
    }

    #[test]
    fn test_get_profile_with_history() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db.clone());

        repo.ensure("stockfish").unwrap();

        // Add some Elo history entries
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO elo_history (bot_name, elo_rating, recorded_at) VALUES ('stockfish', 1500, '2025-01-01T10:00:00')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO elo_history (bot_name, elo_rating, recorded_at) VALUES ('stockfish', 1550, '2025-01-02T10:00:00')",
                [],
            )
            .unwrap();
        }

        let profile = repo.get_profile("stockfish").unwrap().unwrap();
        assert_eq!(profile.name, "stockfish");
        assert_eq!(profile.elo_rating, 1500);
        assert_eq!(profile.elo_history.len(), 2);
        // Verify history is ordered by timestamp ascending
        assert_eq!(profile.elo_history[0].elo, 1500);
        assert_eq!(profile.elo_history[1].elo, 1550);
    }

    #[test]
    fn test_get_profile_nonexistent() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db);

        let profile = repo.get_profile("nonexistent").unwrap();
        assert!(profile.is_none());
    }

    #[test]
    fn test_elo_history_ordered_by_timestamp() {
        let db = init_db(":memory:").unwrap();
        let repo = BotRepo::new(db.clone());

        repo.ensure("test_bot").unwrap();

        // Insert history entries in non-chronological order
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO elo_history (bot_name, elo_rating, recorded_at) VALUES ('test_bot', 1600, '2025-01-03T10:00:00')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO elo_history (bot_name, elo_rating, recorded_at) VALUES ('test_bot', 1500, '2025-01-01T10:00:00')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO elo_history (bot_name, elo_rating, recorded_at) VALUES ('test_bot', 1550, '2025-01-02T10:00:00')",
                [],
            )
            .unwrap();
        }

        let history = repo.get_elo_history("test_bot").unwrap();
        assert_eq!(history.len(), 3);
        // Should be ordered by timestamp ascending
        assert_eq!(history[0].elo, 1500);
        assert_eq!(history[1].elo, 1550);
        assert_eq!(history[2].elo, 1600);
    }
}
