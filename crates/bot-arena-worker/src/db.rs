//! Database operations for the worker.
//!
//! This module provides database connectivity and match claiming functionality
//! for the worker. The `claim_match` function will be used in the worker loop
//! implementation (next phase).

use crate::elo;
use rusqlite::{Connection, OptionalExtension, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Thread-safe database connection pool.
pub type DbPool = Arc<Mutex<Connection>>;

/// Opens a connection to the SQLite database.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or configured.
pub fn connect(path: &Path) -> SqliteResult<DbPool> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(Arc::new(Mutex::new(conn)))
}

/// A match that is pending execution.
#[derive(Debug, Clone)]
pub struct PendingMatch {
    /// Unique identifier for the match.
    pub id: String,
    /// Name of the bot playing as white.
    pub white_bot: String,
    /// Name of the bot playing as black.
    pub black_bot: String,
    /// Total number of games to play.
    pub games_total: i32,
    /// Time limit per move in milliseconds.
    pub movetime_ms: i32,
    /// Optional opening position identifier.
    /// Note: Currently unused but will be used for opening database functionality.
    #[allow(dead_code)]
    pub opening_id: Option<String>,
}

/// Atomically claim a pending match.
///
/// This function finds the oldest pending match and atomically updates its status
/// to 'running' while recording the worker ID and start time.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `worker_id` - Unique identifier for this worker
///
/// # Returns
///
/// * `Ok(Some(PendingMatch))` - Successfully claimed a match
/// * `Ok(None)` - No pending matches available or claim failed due to race condition
/// * `Err(_)` - Database error
pub fn claim_match(db: &DbPool, worker_id: &str) -> SqliteResult<Option<PendingMatch>> {
    let conn = db.lock().unwrap();

    // Find and claim in one transaction
    conn.execute_batch("BEGIN IMMEDIATE;")?;

    let result: SqliteResult<Option<PendingMatch>> = (|| {
        let mut stmt = conn.prepare(
            "SELECT id, white_bot, black_bot, games_total, movetime_ms, opening_id
             FROM matches
             WHERE status = 'pending'
             ORDER BY rowid ASC
             LIMIT 1",
        )?;

        let match_opt = stmt
            .query_row([], |row| {
                Ok(PendingMatch {
                    id: row.get(0)?,
                    white_bot: row.get(1)?,
                    black_bot: row.get(2)?,
                    games_total: row.get(3)?,
                    movetime_ms: row.get(4)?,
                    opening_id: row.get(5)?,
                })
            })
            .optional()?;

        if let Some(ref m) = match_opt {
            let updated = conn.execute(
                "UPDATE matches SET status = 'running', worker_id = ?, started_at = datetime('now')
                 WHERE id = ? AND status = 'pending'",
                (&worker_id, &m.id),
            )?;

            if updated == 0 {
                // Race condition - another worker claimed it
                return Ok(None);
            }
        }

        Ok(match_opt)
    })();

    match result {
        Ok(m) => {
            conn.execute_batch("COMMIT;")?;
            Ok(m)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(e)
        }
    }
}

/// Create a game record.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `game_id` - Unique identifier for the game
/// * `match_id` - ID of the parent match
/// * `game_number` - Sequential game number within the match (0-indexed)
///
/// # Errors
///
/// Returns an error if the database insert fails.
pub fn create_game(
    db: &DbPool,
    game_id: &str,
    match_id: &str,
    game_number: i32,
) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO games (id, match_id, game_number)
         VALUES (?1, ?2, ?3)",
        (game_id, match_id, game_number),
    )?;
    Ok(())
}

/// Insert a move into the database.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `game_id` - ID of the game this move belongs to
/// * `ply` - Ply number (0-indexed, 0 = white's first move, 1 = black's first move)
/// * `uci` - Move in UCI notation (e.g., "e2e4")
/// * `san` - Optional move in Standard Algebraic Notation
/// * `fen_after` - FEN string representing the position after the move
///
/// # Errors
///
/// Returns an error if the database insert fails.
pub fn insert_move(
    db: &DbPool,
    game_id: &str,
    ply: i32,
    uci: &str,
    san: Option<&str>,
    fen_after: &str,
) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO moves (game_id, ply, uci, san, fen_after)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (game_id, ply, uci, san, fen_after),
    )?;
    Ok(())
}

/// Update game result.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `game_id` - ID of the game to update
/// * `result` - Game result string (e.g., "1-0", "0-1", "1/2-1/2")
///
/// # Errors
///
/// Returns an error if the database update fails.
pub fn finish_game(db: &DbPool, game_id: &str, result: &str) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE games SET result = ?1 WHERE id = ?2",
        (result, game_id),
    )?;
    Ok(())
}

/// Release a match back to pending (e.g., on shutdown).
///
/// Sets the match status back to 'pending' and clears the worker_id,
/// but only if the match is currently claimed by this worker.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `match_id` - ID of the match to release
/// * `worker_id` - Worker ID that must currently own the match
///
/// # Errors
///
/// Returns an error if the database update fails.
pub fn release_match(db: &DbPool, match_id: &str, worker_id: &str) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE matches SET status = 'pending', worker_id = NULL, started_at = NULL
         WHERE id = ?1 AND worker_id = ?2 AND status = 'running'",
        (match_id, worker_id),
    )?;
    Ok(())
}

/// Finish a match with final scores.
///
/// Updates the match status to 'completed' and records the final scores.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `match_id` - ID of the match to finish
/// * `white_score` - Total points scored by white (1.0 per win, 0.5 per draw)
/// * `black_score` - Total points scored by black
///
/// # Errors
///
/// Returns an error if the database update fails.
pub fn finish_match(
    db: &DbPool,
    match_id: &str,
    white_score: f64,
    black_score: f64,
) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE matches SET status = 'completed', white_score = ?1, black_score = ?2, finished_at = datetime('now')
         WHERE id = ?3",
        (white_score, black_score, match_id),
    )?;
    Ok(())
}

/// A game result for Elo calculation.
#[derive(Debug, Clone)]
pub struct GameResult {
    /// Game number within the match (0-indexed).
    pub game_num: i32,
    /// Result string: "1-0", "0-1", or "1/2-1/2".
    pub result: String,
}

/// Update Elo ratings for both bots after a match.
///
/// This function calculates new Elo ratings based on all game results in the match.
/// Colors alternate each game: even-numbered games have white_bot as white,
/// odd-numbered games have white_bot as black.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `match_id` - Match ID to get bot names
/// * `game_results` - Vector of game results
///
/// # Errors
///
/// Returns an error if database queries fail.
pub fn update_elo_ratings(
    db: &DbPool,
    match_id: &str,
    game_results: &[GameResult],
) -> SqliteResult<()> {
    let conn = db.lock().unwrap();

    // Get bot names from match
    let (white_bot, black_bot): (String, String) = conn.query_row(
        "SELECT white_bot, black_bot FROM matches WHERE id = ?1",
        [match_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    // Get current ratings
    let white_rating: i32 = conn.query_row(
        "SELECT elo_rating FROM bots WHERE name = ?1",
        [&white_bot],
        |row| row.get(0),
    )?;

    let black_rating: i32 = conn.query_row(
        "SELECT elo_rating FROM bots WHERE name = ?1",
        [&black_bot],
        |row| row.get(0),
    )?;

    // Update Elo for each game
    let mut new_white_rating = white_rating;
    let mut new_black_rating = black_rating;

    for game in game_results {
        let (white_actual, black_actual) = match game.result.as_str() {
            "1-0" => (1.0, 0.0),
            "0-1" => (0.0, 1.0),
            _ => (0.5, 0.5), // Draw
        };

        // Colors alternate each game
        if game.game_num % 2 == 0 {
            // Even games: white_bot plays white
            let new_w = elo::new_rating(new_white_rating, new_black_rating, white_actual);
            let new_b = elo::new_rating(new_black_rating, new_white_rating, black_actual);
            new_white_rating = new_w;
            new_black_rating = new_b;
        } else {
            // Odd games: white_bot plays black
            let new_w = elo::new_rating(new_white_rating, new_black_rating, black_actual);
            let new_b = elo::new_rating(new_black_rating, new_white_rating, white_actual);
            new_white_rating = new_w;
            new_black_rating = new_b;
        }
    }

    // Update database
    conn.execute(
        "UPDATE bots SET elo_rating = ?1, games_played = games_played + ?2 WHERE name = ?3",
        (new_white_rating, game_results.len(), &white_bot),
    )?;

    conn.execute(
        "UPDATE bots SET elo_rating = ?1, games_played = games_played + ?2 WHERE name = ?3",
        (new_black_rating, game_results.len(), &black_bot),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> DbPool {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE bots (
                 name TEXT PRIMARY KEY,
                 elo_rating INTEGER DEFAULT 1500,
                 games_played INTEGER DEFAULT 0
             );
             CREATE TABLE matches (
                 id TEXT PRIMARY KEY,
                 white_bot TEXT,
                 black_bot TEXT,
                 games_total INTEGER,
                 movetime_ms INTEGER DEFAULT 1000,
                 opening_id TEXT,
                 status TEXT DEFAULT 'pending',
                 worker_id TEXT,
                 started_at TEXT
             );
             INSERT INTO bots (name) VALUES ('bot1'), ('bot2');
             INSERT INTO matches (id, white_bot, black_bot, games_total)
             VALUES ('match1', 'bot1', 'bot2', 10);",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn test_claim_match_success() {
        let db = setup_test_db();
        let result = claim_match(&db, "worker-1").unwrap();
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.id, "match1");
        assert_eq!(m.white_bot, "bot1");
    }

    #[test]
    fn test_claim_match_none_pending() {
        let db = setup_test_db();
        // First claim succeeds
        claim_match(&db, "worker-1").unwrap();
        // Second claim returns None
        let result = claim_match(&db, "worker-2").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_claim_match_sets_status() {
        let db = setup_test_db();
        claim_match(&db, "worker-1").unwrap();

        let conn = db.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM matches WHERE id = 'match1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "running");
    }

    #[test]
    fn test_create_and_finish_game() {
        let db = setup_test_db();
        {
            let conn = db.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE games (id TEXT PRIMARY KEY, match_id TEXT, game_number INTEGER, result TEXT);",
            )
            .unwrap();
        }

        create_game(&db, "g1", "match1", 0).unwrap();
        finish_game(&db, "g1", "1-0").unwrap();

        let conn = db.lock().unwrap();
        let result: String = conn
            .query_row("SELECT result FROM games WHERE id = 'g1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(result, "1-0");
    }

    #[test]
    fn test_insert_move() {
        let db = setup_test_db();
        {
            let conn = db.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE games (id TEXT PRIMARY KEY, match_id TEXT, game_number INTEGER, result TEXT);
                 CREATE TABLE moves (game_id TEXT, ply INTEGER, uci TEXT, san TEXT, fen_after TEXT);
                 INSERT INTO games (id, match_id, game_number) VALUES ('g1', 'match1', 1);",
            )
            .unwrap();
        }

        insert_move(
            &db,
            "g1",
            1,
            "e2e4",
            Some("e4"),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        )
        .unwrap();

        let conn = db.lock().unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM moves", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_finish_match() {
        let db = setup_test_db();
        {
            let conn = db.lock().unwrap();
            conn.execute("ALTER TABLE matches ADD COLUMN white_score REAL", [])
                .unwrap();
            conn.execute("ALTER TABLE matches ADD COLUMN black_score REAL", [])
                .unwrap();
            conn.execute("ALTER TABLE matches ADD COLUMN finished_at TEXT", [])
                .unwrap();
        }

        // First claim the match to set it to 'running'
        claim_match(&db, "worker-1").unwrap();

        // Then finish it
        finish_match(&db, "match1", 3.5, 1.5).unwrap();

        let conn = db.lock().unwrap();
        let (status, white_score, black_score): (String, f64, f64) = conn
            .query_row(
                "SELECT status, white_score, black_score FROM matches WHERE id = 'match1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(status, "completed");
        assert!((white_score - 3.5).abs() < 0.001);
        assert!((black_score - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_update_elo_ratings_single_win() {
        let db = setup_test_db();

        // bot1 (white_bot) wins game 0 (playing as white)
        let results = vec![GameResult {
            game_num: 0,
            result: "1-0".to_string(),
        }];

        update_elo_ratings(&db, "match1", &results).unwrap();

        let conn = db.lock().unwrap();
        let (bot1_elo, bot1_games): (i32, i32) = conn
            .query_row(
                "SELECT elo_rating, games_played FROM bots WHERE name = 'bot1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let (bot2_elo, bot2_games): (i32, i32) = conn
            .query_row(
                "SELECT elo_rating, games_played FROM bots WHERE name = 'bot2'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        // bot1 won as white in game 0, gains 16 points
        assert_eq!(bot1_elo, 1516);
        assert_eq!(bot2_elo, 1484);
        assert_eq!(bot1_games, 1);
        assert_eq!(bot2_games, 1);
    }

    #[test]
    fn test_update_elo_ratings_color_alternation() {
        let db = setup_test_db();

        // Game 0: white_bot (bot1) plays white, wins (1-0)
        // Game 1: white_bot (bot1) plays black, wins (0-1)
        let results = vec![
            GameResult {
                game_num: 0,
                result: "1-0".to_string(),
            },
            GameResult {
                game_num: 1,
                result: "0-1".to_string(),
            },
        ];

        update_elo_ratings(&db, "match1", &results).unwrap();

        let conn = db.lock().unwrap();
        let (bot1_elo, bot1_games): (i32, i32) = conn
            .query_row(
                "SELECT elo_rating, games_played FROM bots WHERE name = 'bot1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let (bot2_elo, bot2_games): (i32, i32) = conn
            .query_row(
                "SELECT elo_rating, games_played FROM bots WHERE name = 'bot2'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        // bot1 won both games, should have gained points
        assert!(bot1_elo > 1516); // More than single win
        assert!(bot2_elo < 1484); // Lost twice
        assert_eq!(bot1_games, 2);
        assert_eq!(bot2_games, 2);
    }

    #[test]
    fn test_update_elo_ratings_draw() {
        let db = setup_test_db();

        let results = vec![GameResult {
            game_num: 0,
            result: "1/2-1/2".to_string(),
        }];

        update_elo_ratings(&db, "match1", &results).unwrap();

        let conn = db.lock().unwrap();
        let bot1_elo: i32 = conn
            .query_row(
                "SELECT elo_rating FROM bots WHERE name = 'bot1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let bot2_elo: i32 = conn
            .query_row(
                "SELECT elo_rating FROM bots WHERE name = 'bot2'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // Draw between equal ratings = no change
        assert_eq!(bot1_elo, 1500);
        assert_eq!(bot2_elo, 1500);
    }

    #[test]
    fn test_update_elo_ratings_empty_results() {
        let db = setup_test_db();

        let results: Vec<GameResult> = vec![];

        update_elo_ratings(&db, "match1", &results).unwrap();

        let conn = db.lock().unwrap();
        let bot1_elo: i32 = conn
            .query_row(
                "SELECT elo_rating FROM bots WHERE name = 'bot1'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // No games = no change
        assert_eq!(bot1_elo, 1500);
    }

    #[test]
    fn test_release_match() {
        let db = setup_test_db();
        // Claim the match first
        claim_match(&db, "worker-1").unwrap();

        // Release it
        release_match(&db, "match1", "worker-1").unwrap();

        // Verify it's pending again
        let conn = db.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM matches WHERE id = 'match1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "pending");
    }

    #[test]
    fn test_release_match_wrong_worker() {
        let db = setup_test_db();
        // Claim the match with worker-1
        claim_match(&db, "worker-1").unwrap();

        // Try to release with wrong worker ID - should not change status
        release_match(&db, "match1", "worker-2").unwrap();

        // Verify it's still running (not released)
        let conn = db.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM matches WHERE id = 'match1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "running");
    }
}
