//! Database operations for the worker.
//!
//! This module provides database connectivity and match claiming functionality
//! for the worker. The `claim_match` function will be used in the worker loop
//! implementation (next phase).

// These items are public API that will be used by the worker loop implementation.
// Suppressing dead_code warnings during incremental development.
#![allow(dead_code)]

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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> DbPool {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE bots (name TEXT PRIMARY KEY, elo_rating INTEGER DEFAULT 1500);
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
}
