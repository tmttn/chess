//! Database module for bot arena server.

use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Thread-safe database connection pool.
pub type DbPool = Arc<Mutex<Connection>>;

/// Initialize database with schema.
///
/// Creates all necessary tables for the bot arena:
/// - `bots`: Bot registration and Elo ratings
/// - `matches`: Multi-game series between two bots
/// - `games`: Individual games within a match
/// - `moves`: Move-by-move storage with evaluation data
///
/// # Arguments
///
/// * `path` - Path to the SQLite database file (use `:memory:` for in-memory)
///
/// # Errors
///
/// Returns an error if the database cannot be opened or schema creation fails.
pub fn init_db<P: AsRef<Path>>(path: P) -> SqliteResult<DbPool> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS bots (
            name TEXT PRIMARY KEY,
            elo_rating INTEGER DEFAULT 1500,
            games_played INTEGER DEFAULT 0,
            wins INTEGER DEFAULT 0,
            losses INTEGER DEFAULT 0,
            draws INTEGER DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS matches (
            id TEXT PRIMARY KEY,
            white_bot TEXT NOT NULL REFERENCES bots(name),
            black_bot TEXT NOT NULL REFERENCES bots(name),
            games_total INTEGER NOT NULL,
            white_score REAL DEFAULT 0,
            black_score REAL DEFAULT 0,
            opening_id TEXT,
            movetime_ms INTEGER DEFAULT 1000,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            status TEXT DEFAULT 'pending',
            worker_id TEXT
        );

        CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            match_id TEXT NOT NULL REFERENCES matches(id),
            game_number INTEGER NOT NULL,
            result TEXT,
            opening_name TEXT,
            pgn TEXT,
            started_at TEXT NOT NULL,
            finished_at TEXT
        );

        CREATE TABLE IF NOT EXISTS moves (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id TEXT NOT NULL REFERENCES games(id),
            ply INTEGER NOT NULL,
            uci TEXT NOT NULL,
            san TEXT,
            fen_after TEXT NOT NULL,
            bot_eval INTEGER,
            stockfish_eval INTEGER,
            time_ms INTEGER,
            UNIQUE(game_id, ply)
        );

        CREATE INDEX IF NOT EXISTS idx_games_match ON games(match_id);
        CREATE INDEX IF NOT EXISTS idx_moves_game ON moves(game_id);

        CREATE TABLE IF NOT EXISTS elo_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bot_name TEXT NOT NULL REFERENCES bots(name),
            elo_rating INTEGER NOT NULL,
            recorded_at TEXT NOT NULL,
            match_id TEXT REFERENCES matches(id)
        );

        CREATE INDEX IF NOT EXISTS idx_elo_history_bot ON elo_history(bot_name);
        ",
    )?;

    Ok(Arc::new(Mutex::new(conn)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db_creates_tables() {
        let db = init_db(":memory:").expect("Failed to init db");
        let conn = db.lock().unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"bots".to_string()));
        assert!(tables.contains(&"matches".to_string()));
        assert!(tables.contains(&"games".to_string()));
        assert!(tables.contains(&"moves".to_string()));
        assert!(tables.contains(&"elo_history".to_string()));
    }

    #[test]
    fn test_init_db_creates_indexes() {
        let db = init_db(":memory:").expect("Failed to init db");
        let conn = db.lock().unwrap();

        // Verify indexes exist
        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(indexes.contains(&"idx_games_match".to_string()));
        assert!(indexes.contains(&"idx_moves_game".to_string()));
        assert!(indexes.contains(&"idx_elo_history_bot".to_string()));
    }

    #[test]
    fn test_init_db_idempotent() {
        // Should be safe to call init_db multiple times on same database
        let db = init_db(":memory:").expect("Failed to init db");
        let conn = db.lock().unwrap();

        // Run the schema again - should not fail due to IF NOT EXISTS
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS bots (
                name TEXT PRIMARY KEY,
                elo_rating INTEGER DEFAULT 1500,
                games_played INTEGER DEFAULT 0,
                wins INTEGER DEFAULT 0,
                losses INTEGER DEFAULT 0,
                draws INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            ",
        )
        .expect("Schema should be idempotent");
    }

    #[test]
    fn test_bots_table_defaults() {
        let db = init_db(":memory:").expect("Failed to init db");
        let conn = db.lock().unwrap();

        // Insert a bot with only the name
        conn.execute("INSERT INTO bots (name) VALUES (?)", ["test_bot"])
            .expect("Failed to insert bot");

        // Verify defaults are applied
        let (elo, games_played, wins, losses, draws): (i32, i32, i32, i32, i32) = conn
            .query_row(
                "SELECT elo_rating, games_played, wins, losses, draws FROM bots WHERE name = ?",
                ["test_bot"],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .expect("Failed to query bot");

        assert_eq!(elo, 1500);
        assert_eq!(games_played, 0);
        assert_eq!(wins, 0);
        assert_eq!(losses, 0);
        assert_eq!(draws, 0);
    }

    #[test]
    fn test_moves_unique_constraint() {
        let db = init_db(":memory:").expect("Failed to init db");
        let conn = db.lock().unwrap();

        // Set up test data
        conn.execute("INSERT INTO bots (name) VALUES (?)", ["white_bot"])
            .unwrap();
        conn.execute("INSERT INTO bots (name) VALUES (?)", ["black_bot"])
            .unwrap();
        conn.execute(
            "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at) VALUES (?, ?, ?, ?, ?)",
            ["match1", "white_bot", "black_bot", "1", "2025-01-21"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO games (id, match_id, game_number, started_at) VALUES (?, ?, ?, ?)",
            ["game1", "match1", "1", "2025-01-21"],
        )
        .unwrap();

        // Insert first move
        conn.execute(
            "INSERT INTO moves (game_id, ply, uci, fen_after) VALUES (?, ?, ?, ?)",
            [
                "game1",
                "1",
                "e2e4",
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            ],
        )
        .expect("First move should insert");

        // Try to insert duplicate move (same game_id and ply)
        let result = conn.execute(
            "INSERT INTO moves (game_id, ply, uci, fen_after) VALUES (?, ?, ?, ?)",
            ["game1", "1", "d2d4", "different_fen"],
        );

        assert!(result.is_err(), "Duplicate game_id/ply should fail");
    }

    #[test]
    fn test_foreign_key_enforcement() {
        let db = init_db(":memory:").expect("Failed to init db");
        let conn = db.lock().unwrap();

        // Try to insert a game with non-existent match_id - should fail
        let result = conn.execute(
            "INSERT INTO games (id, match_id, game_number, started_at) VALUES ('g1', 'nonexistent', 1, '2025-01-01')",
            [],
        );

        assert!(
            result.is_err(),
            "Foreign key constraint should prevent orphaned game"
        );
    }
}
