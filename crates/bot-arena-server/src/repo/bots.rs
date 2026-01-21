//! Bot repository for database operations.

use crate::db::DbPool;
use crate::models::Bot;
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
    // Justification: Will be used when registering bots for matches (Phase 5, later tasks)
    #[allow(dead_code)]
    pub fn ensure(&self, name: &str) -> SqliteResult<()> {
        let conn = self.db.lock().unwrap();
        conn.execute("INSERT OR IGNORE INTO bots (name) VALUES (?1)", [name])?;
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
}
