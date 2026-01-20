//! SQLite storage for chess game results and bot statistics.
//!
//! This module provides persistent storage for game results, bot information,
//! and performance statistics using SQLite as the backing database.

#[cfg(test)]
use crate::game_runner::MoveRecord;
use crate::game_runner::{GameResult, MatchResult};
use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use uuid::Uuid;

/// SQLite-backed storage for game results and bot statistics.
///
/// Provides methods to persist game outcomes, track bot performance,
/// and retrieve aggregate statistics across matches.
///
/// # Example
///
/// ```ignore
/// let storage = Storage::open("data/arena.db")?;
/// storage.ensure_bot("stockfish", Some("/usr/bin/stockfish"))?;
/// storage.save_game(&game_result)?;
/// let (games, wins, draws, losses) = storage.get_stats("stockfish")?;
/// ```
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Opens or creates a SQLite database at the given path.
    ///
    /// If the database does not exist, it will be created. The schema
    /// is automatically initialized on first open.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path where the database file should be stored.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or if schema
    /// initialization fails.
    pub fn open<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    /// Initializes the database schema if tables do not exist.
    fn init_schema(&self) -> SqliteResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS bots (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT
            );

            CREATE TABLE IF NOT EXISTS games (
                id TEXT PRIMARY KEY,
                white_bot TEXT NOT NULL,
                black_bot TEXT NOT NULL,
                result TEXT NOT NULL,
                move_count INTEGER NOT NULL,
                moves TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS bot_stats (
                bot_id TEXT NOT NULL,
                opponent_id TEXT NOT NULL,
                games INTEGER DEFAULT 0,
                wins INTEGER DEFAULT 0,
                draws INTEGER DEFAULT 0,
                losses INTEGER DEFAULT 0,
                PRIMARY KEY (bot_id, opponent_id)
            );
            ",
        )
    }

    /// Ensures a bot exists in the database.
    ///
    /// If a bot with the given name does not exist, it will be inserted.
    /// If it already exists, this operation is a no-op.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique identifier/name for the bot.
    /// * `path` - Optional filesystem path to the bot executable.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub fn ensure_bot(&self, name: &str, path: Option<&str>) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO bots (id, name, path) VALUES (?1, ?1, ?2)",
            [name, path.unwrap_or("")],
        )?;
        Ok(())
    }

    /// Saves a game result to the database.
    ///
    /// This method persists the game outcome and updates the statistics
    /// for both participating bots.
    ///
    /// # Arguments
    ///
    /// * `result` - The completed game result to save.
    ///
    /// # Returns
    ///
    /// Returns the unique ID assigned to the saved game.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operations fail.
    pub fn save_game(&self, result: &GameResult) -> SqliteResult<String> {
        let id = Uuid::new_v4().to_string();
        let result_str = match result.result {
            MatchResult::WhiteWins => "white",
            MatchResult::BlackWins => "black",
            MatchResult::Draw => "draw",
        };

        // Extract UCI moves for storage
        let moves_str = result
            .moves
            .iter()
            .map(|m| m.uci.clone())
            .collect::<Vec<_>>()
            .join(" ");

        self.conn.execute(
            "INSERT INTO games (id, white_bot, black_bot, result, move_count, moves, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &id,
                &result.white_name,
                &result.black_name,
                result_str,
                result.moves.len() as i32,
                moves_str,
                Utc::now().to_rfc3339(),
            ),
        )?;

        self.update_stats(&result.white_name, &result.black_name, result.result)?;

        Ok(id)
    }

    /// Updates the statistics for both bots after a game.
    fn update_stats(&self, white: &str, black: &str, result: MatchResult) -> SqliteResult<()> {
        // Update white's stats
        self.conn.execute(
            "INSERT INTO bot_stats (bot_id, opponent_id, games, wins, draws, losses)
             VALUES (?1, ?2, 1, ?3, ?4, ?5)
             ON CONFLICT(bot_id, opponent_id) DO UPDATE SET
                games = games + 1,
                wins = wins + ?3,
                draws = draws + ?4,
                losses = losses + ?5",
            (
                white,
                black,
                if result == MatchResult::WhiteWins {
                    1
                } else {
                    0
                },
                if result == MatchResult::Draw { 1 } else { 0 },
                if result == MatchResult::BlackWins {
                    1
                } else {
                    0
                },
            ),
        )?;

        // Update black's stats (mirror)
        self.conn.execute(
            "INSERT INTO bot_stats (bot_id, opponent_id, games, wins, draws, losses)
             VALUES (?1, ?2, 1, ?3, ?4, ?5)
             ON CONFLICT(bot_id, opponent_id) DO UPDATE SET
                games = games + 1,
                wins = wins + ?3,
                draws = draws + ?4,
                losses = losses + ?5",
            (
                black,
                white,
                if result == MatchResult::BlackWins {
                    1
                } else {
                    0
                },
                if result == MatchResult::Draw { 1 } else { 0 },
                if result == MatchResult::WhiteWins {
                    1
                } else {
                    0
                },
            ),
        )?;

        Ok(())
    }

    /// Retrieves aggregate statistics for a bot.
    ///
    /// Returns the total games, wins, draws, and losses for a bot
    /// across all opponents.
    ///
    /// # Arguments
    ///
    /// * `bot` - The bot name to retrieve statistics for.
    ///
    /// # Returns
    ///
    /// A tuple of `(games, wins, draws, losses)`. Returns `(0, 0, 0, 0)`
    /// if the bot has no recorded games.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_stats(&self, bot: &str) -> SqliteResult<(i32, i32, i32, i32)> {
        let mut stmt = self.conn.prepare(
            "SELECT SUM(games), SUM(wins), SUM(draws), SUM(losses)
             FROM bot_stats WHERE bot_id = ?1",
        )?;

        stmt.query_row([bot], |row| {
            Ok((
                row.get::<_, Option<i32>>(0)?.unwrap_or(0),
                row.get::<_, Option<i32>>(1)?.unwrap_or(0),
                row.get::<_, Option<i32>>(2)?.unwrap_or(0),
                row.get::<_, Option<i32>>(3)?.unwrap_or(0),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create an in-memory database for testing.
    fn create_test_storage() -> Storage {
        Storage::open(":memory:").expect("Failed to create in-memory storage")
    }

    #[test]
    fn test_open_creates_tables() {
        let storage = create_test_storage();

        // Verify bots table exists
        let bots_exists: bool = storage
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='bots'",
                [],
                |row| row.get(0),
            )
            .map(|count: i32| count > 0)
            .unwrap();
        assert!(bots_exists, "bots table should exist");

        // Verify games table exists
        let games_exists: bool = storage
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='games'",
                [],
                |row| row.get(0),
            )
            .map(|count: i32| count > 0)
            .unwrap();
        assert!(games_exists, "games table should exist");

        // Verify bot_stats table exists
        let stats_exists: bool = storage
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='bot_stats'",
                [],
                |row| row.get(0),
            )
            .map(|count: i32| count > 0)
            .unwrap();
        assert!(stats_exists, "bot_stats table should exist");
    }

    #[test]
    fn test_ensure_bot_inserts_bot() {
        let storage = create_test_storage();

        storage
            .ensure_bot("stockfish", Some("/usr/bin/stockfish"))
            .expect("Failed to ensure bot");

        let (name, path): (String, String) = storage
            .conn
            .query_row(
                "SELECT name, path FROM bots WHERE id = 'stockfish'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("Bot should exist");

        assert_eq!(name, "stockfish");
        assert_eq!(path, "/usr/bin/stockfish");

        // Ensure idempotent - calling again should not fail
        storage
            .ensure_bot("stockfish", Some("/different/path"))
            .expect("Second ensure should succeed");

        // Path should remain unchanged (INSERT OR IGNORE)
        let path_after: String = storage
            .conn
            .query_row("SELECT path FROM bots WHERE id = 'stockfish'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(path_after, "/usr/bin/stockfish");
    }

    #[test]
    fn test_save_game_and_get_stats() {
        let storage = create_test_storage();

        // Create a game result
        let result = GameResult {
            moves: vec![
                MoveRecord {
                    uci: "e2e4".to_string(),
                    search_info: None,
                },
                MoveRecord {
                    uci: "e7e5".to_string(),
                    search_info: None,
                },
            ],
            result: MatchResult::WhiteWins,
            white_name: "engine_a".to_string(),
            black_name: "engine_b".to_string(),
            opening: None,
        };

        let game_id = storage.save_game(&result).expect("Failed to save game");
        assert!(!game_id.is_empty(), "Game ID should not be empty");

        // Check white's stats
        let (games, wins, draws, losses) =
            storage.get_stats("engine_a").expect("Failed to get stats");
        assert_eq!(games, 1);
        assert_eq!(wins, 1);
        assert_eq!(draws, 0);
        assert_eq!(losses, 0);

        // Check black's stats (should have a loss)
        let (games, wins, draws, losses) =
            storage.get_stats("engine_b").expect("Failed to get stats");
        assert_eq!(games, 1);
        assert_eq!(wins, 0);
        assert_eq!(draws, 0);
        assert_eq!(losses, 1);

        // Add another game - a draw
        let draw_result = GameResult {
            moves: vec![MoveRecord {
                uci: "d2d4".to_string(),
                search_info: None,
            }],
            result: MatchResult::Draw,
            white_name: "engine_a".to_string(),
            black_name: "engine_b".to_string(),
            opening: None,
        };
        storage
            .save_game(&draw_result)
            .expect("Failed to save draw");

        // Check updated stats
        let (games, wins, draws, losses) =
            storage.get_stats("engine_a").expect("Failed to get stats");
        assert_eq!(games, 2);
        assert_eq!(wins, 1);
        assert_eq!(draws, 1);
        assert_eq!(losses, 0);
    }

    #[test]
    fn test_stats_for_unknown_bot_returns_zeros() {
        let storage = create_test_storage();

        let (games, wins, draws, losses) = storage
            .get_stats("nonexistent_bot")
            .expect("Should return zeros for unknown bot");

        assert_eq!(games, 0);
        assert_eq!(wins, 0);
        assert_eq!(draws, 0);
        assert_eq!(losses, 0);
    }
}
