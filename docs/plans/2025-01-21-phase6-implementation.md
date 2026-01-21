# Phase 6: Match Execution, Analysis & Features - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the bot arena fully functional from the web UI with match execution, Stockfish analysis, and enhanced viewing features.

**Architecture:** Worker process polls SQLite for pending matches, executes them via UCI, writes moves in real-time. Server watches for changes and broadcasts via WebSocket. Analysis runs on-demand via Stockfish pool.

**Tech Stack:** Rust (Axum, rusqlite, tokio), SvelteKit 5, UCI protocol, Stockfish

---

## Phase 6A: Match Execution

### Task 1: Add status columns to matches table

**Files:**
- Modify: `crates/bot-arena-server/src/db.rs`

**Step 1: Add migration for status column**

In `db.rs`, update the `matches` table creation:

```rust
CREATE TABLE IF NOT EXISTS matches (
    id TEXT PRIMARY KEY,
    white_bot TEXT NOT NULL REFERENCES bots(name),
    black_bot TEXT NOT NULL REFERENCES bots(name),
    games_total INTEGER NOT NULL,
    white_score REAL DEFAULT 0,
    black_score REAL DEFAULT 0,
    opening_id TEXT,
    movetime_ms INTEGER DEFAULT 1000,
    status TEXT DEFAULT 'pending',
    worker_id TEXT,
    started_at TEXT,
    finished_at TEXT
)
```

**Step 2: Update models**

Modify `crates/bot-arena-server/src/models.rs` to include status field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub white_bot: String,
    pub black_bot: String,
    pub games_total: i32,
    pub white_score: f64,
    pub black_score: f64,
    pub opening_id: Option<String>,
    pub movetime_ms: i32,
    pub status: String,
    pub worker_id: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}
```

**Step 3: Update repository queries**

Modify `crates/bot-arena-server/src/repo/matches.rs` to select/map status and worker_id.

**Step 4: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/bot-arena-server/src/
git commit -m "feat(bot-arena-server): add status and worker_id columns to matches"
```

---

### Task 2: Create bot-arena-worker crate scaffold

**Files:**
- Create: `crates/bot-arena-worker/Cargo.toml`
- Create: `crates/bot-arena-worker/src/main.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Create Cargo.toml**

```toml
[package]
name = "bot-arena-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.35", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
clap = { version = "4", features = ["derive"] }

# Reuse from workspace
chess-core = { path = "../chess-core" }
chess-engine = { path = "../chess-engine" }
uci = { path = "../uci" }
```

**Step 2: Create main.rs scaffold**

```rust
//! Bot Arena Worker - Executes matches from the database.

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bot-arena-worker")]
#[command(about = "Executes bot matches from the database")]
struct Args {
    /// Path to SQLite database
    #[arg(long, default_value = "data/arena.db")]
    db: PathBuf,

    /// Poll interval in milliseconds
    #[arg(long, default_value = "1000")]
    poll_interval: u64,

    /// Directory containing bot executables
    #[arg(long, default_value = "bots")]
    bots_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    tracing::info!("Starting bot-arena-worker");
    tracing::info!("Database: {:?}", args.db);
    tracing::info!("Poll interval: {}ms", args.poll_interval);

    // TODO: Implement worker loop
    Ok(())
}
```

**Step 3: Add to workspace**

In root `Cargo.toml`, add to members:
```toml
members = [
    # ... existing
    "crates/bot-arena-worker",
]
```

**Step 4: Verify it builds**

Run: `cargo build -p bot-arena-worker`
Expected: Builds successfully

**Step 5: Commit**

```bash
git add Cargo.toml crates/bot-arena-worker/
git commit -m "feat(bot-arena-worker): create crate scaffold"
```

---

### Task 3: Add match claiming logic with atomic updates

**Files:**
- Create: `crates/bot-arena-worker/src/db.rs`
- Modify: `crates/bot-arena-worker/src/main.rs`

**Step 1: Create db.rs with claim function**

```rust
//! Database operations for the worker.

use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn connect(path: &Path) -> SqliteResult<DbPool> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(Arc::new(Mutex::new(conn)))
}

#[derive(Debug, Clone)]
pub struct PendingMatch {
    pub id: String,
    pub white_bot: String,
    pub black_bot: String,
    pub games_total: i32,
    pub movetime_ms: i32,
    pub opening_id: Option<String>,
}

/// Atomically claim a pending match.
/// Returns None if no pending matches or claim failed.
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
             LIMIT 1"
        )?;

        let match_opt = stmt.query_row([], |row| {
            Ok(PendingMatch {
                id: row.get(0)?,
                white_bot: row.get(1)?,
                black_bot: row.get(2)?,
                games_total: row.get(3)?,
                movetime_ms: row.get(4)?,
                opening_id: row.get(5)?,
            })
        }).optional()?;

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
    use rusqlite::Connection;

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
             VALUES ('match1', 'bot1', 'bot2', 10);"
        ).unwrap();
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
        let status: String = conn.query_row(
            "SELECT status FROM matches WHERE id = 'match1'",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(status, "running");
    }
}
```

**Step 2: Update main.rs to use db module**

```rust
mod db;

// In main():
let db = db::connect(&args.db)?;
let worker_id = uuid::Uuid::new_v4().to_string();
tracing::info!("Worker ID: {}", worker_id);
```

**Step 3: Run tests**

Run: `cargo test -p bot-arena-worker`
Expected: 3 tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena-worker/
git commit -m "feat(bot-arena-worker): add match claiming with atomic updates"
```

---

### Task 4: Integrate game runner from bot-arena

**Files:**
- Create: `crates/bot-arena-worker/src/runner.rs`
- Modify: `crates/bot-arena-worker/Cargo.toml`
- Modify: `crates/bot-arena-worker/src/main.rs`

**Step 1: Add bot-arena dependency**

In `Cargo.toml`:
```toml
[dependencies]
bot-arena = { path = "../bot-arena" }
```

**Step 2: Create runner.rs**

```rust
//! Match runner - executes games between bots.

use crate::db::{DbPool, PendingMatch};
use bot_arena::{BotConfig, GameRunner, GameResult};
use std::path::Path;

pub struct MatchRunner {
    bots_dir: std::path::PathBuf,
}

impl MatchRunner {
    pub fn new(bots_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            bots_dir: bots_dir.into(),
        }
    }

    /// Run a single match, returning results for each game.
    pub async fn run_match(
        &self,
        pending: &PendingMatch,
        on_move: impl Fn(&str, i32, &str), // (game_id, ply, uci)
        on_game_end: impl Fn(&str, &str),   // (game_id, result)
    ) -> anyhow::Result<Vec<GameResult>> {
        let white_path = self.bots_dir.join(&pending.white_bot);
        let black_path = self.bots_dir.join(&pending.black_bot);

        let white = BotConfig {
            name: pending.white_bot.clone(),
            command: white_path.to_string_lossy().to_string(),
            args: vec![],
        };

        let black = BotConfig {
            name: pending.black_bot.clone(),
            command: black_path.to_string_lossy().to_string(),
            args: vec![],
        };

        let mut results = Vec::new();

        for game_num in 0..pending.games_total {
            let game_id = format!("{}-{}", pending.id, game_num);

            // Alternate colors each game
            let (w, b) = if game_num % 2 == 0 {
                (&white, &black)
            } else {
                (&black, &white)
            };

            let runner = GameRunner::new(w.clone(), b.clone(), pending.movetime_ms as u64);

            let result = runner.run_game(|ply, uci| {
                on_move(&game_id, ply as i32, uci);
            }).await?;

            on_game_end(&game_id, &result.to_string());
            results.push(result);
        }

        Ok(results)
    }
}
```

**Step 3: Wire into main.rs**

```rust
mod runner;

use runner::MatchRunner;

// In main loop:
let runner = MatchRunner::new(&args.bots_dir);
```

**Step 4: Verify it builds**

Run: `cargo build -p bot-arena-worker`
Expected: Builds (may need to adjust bot-arena API)

**Step 5: Commit**

```bash
git add crates/bot-arena-worker/
git commit -m "feat(bot-arena-worker): integrate game runner"
```

---

### Task 5: Write moves to database in real-time

**Files:**
- Modify: `crates/bot-arena-worker/src/db.rs`
- Modify: `crates/bot-arena-worker/src/main.rs`

**Step 1: Add move insertion function**

In `db.rs`:

```rust
/// Insert a move into the database.
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

/// Create a game record.
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

/// Update game result.
pub fn finish_game(
    db: &DbPool,
    game_id: &str,
    result: &str,
) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE games SET result = ?1 WHERE id = ?2",
        (result, game_id),
    )?;
    Ok(())
}

#[test]
fn test_insert_move() {
    let db = setup_test_db();
    // Add games table
    {
        let conn = db.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE games (id TEXT PRIMARY KEY, match_id TEXT, game_number INTEGER, result TEXT);
             CREATE TABLE moves (game_id TEXT, ply INTEGER, uci TEXT, san TEXT, fen_after TEXT);
             INSERT INTO games (id, match_id, game_number) VALUES ('g1', 'match1', 1);"
        ).unwrap();
    }

    insert_move(&db, "g1", 1, "e2e4", Some("e4"), "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();

    let conn = db.lock().unwrap();
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM moves", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1);
}
```

**Step 2: Update main.rs worker loop**

```rust
// Main worker loop
loop {
    if let Some(pending) = db::claim_match(&db, &worker_id)? {
        tracing::info!("Claimed match: {}", pending.id);

        let db_clone = db.clone();
        let match_id = pending.id.clone();

        for game_num in 0..pending.games_total {
            let game_id = format!("{}-{}", match_id, game_num);
            db::create_game(&db, &game_id, &match_id, game_num)?;
        }

        let results = runner.run_match(
            &pending,
            |game_id, ply, uci| {
                // TODO: Get FEN from game state
                let _ = db::insert_move(&db_clone, game_id, ply, uci, None, "");
            },
            |game_id, result| {
                let _ = db::finish_game(&db_clone, game_id, result);
            },
        ).await?;

        db::finish_match(&db, &match_id, &results)?;
    } else {
        tokio::time::sleep(std::time::Duration::from_millis(args.poll_interval)).await;
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p bot-arena-worker`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena-worker/
git commit -m "feat(bot-arena-worker): write moves to database in real-time"
```

---

### Task 6: Add server-side move watcher + WebSocket broadcast

**Files:**
- Modify: `crates/bot-arena-server/src/main.rs`
- Create: `crates/bot-arena-server/src/watcher.rs`

**Step 1: Create watcher.rs**

```rust
//! Watches database for new moves and broadcasts via WebSocket.

use crate::db::DbPool;
use crate::ws::{WsBroadcast, WsMessage};
use std::collections::HashMap;
use tokio::time::{interval, Duration};

pub async fn watch_moves(db: DbPool, broadcast: WsBroadcast) {
    let mut last_move_ids: HashMap<String, i32> = HashMap::new();
    let mut ticker = interval(Duration::from_millis(100));

    loop {
        ticker.tick().await;

        let new_moves = {
            let conn = db.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT m.game_id, m.ply, m.uci, g.match_id
                 FROM moves m
                 JOIN games g ON m.game_id = g.id
                 ORDER BY m.rowid DESC
                 LIMIT 100"
            ).unwrap();

            let moves: Vec<(String, i32, String, String)> = stmt
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            moves
        };

        for (game_id, ply, uci, match_id) in new_moves {
            let last_ply = last_move_ids.get(&game_id).copied().unwrap_or(-1);
            if ply > last_ply {
                last_move_ids.insert(game_id.clone(), ply);

                let _ = broadcast.send(WsMessage::Move {
                    match_id,
                    uci,
                    centipawns: None,
                });
            }
        }
    }
}
```

**Step 2: Spawn watcher in main.rs**

```rust
mod watcher;

// In main(), after creating AppState:
let db_for_watcher = state.db.clone();
let broadcast_for_watcher = state.ws_broadcast.clone();
tokio::spawn(async move {
    watcher::watch_moves(db_for_watcher, broadcast_for_watcher).await;
});
```

**Step 3: Verify it builds**

Run: `cargo build -p bot-arena-server`
Expected: Builds successfully

**Step 4: Commit**

```bash
git add crates/bot-arena-server/
git commit -m "feat(bot-arena-server): add move watcher for WebSocket broadcast"
```

---

### Task 7: Update Elo after match completion

**Files:**
- Modify: `crates/bot-arena-worker/src/db.rs`

**Step 1: Add finish_match function with Elo updates**

```rust
use crate::elo;

/// Finish a match, update scores and Elo ratings.
pub fn finish_match(
    db: &DbPool,
    match_id: &str,
    results: &[GameResult],
) -> SqliteResult<()> {
    let conn = db.lock().unwrap();

    // Calculate scores
    let mut white_score = 0.0;
    let mut black_score = 0.0;

    for (i, result) in results.iter().enumerate() {
        let (w, b) = match result {
            GameResult::WhiteWins => (1.0, 0.0),
            GameResult::BlackWins => (0.0, 1.0),
            GameResult::Draw => (0.5, 0.5),
        };

        // Colors alternate each game
        if i % 2 == 0 {
            white_score += w;
            black_score += b;
        } else {
            white_score += b;
            black_score += w;
        }
    }

    // Get bot ratings
    let (white_bot, black_bot): (String, String) = conn.query_row(
        "SELECT white_bot, black_bot FROM matches WHERE id = ?1",
        [match_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

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

    // Update Elo for each game result
    let mut new_white_rating = white_rating;
    let mut new_black_rating = black_rating;

    for (i, result) in results.iter().enumerate() {
        let (w_actual, b_actual) = match result {
            GameResult::WhiteWins => (1.0, 0.0),
            GameResult::BlackWins => (0.0, 1.0),
            GameResult::Draw => (0.5, 0.5),
        };

        // Colors alternate
        if i % 2 == 0 {
            new_white_rating = elo::new_rating(new_white_rating, new_black_rating, w_actual);
            new_black_rating = elo::new_rating(new_black_rating, new_white_rating, b_actual);
        } else {
            new_black_rating = elo::new_rating(new_black_rating, new_white_rating, w_actual);
            new_white_rating = elo::new_rating(new_white_rating, new_black_rating, b_actual);
        }
    }

    // Update database
    conn.execute(
        "UPDATE bots SET elo_rating = ?1, games_played = games_played + ?2 WHERE name = ?3",
        (new_white_rating, results.len(), &white_bot),
    )?;

    conn.execute(
        "UPDATE bots SET elo_rating = ?1, games_played = games_played + ?2 WHERE name = ?3",
        (new_black_rating, results.len(), &black_bot),
    )?;

    conn.execute(
        "UPDATE matches SET status = 'completed', white_score = ?1, black_score = ?2, finished_at = datetime('now')
         WHERE id = ?3",
        (white_score, black_score, match_id),
    )?;

    Ok(())
}
```

**Step 2: Add elo module reference**

The worker needs access to Elo calculations. Either:
- Copy elo.rs from bot-arena-server, or
- Extract to shared crate

For simplicity, copy the module:

```rust
// crates/bot-arena-worker/src/elo.rs
pub fn new_rating(rating: i32, opponent_rating: i32, actual: f64) -> i32 {
    let expected = 1.0 / (1.0 + 10_f64.powf((opponent_rating - rating) as f64 / 400.0));
    let new = rating as f64 + 32.0 * (actual - expected);
    new.round() as i32
}
```

**Step 3: Run tests**

Run: `cargo test -p bot-arena-worker`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena-worker/
git commit -m "feat(bot-arena-worker): update Elo ratings after match completion"
```

---

### Task 8: Add worker graceful shutdown

**Files:**
- Modify: `crates/bot-arena-worker/src/main.rs`
- Modify: `crates/bot-arena-worker/src/db.rs`

**Step 1: Add release_match function**

In `db.rs`:

```rust
/// Release a match back to pending (e.g., on shutdown).
pub fn release_match(db: &DbPool, match_id: &str, worker_id: &str) -> SqliteResult<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE matches SET status = 'pending', worker_id = NULL
         WHERE id = ?1 AND worker_id = ?2 AND status = 'running'",
        (match_id, worker_id),
    )?;
    Ok(())
}
```

**Step 2: Add signal handling in main.rs**

```rust
use tokio::signal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ... setup ...

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        tracing::info!("Shutdown signal received");
        shutdown_clone.store(true, Ordering::SeqCst);
    });

    let mut current_match: Option<String> = None;

    loop {
        if shutdown.load(Ordering::SeqCst) {
            if let Some(ref match_id) = current_match {
                tracing::info!("Releasing match {} due to shutdown", match_id);
                db::release_match(&db, match_id, &worker_id)?;
            }
            break;
        }

        if let Some(pending) = db::claim_match(&db, &worker_id)? {
            current_match = Some(pending.id.clone());
            // ... run match ...
            current_match = None;
        } else {
            tokio::time::sleep(Duration::from_millis(args.poll_interval)).await;
        }
    }

    tracing::info!("Worker shutdown complete");
    Ok(())
}
```

**Step 3: Run tests**

Run: `cargo test -p bot-arena-worker`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena-worker/
git commit -m "feat(bot-arena-worker): add graceful shutdown with match release"
```

---

## Phase 6B: Stockfish Analysis

### Task 9: Add Stockfish engine pool to server

**Files:**
- Create: `crates/bot-arena-server/src/analysis.rs`
- Modify: `crates/bot-arena-server/src/main.rs`
- Modify: `crates/bot-arena-server/Cargo.toml`

**Step 1: Create analysis.rs**

```rust
//! Stockfish analysis pool.

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, Semaphore};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub depth: i32,
    pub score_cp: Option<i32>,
    pub score_mate: Option<i32>,
    pub best_move: String,
    pub pv: Vec<String>,
}

pub struct EnginePool {
    semaphore: Arc<Semaphore>,
    stockfish_path: String,
}

impl EnginePool {
    pub fn new(stockfish_path: String, pool_size: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(pool_size)),
            stockfish_path,
        }
    }

    pub async fn analyze(&self, fen: &str, depth: i32) -> anyhow::Result<AnalysisResult> {
        let _permit = self.semaphore.acquire().await?;

        let mut child = Command::new(&self.stockfish_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout).lines();

        // Send commands
        stdin.write_all(b"uci\n").await?;
        stdin.write_all(format!("position fen {}\n", fen).as_bytes()).await?;
        stdin.write_all(format!("go depth {}\n", depth).as_bytes()).await?;

        let mut result = AnalysisResult {
            depth: 0,
            score_cp: None,
            score_mate: None,
            best_move: String::new(),
            pv: Vec::new(),
        };

        // Parse output
        while let Some(line) = reader.next_line().await? {
            if line.starts_with("info depth") {
                // Parse info line
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    match *part {
                        "depth" => result.depth = parts.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0),
                        "cp" => result.score_cp = parts.get(i + 1).and_then(|s| s.parse().ok()),
                        "mate" => result.score_mate = parts.get(i + 1).and_then(|s| s.parse().ok()),
                        "pv" => result.pv = parts[i + 1..].iter().map(|s| s.to_string()).collect(),
                        _ => {}
                    }
                }
            } else if line.starts_with("bestmove") {
                result.best_move = line.split_whitespace().nth(1).unwrap_or("").to_string();
                break;
            }
        }

        stdin.write_all(b"quit\n").await?;
        child.wait().await?;

        Ok(result)
    }
}
```

**Step 2: Add to AppState**

In `main.rs`:

```rust
mod analysis;

pub struct AppState {
    pub db: DbPool,
    pub ws_broadcast: ws::WsBroadcast,
    pub engine_pool: Option<analysis::EnginePool>,
}

// In main():
let engine_pool = std::env::var("STOCKFISH_PATH").ok().map(|path| {
    analysis::EnginePool::new(path, 2)
});
```

**Step 3: Verify it builds**

Run: `cargo build -p bot-arena-server`
Expected: Builds successfully

**Step 4: Commit**

```bash
git add crates/bot-arena-server/
git commit -m "feat(bot-arena-server): add Stockfish engine pool"
```

---

### Task 10: Create analysis API endpoint

**Files:**
- Create: `crates/bot-arena-server/src/api/analysis.rs`
- Modify: `crates/bot-arena-server/src/api/mod.rs`
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Create analysis.rs endpoint**

```rust
//! Analysis API endpoints.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AnalysisQuery {
    pub fen: String,
    #[serde(default = "default_depth")]
    pub depth: i32,
}

fn default_depth() -> i32 {
    20
}

#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    pub fen: String,
    pub depth: i32,
    pub score_cp: Option<i32>,
    pub score_mate: Option<i32>,
    pub best_move: String,
    pub pv: Vec<String>,
}

/// GET /api/analysis?fen=...&depth=20
pub async fn get_analysis(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalysisQuery>,
) -> Result<Json<AnalysisResponse>, (StatusCode, String)> {
    let pool = state.engine_pool.as_ref().ok_or_else(|| {
        (StatusCode::SERVICE_UNAVAILABLE, "Stockfish not configured".to_string())
    })?;

    let result = pool
        .analyze(&query.fen, query.depth)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AnalysisResponse {
        fen: query.fen,
        depth: result.depth,
        score_cp: result.score_cp,
        score_mate: result.score_mate,
        best_move: result.best_move,
        pv: result.pv,
    }))
}
```

**Step 2: Add route**

In `main.rs`:

```rust
.route("/api/analysis", get(api::analysis::get_analysis))
```

**Step 3: Verify it builds**

Run: `cargo build -p bot-arena-server`
Expected: Builds successfully

**Step 4: Commit**

```bash
git add crates/bot-arena-server/
git commit -m "feat(bot-arena-server): add analysis API endpoint"
```

---

### Task 11: Add analysis UI to game detail page

**Files:**
- Modify: `apps/web/bot-arena-ui/src/lib/api.ts`
- Modify: `apps/web/bot-arena-ui/src/routes/games/[id]/+page.svelte`

**Step 1: Add analysis API method**

In `api.ts`:

```typescript
export interface AnalysisResult {
  fen: string;
  depth: number;
  score_cp: number | null;
  score_mate: number | null;
  best_move: string;
  pv: string[];
}

async getAnalysis(fen: string, depth: number = 20): Promise<AnalysisResult> {
  const params = new URLSearchParams({ fen, depth: depth.toString() });
  return this.fetchJson(`/analysis?${params}`);
}
```

**Step 2: Add analysis to game detail page**

Add analysis state and button to `+page.svelte`:

```svelte
<script lang="ts">
  // ... existing imports ...
  import type { AnalysisResult } from '$lib/types';

  let analysis: AnalysisResult | null = $state(null);
  let analyzing = $state(false);

  async function analyzePosition() {
    if (!currentFen) return;
    analyzing = true;
    try {
      analysis = await api.getAnalysis(currentFen);
    } catch (e) {
      console.error('Analysis failed:', e);
    } finally {
      analyzing = false;
    }
  }

  // Clear analysis when moving to different position
  $effect(() => {
    currentFen;
    analysis = null;
  });
</script>

<!-- In template, after board -->
<div class="analysis-panel">
  <button onclick={analyzePosition} disabled={analyzing}>
    {analyzing ? 'Analyzing...' : 'Analyze Position'}
  </button>

  {#if analysis}
    <div class="analysis-result">
      <div class="eval">
        {#if analysis.score_mate !== null}
          Mate in {analysis.score_mate}
        {:else if analysis.score_cp !== null}
          {(analysis.score_cp / 100).toFixed(2)}
        {/if}
      </div>
      <div class="best-move">Best: {analysis.best_move}</div>
      <div class="pv">PV: {analysis.pv.slice(0, 5).join(' ')}</div>
    </div>
  {/if}
</div>
```

**Step 3: Verify it builds**

Run: `cd apps/web/bot-arena-ui && pnpm check && pnpm build`
Expected: Builds successfully

**Step 4: Commit**

```bash
git add apps/web/bot-arena-ui/
git commit -m "feat(bot-arena-ui): add position analysis to game detail"
```

---

### Task 12: Add eval bar visualization

**Files:**
- Create: `apps/web/bot-arena-ui/src/lib/components/EvalBar.svelte`
- Modify: `apps/web/bot-arena-ui/src/routes/games/[id]/+page.svelte`

**Step 1: Create EvalBar component**

```svelte
<script lang="ts">
  interface Props {
    scoreCp: number | null;
    scoreMate: number | null;
  }

  let { scoreCp, scoreMate }: Props = $props();

  // Convert score to percentage (0-100, 50 = equal)
  const percentage = $derived(() => {
    if (scoreMate !== null) {
      return scoreMate > 0 ? 100 : 0;
    }
    if (scoreCp === null) return 50;

    // Sigmoid-like scaling: ±500cp maps to roughly 10-90%
    const clamped = Math.max(-1000, Math.min(1000, scoreCp));
    return 50 + (clamped / 20);
  });

  const whiteHeight = $derived(`${percentage()}%`);
  const blackHeight = $derived(`${100 - percentage()}%`);
</script>

<div class="eval-bar">
  <div class="black-side" style="height: {blackHeight}"></div>
  <div class="white-side" style="height: {whiteHeight}"></div>
</div>

<style>
  .eval-bar {
    width: 20px;
    height: 100%;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow: hidden;
  }

  .black-side {
    background: #333;
    transition: height 0.3s ease;
  }

  .white-side {
    background: #eee;
    transition: height 0.3s ease;
  }
</style>
```

**Step 2: Use in game detail page**

```svelte
<script>
  import EvalBar from '$lib/components/EvalBar.svelte';
</script>

<div class="board-with-eval">
  {#if analysis}
    <EvalBar scoreCp={analysis.score_cp} scoreMate={analysis.score_mate} />
  {/if}
  <div class="board-container">
    <!-- Board component -->
  </div>
</div>
```

**Step 3: Verify it builds**

Run: `cd apps/web/bot-arena-ui && pnpm check && pnpm build`
Expected: Builds successfully

**Step 4: Commit**

```bash
git add apps/web/bot-arena-ui/
git commit -m "feat(bot-arena-ui): add eval bar visualization"
```

---

## Phase 6C: Enhanced Features

### Task 13: Add opening statistics API

**Files:**
- Create: `crates/bot-arena-server/src/api/openings.rs`
- Modify: `crates/bot-arena-server/src/api/mod.rs`
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Create openings.rs**

```rust
//! Opening statistics API.

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct OpeningStats {
    pub eco: String,
    pub name: String,
    pub games_played: i32,
    pub white_wins: i32,
    pub black_wins: i32,
    pub draws: i32,
}

/// GET /api/openings
pub async fn list_openings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<OpeningStats>>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT
                g.opening_name as name,
                COUNT(*) as games,
                SUM(CASE WHEN g.result = '1-0' THEN 1 ELSE 0 END) as white_wins,
                SUM(CASE WHEN g.result = '0-1' THEN 1 ELSE 0 END) as black_wins,
                SUM(CASE WHEN g.result = '1/2-1/2' THEN 1 ELSE 0 END) as draws
             FROM games g
             WHERE g.opening_name IS NOT NULL
             GROUP BY g.opening_name
             ORDER BY games DESC",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let openings = stmt
        .query_map([], |row| {
            Ok(OpeningStats {
                eco: String::new(), // TODO: lookup ECO
                name: row.get(0)?,
                games_played: row.get(1)?,
                white_wins: row.get(2)?,
                black_wins: row.get(3)?,
                draws: row.get(4)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(openings))
}
```

**Step 2: Add route**

```rust
.route("/api/openings", get(api::openings::list_openings))
```

**Step 3: Commit**

```bash
git add crates/bot-arena-server/
git commit -m "feat(bot-arena-server): add opening statistics API"
```

---

### Task 14: Create opening explorer page

**Files:**
- Create: `apps/web/bot-arena-ui/src/routes/openings/+page.svelte`
- Modify: `apps/web/bot-arena-ui/src/lib/api.ts`
- Modify: `apps/web/bot-arena-ui/src/routes/+layout.svelte`

**Step 1: Add API method**

```typescript
export interface OpeningStats {
  eco: string;
  name: string;
  games_played: number;
  white_wins: number;
  black_wins: number;
  draws: number;
}

async getOpenings(): Promise<OpeningStats[]> {
  return this.fetchJson('/openings');
}
```

**Step 2: Create openings page**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { OpeningStats } from '$lib/types';

  let openings: OpeningStats[] = $state([]);
  let loading = $state(true);
  let search = $state('');

  onMount(async () => {
    try {
      openings = await api.getOpenings();
    } finally {
      loading = false;
    }
  });

  const filtered = $derived(
    openings.filter(o =>
      o.name.toLowerCase().includes(search.toLowerCase()) ||
      o.eco.toLowerCase().includes(search.toLowerCase())
    )
  );
</script>

<div class="openings-page">
  <h1>Opening Explorer</h1>

  <input
    type="search"
    placeholder="Search openings..."
    bind:value={search}
  />

  {#if loading}
    <p>Loading...</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>ECO</th>
          <th>Name</th>
          <th>Games</th>
          <th>White %</th>
          <th>Draw %</th>
          <th>Black %</th>
        </tr>
      </thead>
      <tbody>
        {#each filtered as opening}
          <tr>
            <td>{opening.eco}</td>
            <td>{opening.name}</td>
            <td>{opening.games_played}</td>
            <td>{((opening.white_wins / opening.games_played) * 100).toFixed(1)}%</td>
            <td>{((opening.draws / opening.games_played) * 100).toFixed(1)}%</td>
            <td>{((opening.black_wins / opening.games_played) * 100).toFixed(1)}%</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  /* Similar table styles as game browser */
</style>
```

**Step 3: Add nav link**

In `+layout.svelte`, add to nav:
```svelte
<a href="/openings">Openings</a>
```

**Step 4: Commit**

```bash
git add apps/web/bot-arena-ui/
git commit -m "feat(bot-arena-ui): add opening explorer page"
```

---

### Task 15: Add head-to-head matrix API

**Files:**
- Create: `crates/bot-arena-server/src/api/stats.rs`
- Modify: `crates/bot-arena-server/src/api/mod.rs`
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Create stats.rs**

```rust
//! Statistics API endpoints.

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct HeadToHeadRecord {
    pub white_bot: String,
    pub black_bot: String,
    pub white_wins: i32,
    pub black_wins: i32,
    pub draws: i32,
    pub games: i32,
}

#[derive(Debug, Serialize)]
pub struct HeadToHeadMatrix {
    pub bots: Vec<String>,
    pub records: Vec<HeadToHeadRecord>,
}

/// GET /api/stats/head-to-head
pub async fn head_to_head(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HeadToHeadMatrix>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();

    // Get all bot names
    let mut stmt = conn
        .prepare("SELECT name FROM bots ORDER BY elo_rating DESC")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let bots: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    // Get head-to-head records
    let mut stmt = conn
        .prepare(
            "SELECT
                m.white_bot,
                m.black_bot,
                SUM(CASE WHEN g.result = '1-0' THEN 1 ELSE 0 END) as white_wins,
                SUM(CASE WHEN g.result = '0-1' THEN 1 ELSE 0 END) as black_wins,
                SUM(CASE WHEN g.result = '1/2-1/2' THEN 1 ELSE 0 END) as draws,
                COUNT(*) as games
             FROM matches m
             JOIN games g ON g.match_id = m.id
             WHERE m.status = 'completed'
             GROUP BY m.white_bot, m.black_bot",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let records: Vec<HeadToHeadRecord> = stmt
        .query_map([], |row| {
            Ok(HeadToHeadRecord {
                white_bot: row.get(0)?,
                black_bot: row.get(1)?,
                white_wins: row.get(2)?,
                black_wins: row.get(3)?,
                draws: row.get(4)?,
                games: row.get(5)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(HeadToHeadMatrix { bots, records }))
}
```

**Step 2: Add route**

```rust
.route("/api/stats/head-to-head", get(api::stats::head_to_head))
```

**Step 3: Commit**

```bash
git add crates/bot-arena-server/
git commit -m "feat(bot-arena-server): add head-to-head matrix API"
```

---

### Task 16: Create stats page with head-to-head matrix

**Files:**
- Create: `apps/web/bot-arena-ui/src/routes/stats/+page.svelte`
- Modify: `apps/web/bot-arena-ui/src/lib/api.ts`
- Modify: `apps/web/bot-arena-ui/src/routes/+layout.svelte`

**Step 1: Add API method**

```typescript
export interface HeadToHeadRecord {
  white_bot: string;
  black_bot: string;
  white_wins: number;
  black_wins: number;
  draws: number;
  games: number;
}

export interface HeadToHeadMatrix {
  bots: string[];
  records: HeadToHeadRecord[];
}

async getHeadToHead(): Promise<HeadToHeadMatrix> {
  return this.fetchJson('/stats/head-to-head');
}
```

**Step 2: Create stats page**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { HeadToHeadMatrix } from '$lib/types';

  let matrix: HeadToHeadMatrix | null = $state(null);
  let loading = $state(true);

  onMount(async () => {
    try {
      matrix = await api.getHeadToHead();
    } finally {
      loading = false;
    }
  });

  function getRecord(white: string, black: string) {
    if (!matrix) return null;
    return matrix.records.find(
      r => r.white_bot === white && r.black_bot === black
    );
  }

  function formatRecord(white: string, black: string): string {
    const r = getRecord(white, black);
    const r2 = getRecord(black, white);
    if (!r && !r2) return '-';

    let wins = 0, draws = 0, losses = 0;
    if (r) {
      wins += r.white_wins;
      draws += r.draws;
      losses += r.black_wins;
    }
    if (r2) {
      wins += r2.black_wins;
      draws += r2.draws;
      losses += r2.white_wins;
    }
    return `${wins}/${draws}/${losses}`;
  }
</script>

<div class="stats-page">
  <h1>Head-to-Head Matrix</h1>

  {#if loading}
    <p>Loading...</p>
  {:else if matrix}
    <div class="matrix-container">
      <table class="matrix">
        <thead>
          <tr>
            <th></th>
            {#each matrix.bots as bot}
              <th class="bot-header">{bot}</th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each matrix.bots as rowBot}
            <tr>
              <th>{rowBot}</th>
              {#each matrix.bots as colBot}
                <td class:self={rowBot === colBot}>
                  {rowBot === colBot ? '-' : formatRecord(rowBot, colBot)}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <p class="legend">Format: Wins/Draws/Losses (as white + as black combined)</p>
  {/if}
</div>

<style>
  .matrix-container {
    overflow-x: auto;
  }

  .matrix {
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  .matrix th, .matrix td {
    padding: 0.5rem;
    border: 1px solid var(--border);
    text-align: center;
    min-width: 80px;
  }

  .bot-header {
    writing-mode: vertical-rl;
    transform: rotate(180deg);
    height: 100px;
  }

  .self {
    background: var(--bg-secondary);
  }

  .legend {
    color: var(--text-muted);
    font-size: 0.875rem;
    margin-top: 1rem;
  }
</style>
```

**Step 3: Add nav link**

```svelte
<a href="/stats">Stats</a>
```

**Step 4: Commit**

```bash
git add apps/web/bot-arena-ui/
git commit -m "feat(bot-arena-ui): add head-to-head matrix page"
```

---

### Task 17: Add bot profile API with Elo history

**Files:**
- Modify: `crates/bot-arena-server/src/api/bots.rs`
- Modify: `crates/bot-arena-server/src/db.rs`

**Step 1: Add elo_history table to schema**

In `db.rs`:

```sql
CREATE TABLE IF NOT EXISTS elo_history (
    bot_name TEXT NOT NULL REFERENCES bots(name),
    elo_rating INTEGER NOT NULL,
    recorded_at TEXT NOT NULL,
    match_id TEXT REFERENCES matches(id)
)
```

**Step 2: Add bot profile endpoint**

In `bots.rs`:

```rust
#[derive(Debug, Serialize)]
pub struct BotProfile {
    pub name: String,
    pub elo_rating: i32,
    pub games_played: i32,
    pub wins: i32,
    pub draws: i32,
    pub losses: i32,
    pub elo_history: Vec<EloHistoryPoint>,
    pub recent_matches: Vec<MatchSummary>,
}

#[derive(Debug, Serialize)]
pub struct EloHistoryPoint {
    pub elo: i32,
    pub timestamp: String,
}

/// GET /api/bots/:name
pub async fn get_bot(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<BotProfile>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();

    // Get bot info
    let bot: BotProfile = conn
        .query_row(
            "SELECT name, elo_rating, games_played, wins, draws, losses
             FROM bots WHERE name = ?1",
            [&name],
            |row| {
                Ok(BotProfile {
                    name: row.get(0)?,
                    elo_rating: row.get(1)?,
                    games_played: row.get(2)?,
                    wins: row.get(3)?,
                    draws: row.get(4)?,
                    losses: row.get(5)?,
                    elo_history: Vec::new(),
                    recent_matches: Vec::new(),
                })
            },
        )
        .map_err(|_| (StatusCode::NOT_FOUND, "Bot not found".to_string()))?;

    // Get Elo history
    let mut stmt = conn
        .prepare(
            "SELECT elo_rating, recorded_at FROM elo_history
             WHERE bot_name = ?1 ORDER BY recorded_at ASC",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let history: Vec<EloHistoryPoint> = stmt
        .query_map([&name], |row| {
            Ok(EloHistoryPoint {
                elo: row.get(0)?,
                timestamp: row.get(1)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(BotProfile {
        elo_history: history,
        ..bot
    }))
}
```

**Step 3: Add route**

```rust
.route("/api/bots/:name", get(api::bots::get_bot))
```

**Step 4: Commit**

```bash
git add crates/bot-arena-server/
git commit -m "feat(bot-arena-server): add bot profile API with Elo history"
```

---

### Task 18: Create bot profile page

**Files:**
- Create: `apps/web/bot-arena-ui/src/routes/bots/[name]/+page.svelte`
- Modify: `apps/web/bot-arena-ui/src/lib/api.ts`

**Step 1: Add API method**

```typescript
export interface BotProfile {
  name: string;
  elo_rating: number;
  games_played: number;
  wins: number;
  draws: number;
  losses: number;
  elo_history: { elo: number; timestamp: string }[];
  recent_matches: MatchSummary[];
}

async getBot(name: string): Promise<BotProfile> {
  return this.fetchJson(`/bots/${encodeURIComponent(name)}`);
}
```

**Step 2: Create bot profile page**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import type { BotProfile } from '$lib/types';

  let profile: BotProfile | null = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  const name = $derived($page.params.name);

  onMount(async () => {
    try {
      profile = await api.getBot(name);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load bot';
    } finally {
      loading = false;
    }
  });

  const winRate = $derived(
    profile && profile.games_played > 0
      ? ((profile.wins / profile.games_played) * 100).toFixed(1)
      : '0'
  );
</script>

<div class="bot-profile">
  {#if loading}
    <p>Loading...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if profile}
    <header>
      <h1>{profile.name}</h1>
      <div class="elo">{profile.elo_rating} Elo</div>
    </header>

    <div class="stats-grid">
      <div class="stat">
        <div class="value">{profile.games_played}</div>
        <div class="label">Games</div>
      </div>
      <div class="stat">
        <div class="value">{profile.wins}</div>
        <div class="label">Wins</div>
      </div>
      <div class="stat">
        <div class="value">{profile.draws}</div>
        <div class="label">Draws</div>
      </div>
      <div class="stat">
        <div class="value">{profile.losses}</div>
        <div class="label">Losses</div>
      </div>
      <div class="stat">
        <div class="value">{winRate}%</div>
        <div class="label">Win Rate</div>
      </div>
    </div>

    {#if profile.elo_history.length > 0}
      <section>
        <h2>Elo History</h2>
        <div class="elo-chart">
          <!-- Simple text-based chart for MVP -->
          {#each profile.elo_history.slice(-20) as point}
            <div class="elo-point" title={point.timestamp}>
              {point.elo}
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  .bot-profile {
    max-width: 800px;
    margin: 0 auto;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 2rem;
  }

  .elo {
    font-size: 1.5rem;
    color: var(--highlight);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .stat {
    text-align: center;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 8px;
  }

  .stat .value {
    font-size: 1.5rem;
    font-weight: bold;
  }

  .stat .label {
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .elo-chart {
    display: flex;
    gap: 0.5rem;
    overflow-x: auto;
    padding: 1rem 0;
  }

  .elo-point {
    padding: 0.25rem 0.5rem;
    background: var(--accent);
    border-radius: 4px;
    font-size: 0.75rem;
  }
</style>
```

**Step 3: Link from dashboard**

In dashboard, make bot names clickable:
```svelte
<a href="/bots/{bot.name}">{bot.name}</a>
```

**Step 4: Commit**

```bash
git add apps/web/bot-arena-ui/
git commit -m "feat(bot-arena-ui): add bot profile page"
```

---

### Task 19: Final integration test

**Step 1: Build everything**

```bash
cargo build --workspace
cd apps/web/bot-arena-ui && pnpm build
```

**Step 2: Run all tests**

```bash
cargo test --workspace
cd apps/web/bot-arena-ui && pnpm test
```

**Step 3: Verify functionality**

1. Start server: `cargo run -p bot-arena-server`
2. Start worker: `cargo run -p bot-arena-worker`
3. Open http://localhost:3000
4. Create a new match
5. Watch it execute in real-time
6. View completed game with analysis
7. Check opening explorer
8. Check head-to-head matrix
9. View bot profiles

**Step 4: Final commit**

```bash
git add .
git commit -m "feat(phase6): complete match execution, analysis, and enhanced features"
```

---

## Summary

| Sub-Phase | Tasks | Description |
|-----------|-------|-------------|
| 6A | 1-8 | Match execution via worker process |
| 6B | 9-12 | On-demand Stockfish analysis |
| 6C | 13-18 | Opening explorer, head-to-head matrix, bot profiles |
| - | 19 | Final integration test |

**Total: 19 tasks**

---

*Last updated: 2025-01-21*
