# Phase 5: UI MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a web-based UI for the bot arena with real-time match viewing, game browsing, Elo rankings, and match control.

**Architecture:** Rust/Axum backend serving REST API + WebSocket for live updates, SvelteKit frontend using existing `@tmttn-chess/*` packages. SQLite for persistent storage with Elo ratings.

**Tech Stack:** Rust (Axum, tokio, rusqlite), TypeScript (SvelteKit 5, Svelte stores), `@tmttn-chess/board`, `@tmttn-chess/game-store`

---

## Phase A: Server Foundation

### Task 1: Create bot-arena-server crate

**Files:**
- Create: `crates/bot-arena-server/Cargo.toml`
- Create: `crates/bot-arena-server/src/main.rs`
- Modify: `Cargo.toml` (workspace members)

**Step 1: Create Cargo.toml**

```toml
[package]
name = "bot-arena-server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Step 2: Create minimal main.rs with health check**

```rust
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber;

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::init();

    let app = Router::new().route("/health", get(health));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

**Step 3: Add to workspace members in root Cargo.toml**

Add `"crates/bot-arena-server"` to the members array.

**Step 4: Verify it compiles**

Run: `cargo build -p bot-arena-server`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add crates/bot-arena-server/ Cargo.toml
git commit -m "feat(bot-arena-server): add minimal Axum server with health check"
```

---

### Task 2: Add database module with extended schema

**Files:**
- Create: `crates/bot-arena-server/src/db.rs`
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Create db.rs with schema**

```rust
//! Database module for bot arena server.

use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

/// Initialize database with schema
pub fn init_db<P: AsRef<Path>>(path: P) -> SqliteResult<DbPool> {
    let conn = Connection::open(path)?;

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
            status TEXT DEFAULT 'pending'
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
        "
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
    }
}
```

**Step 2: Update main.rs to use db module**

Add `mod db;` and initialize database in main.

**Step 3: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: 1 test passes

**Step 4: Commit**

```bash
git add crates/bot-arena-server/src/
git commit -m "feat(bot-arena-server): add database module with extended schema"
```

---

### Task 3: Add bot repository and API types

**Files:**
- Create: `crates/bot-arena-server/src/models.rs`
- Create: `crates/bot-arena-server/src/repo/mod.rs`
- Create: `crates/bot-arena-server/src/repo/bots.rs`

**Step 1: Create models.rs with API types**

```rust
//! API models for serialization.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    pub name: String,
    pub elo_rating: i32,
    pub games_played: i32,
    pub wins: i32,
    pub losses: i32,
    pub draws: i32,
}

impl Bot {
    pub fn win_rate(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            (self.wins as f64 + self.draws as f64 * 0.5) / self.games_played as f64
        }
    }
}

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
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub match_id: String,
    pub game_number: i32,
    pub result: Option<String>,
    pub opening_name: Option<String>,
    pub pgn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub ply: i32,
    pub uci: String,
    pub san: Option<String>,
    pub fen_after: String,
    pub bot_eval: Option<i32>,
    pub stockfish_eval: Option<i32>,
}
```

**Step 2: Create repo/mod.rs**

```rust
pub mod bots;
pub use bots::BotRepo;
```

**Step 3: Create repo/bots.rs with tests**

```rust
//! Bot repository for database operations.

use crate::db::DbPool;
use crate::models::Bot;
use rusqlite::Result as SqliteResult;

pub struct BotRepo {
    db: DbPool,
}

impl BotRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn list(&self) -> SqliteResult<Vec<Bot>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, elo_rating, games_played, wins, losses, draws
             FROM bots ORDER BY elo_rating DESC"
        )?;

        let bots = stmt.query_map([], |row| {
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

    pub fn get(&self, name: &str) -> SqliteResult<Option<Bot>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, elo_rating, games_played, wins, losses, draws
             FROM bots WHERE name = ?1"
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
        }).optional()
    }

    pub fn ensure(&self, name: &str) -> SqliteResult<()> {
        let conn = self.db.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO bots (name) VALUES (?1)",
            [name],
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
}
```

**Step 4: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/bot-arena-server/src/
git commit -m "feat(bot-arena-server): add models and bot repository"
```

---

### Task 4: Add REST API routes for bots

**Files:**
- Create: `crates/bot-arena-server/src/api/mod.rs`
- Create: `crates/bot-arena-server/src/api/bots.rs`
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Create api/mod.rs**

```rust
pub mod bots;
```

**Step 2: Create api/bots.rs**

```rust
//! Bot API handlers.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::models::Bot;
use crate::repo::BotRepo;
use crate::AppState;

pub async fn list_bots(
    State(state): State<AppState>,
) -> Result<Json<Vec<Bot>>, StatusCode> {
    let repo = BotRepo::new(state.db.clone());
    repo.list()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_bot(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Bot>, StatusCode> {
    let repo = BotRepo::new(state.db.clone());
    repo.get(&name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
```

**Step 3: Update main.rs with AppState and routes**

```rust
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;

mod api;
mod db;
mod models;
mod repo;

use db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::init();

    let db = db::init_db("data/arena.db").expect("Failed to init database");
    let state = AppState { db };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/bots", get(api::bots::list_bots))
        .route("/api/bots/:name", get(api::bots::get_bot))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

**Step 4: Verify compilation**

Run: `cargo build -p bot-arena-server`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add crates/bot-arena-server/src/
git commit -m "feat(bot-arena-server): add REST API routes for bots"
```

---

### Task 5: Add match repository

**Files:**
- Create: `crates/bot-arena-server/src/repo/matches.rs`
- Modify: `crates/bot-arena-server/src/repo/mod.rs`

**Step 1: Create repo/matches.rs**

```rust
//! Match repository for database operations.

use crate::db::DbPool;
use crate::models::{Game, Match, Move};
use rusqlite::Result as SqliteResult;
use uuid::Uuid;

pub struct MatchRepo {
    db: DbPool,
}

#[derive(Debug)]
pub struct MatchFilter {
    pub bot: Option<String>,
    pub limit: i32,
    pub offset: i32,
}

impl Default for MatchFilter {
    fn default() -> Self {
        Self {
            bot: None,
            limit: 20,
            offset: 0,
        }
    }
}

impl MatchRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn list(&self, filter: MatchFilter) -> SqliteResult<Vec<Match>> {
        let conn = self.db.lock().unwrap();

        let sql = if filter.bot.is_some() {
            "SELECT id, white_bot, black_bot, games_total, white_score, black_score,
                    opening_id, movetime_ms, started_at, finished_at, status
             FROM matches
             WHERE white_bot = ?1 OR black_bot = ?1
             ORDER BY started_at DESC LIMIT ?2 OFFSET ?3"
        } else {
            "SELECT id, white_bot, black_bot, games_total, white_score, black_score,
                    opening_id, movetime_ms, started_at, finished_at, status
             FROM matches
             ORDER BY started_at DESC LIMIT ?1 OFFSET ?2"
        };

        let mut stmt = conn.prepare(sql)?;

        let matches = if let Some(ref bot) = filter.bot {
            stmt.query_map([bot, &filter.limit.to_string(), &filter.offset.to_string()], Self::map_row)?
        } else {
            stmt.query_map([&filter.limit.to_string(), &filter.offset.to_string()], Self::map_row)?
        };

        Ok(matches.filter_map(|r| r.ok()).collect())
    }

    pub fn get(&self, id: &str) -> SqliteResult<Option<Match>> {
        let conn = self.db.lock().unwrap();
        conn.prepare(
            "SELECT id, white_bot, black_bot, games_total, white_score, black_score,
                    opening_id, movetime_ms, started_at, finished_at, status
             FROM matches WHERE id = ?1"
        )?
        .query_row([id], Self::map_row)
        .optional()
    }

    pub fn get_games(&self, match_id: &str) -> SqliteResult<Vec<Game>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, match_id, game_number, result, opening_name, pgn
             FROM games WHERE match_id = ?1 ORDER BY game_number"
        )?;

        let games = stmt.query_map([match_id], |row| {
            Ok(Game {
                id: row.get(0)?,
                match_id: row.get(1)?,
                game_number: row.get(2)?,
                result: row.get(3)?,
                opening_name: row.get(4)?,
                pgn: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(games)
    }

    pub fn get_moves(&self, game_id: &str) -> SqliteResult<Vec<Move>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ply, uci, san, fen_after, bot_eval, stockfish_eval
             FROM moves WHERE game_id = ?1 ORDER BY ply"
        )?;

        let moves = stmt.query_map([game_id], |row| {
            Ok(Move {
                ply: row.get(0)?,
                uci: row.get(1)?,
                san: row.get(2)?,
                fen_after: row.get(3)?,
                bot_eval: row.get(4)?,
                stockfish_eval: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(moves)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Match> {
        Ok(Match {
            id: row.get(0)?,
            white_bot: row.get(1)?,
            black_bot: row.get(2)?,
            games_total: row.get(3)?,
            white_score: row.get(4)?,
            black_score: row.get(5)?,
            opening_id: row.get(6)?,
            movetime_ms: row.get(7)?,
            started_at: row.get(8)?,
            finished_at: row.get(9)?,
            status: row.get(10)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;

    #[test]
    fn test_list_matches_empty() {
        let db = init_db(":memory:").unwrap();
        let repo = MatchRepo::new(db);
        let matches = repo.list(MatchFilter::default()).unwrap();
        assert!(matches.is_empty());
    }
}
```

**Step 2: Update repo/mod.rs**

```rust
pub mod bots;
pub mod matches;

pub use bots::BotRepo;
pub use matches::{MatchRepo, MatchFilter};
```

**Step 3: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena-server/src/repo/
git commit -m "feat(bot-arena-server): add match repository"
```

---

### Task 6: Add REST API routes for matches

**Files:**
- Create: `crates/bot-arena-server/src/api/matches.rs`
- Modify: `crates/bot-arena-server/src/api/mod.rs`
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Create api/matches.rs**

```rust
//! Match API handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::models::{Game, Match, Move};
use crate::repo::{MatchRepo, MatchFilter};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListMatchesQuery {
    pub bot: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

pub async fn list_matches(
    State(state): State<AppState>,
    Query(query): Query<ListMatchesQuery>,
) -> Result<Json<Vec<Match>>, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());
    let filter = MatchFilter {
        bot: query.bot,
        limit: query.limit.unwrap_or(20),
        offset: query.offset.unwrap_or(0),
    };

    repo.list(filter)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_match(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Match>, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());
    repo.get(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Debug, serde::Serialize)]
pub struct MatchDetail {
    #[serde(flatten)]
    pub match_info: Match,
    pub games: Vec<Game>,
}

pub async fn get_match_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MatchDetail>, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());

    let match_info = repo.get(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let games = repo.get_games(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MatchDetail { match_info, games }))
}

pub async fn get_game_moves(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> Result<Json<Vec<Move>>, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());
    repo.get_moves(&game_id)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
```

**Step 2: Update api/mod.rs**

```rust
pub mod bots;
pub mod matches;
```

**Step 3: Add routes to main.rs**

Add these routes to the Router:
```rust
.route("/api/matches", get(api::matches::list_matches))
.route("/api/matches/:id", get(api::matches::get_match_detail))
.route("/api/games/:id/moves", get(api::matches::get_game_moves))
```

**Step 4: Verify compilation**

Run: `cargo build -p bot-arena-server`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add crates/bot-arena-server/src/
git commit -m "feat(bot-arena-server): add REST API routes for matches"
```

---

## Phase B: Elo System

### Task 7: Add Elo calculation module

**Files:**
- Create: `crates/bot-arena-server/src/elo.rs`
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Create elo.rs with tests**

```rust
//! Elo rating calculation.

const K_FACTOR: f64 = 32.0;

/// Calculate expected score for player A against player B.
pub fn expected_score(rating_a: i32, rating_b: i32) -> f64 {
    1.0 / (1.0 + 10_f64.powf((rating_b - rating_a) as f64 / 400.0))
}

/// Calculate new rating after a game.
///
/// # Arguments
/// * `rating` - Current rating
/// * `opponent_rating` - Opponent's rating
/// * `actual` - Actual score (1.0 = win, 0.5 = draw, 0.0 = loss)
pub fn new_rating(rating: i32, opponent_rating: i32, actual: f64) -> i32 {
    let expected = expected_score(rating, opponent_rating);
    let new = rating as f64 + K_FACTOR * (actual - expected);
    new.round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_score_equal_ratings() {
        let expected = expected_score(1500, 1500);
        assert!((expected - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_expected_score_higher_rated() {
        let expected = expected_score(1700, 1500);
        assert!(expected > 0.7);
        assert!(expected < 0.8);
    }

    #[test]
    fn test_expected_score_lower_rated() {
        let expected = expected_score(1300, 1500);
        assert!(expected < 0.3);
        assert!(expected > 0.2);
    }

    #[test]
    fn test_new_rating_win() {
        let new = new_rating(1500, 1500, 1.0);
        assert_eq!(new, 1516); // +16 for expected win
    }

    #[test]
    fn test_new_rating_loss() {
        let new = new_rating(1500, 1500, 0.0);
        assert_eq!(new, 1484); // -16 for expected loss
    }

    #[test]
    fn test_new_rating_draw() {
        let new = new_rating(1500, 1500, 0.5);
        assert_eq!(new, 1500); // No change for draw between equals
    }

    #[test]
    fn test_new_rating_upset_win() {
        // Lower rated player wins
        let new = new_rating(1300, 1500, 1.0);
        assert!(new > 1320); // Bigger gain for upset
    }
}
```

**Step 2: Add mod elo to main.rs**

**Step 3: Run tests**

Run: `cargo test -p bot-arena-server elo`
Expected: All 7 tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena-server/src/elo.rs crates/bot-arena-server/src/main.rs
git commit -m "feat(bot-arena-server): add Elo calculation module"
```

---

### Task 8: Add Elo update to bot repository

**Files:**
- Modify: `crates/bot-arena-server/src/repo/bots.rs`

**Step 1: Add update_after_game method**

```rust
use crate::elo;

impl BotRepo {
    // ... existing methods ...

    /// Update bot stats and Elo after a game.
    ///
    /// # Arguments
    /// * `name` - Bot name
    /// * `opponent_rating` - Opponent's Elo rating
    /// * `result` - 1.0 = win, 0.5 = draw, 0.0 = loss
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
}
```

**Step 2: Add test for update_after_game**

```rust
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
```

**Step 3: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/bot-arena-server/src/
git commit -m "feat(bot-arena-server): add Elo update to bot repository"
```

---

## Phase C: Frontend Foundation

### Task 9: Create SvelteKit app scaffold

**Files:**
- Create: `apps/web/bot-arena-ui/package.json`
- Create: `apps/web/bot-arena-ui/svelte.config.js`
- Create: `apps/web/bot-arena-ui/vite.config.ts`
- Create: `apps/web/bot-arena-ui/tsconfig.json`
- Create: `apps/web/bot-arena-ui/src/app.html`
- Create: `apps/web/bot-arena-ui/src/app.d.ts`
- Modify: `pnpm-workspace.yaml`

**Step 1: Create package.json**

```json
{
  "name": "@tmttn-chess/bot-arena-ui",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.50.0",
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "svelte": "^5.0.0",
    "typescript": "^5.9.0",
    "vite": "^5.0.0"
  },
  "dependencies": {
    "@tmttn-chess/board": "workspace:*",
    "@tmttn-chess/game-store": "workspace:*"
  }
}
```

**Step 2: Create svelte.config.js**

```javascript
import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html'
    }),
    paths: {
      base: ''
    }
  }
};

export default config;
```

**Step 3: Create vite.config.ts**

```typescript
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      '/api': 'http://localhost:3000',
      '/ws': {
        target: 'ws://localhost:3000',
        ws: true
      }
    }
  }
});
```

**Step 4: Create tsconfig.json**

```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "strict": true,
    "moduleResolution": "bundler"
  }
}
```

**Step 5: Create src/app.html**

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Bot Arena</title>
    %sveltekit.head%
  </head>
  <body data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
```

**Step 6: Create src/app.d.ts**

```typescript
declare global {
  namespace App {}
}

export {};
```

**Step 7: Update pnpm-workspace.yaml**

```yaml
packages:
  - "packages/*"
  - "apps/web/chess-devtools/svelte-test-ui"
  - "apps/web/bot-arena-ui"
```

**Step 8: Install dependencies**

Run: `cd apps/web/bot-arena-ui && pnpm install`

**Step 9: Commit**

```bash
git add apps/web/bot-arena-ui/ pnpm-workspace.yaml pnpm-lock.yaml
git commit -m "feat(bot-arena-ui): create SvelteKit app scaffold"
```

---

### Task 10: Add API client module

**Files:**
- Create: `apps/web/bot-arena-ui/src/lib/api.ts`
- Create: `apps/web/bot-arena-ui/src/lib/types.ts`

**Step 1: Create types.ts**

```typescript
// API response types

export interface Bot {
  name: string;
  elo_rating: number;
  games_played: number;
  wins: number;
  losses: number;
  draws: number;
}

export interface Match {
  id: string;
  white_bot: string;
  black_bot: string;
  games_total: number;
  white_score: number;
  black_score: number;
  opening_id: string | null;
  movetime_ms: number;
  started_at: string;
  finished_at: string | null;
  status: string;
}

export interface Game {
  id: string;
  match_id: string;
  game_number: number;
  result: string | null;
  opening_name: string | null;
  pgn: string | null;
}

export interface Move {
  ply: number;
  uci: string;
  san: string | null;
  fen_after: string;
  bot_eval: number | null;
  stockfish_eval: number | null;
}

export interface MatchDetail extends Match {
  games: Game[];
}
```

**Step 2: Create api.ts**

```typescript
import type { Bot, Match, MatchDetail, Move } from './types';

const BASE_URL = '/api';

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(`${BASE_URL}${url}`);
  if (!response.ok) {
    throw new Error(`API error: ${response.status}`);
  }
  return response.json();
}

export const api = {
  // Bots
  getBots(): Promise<Bot[]> {
    return fetchJson('/bots');
  },

  getBot(name: string): Promise<Bot> {
    return fetchJson(`/bots/${encodeURIComponent(name)}`);
  },

  // Matches
  getMatches(params?: { bot?: string; limit?: number; offset?: number }): Promise<Match[]> {
    const searchParams = new URLSearchParams();
    if (params?.bot) searchParams.set('bot', params.bot);
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.offset) searchParams.set('offset', params.offset.toString());

    const query = searchParams.toString();
    return fetchJson(`/matches${query ? `?${query}` : ''}`);
  },

  getMatch(id: string): Promise<MatchDetail> {
    return fetchJson(`/matches/${id}`);
  },

  getGameMoves(gameId: string): Promise<Move[]> {
    return fetchJson(`/games/${gameId}/moves`);
  },
};
```

**Step 3: Commit**

```bash
git add apps/web/bot-arena-ui/src/lib/
git commit -m "feat(bot-arena-ui): add API client module"
```

---

### Task 11: Create layout and dashboard page

**Files:**
- Create: `apps/web/bot-arena-ui/src/routes/+layout.svelte`
- Create: `apps/web/bot-arena-ui/src/routes/+page.svelte`
- Create: `apps/web/bot-arena-ui/src/app.css`

**Step 1: Create app.css**

```css
:root {
  --bg: #1a1a2e;
  --bg-secondary: #16213e;
  --text: #eee;
  --text-muted: #888;
  --accent: #0f3460;
  --highlight: #e94560;
  --success: #4ade80;
  --warning: #fbbf24;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: system-ui, -apple-system, sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.5;
}

a {
  color: var(--highlight);
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}
```

**Step 2: Create +layout.svelte**

```svelte
<script lang="ts">
  import '../app.css';
</script>

<div class="layout">
  <header>
    <nav>
      <a href="/" class="logo">Bot Arena</a>
      <div class="nav-links">
        <a href="/">Dashboard</a>
        <a href="/games">Games</a>
        <a href="/match/new">New Match</a>
      </div>
    </nav>
  </header>

  <main>
    <slot />
  </main>
</div>

<style>
  .layout {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  header {
    background: var(--bg-secondary);
    padding: 1rem 2rem;
    border-bottom: 1px solid var(--accent);
  }

  nav {
    display: flex;
    align-items: center;
    gap: 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .logo {
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--highlight);
  }

  .nav-links {
    display: flex;
    gap: 1.5rem;
  }

  .nav-links a {
    color: var(--text);
  }

  main {
    flex: 1;
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
  }
</style>
```

**Step 3: Create +page.svelte (Dashboard)**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Bot, Match } from '$lib/types';

  let bots: Bot[] = $state([]);
  let recentMatches: Match[] = $state([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      [bots, recentMatches] = await Promise.all([
        api.getBots(),
        api.getMatches({ limit: 10 })
      ]);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      loading = false;
    }
  });

  function winRate(bot: Bot): string {
    if (bot.games_played === 0) return '-';
    const rate = (bot.wins + bot.draws * 0.5) / bot.games_played * 100;
    return rate.toFixed(1) + '%';
  }
</script>

<div class="dashboard">
  <h1>Bot Arena Dashboard</h1>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <section class="leaderboard">
      <h2>Leaderboard</h2>
      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>Bot</th>
            <th>Elo</th>
            <th>W/L/D</th>
            <th>Win Rate</th>
          </tr>
        </thead>
        <tbody>
          {#each bots as bot, i}
            <tr>
              <td>{i + 1}</td>
              <td><a href="/bots/{bot.name}">{bot.name}</a></td>
              <td>{bot.elo_rating}</td>
              <td>{bot.wins}/{bot.losses}/{bot.draws}</td>
              <td>{winRate(bot)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section class="recent-matches">
      <h2>Recent Matches</h2>
      {#if recentMatches.length === 0}
        <p class="empty">No matches yet</p>
      {:else}
        <ul>
          {#each recentMatches as match}
            <li>
              <a href="/games/{match.id}">
                {match.white_bot} vs {match.black_bot}
                ({match.white_score}-{match.black_score})
              </a>
              <span class="date">{new Date(match.started_at).toLocaleDateString()}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</div>

<style>
  .dashboard {
    display: grid;
    gap: 2rem;
  }

  h1 {
    margin-bottom: 1rem;
  }

  h2 {
    margin-bottom: 1rem;
    color: var(--text-muted);
    font-size: 1.2rem;
  }

  section {
    background: var(--bg-secondary);
    padding: 1.5rem;
    border-radius: 8px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--accent);
  }

  th {
    color: var(--text-muted);
    font-weight: 500;
  }

  .recent-matches ul {
    list-style: none;
  }

  .recent-matches li {
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--accent);
    display: flex;
    justify-content: space-between;
  }

  .date {
    color: var(--text-muted);
  }

  .loading, .error, .empty {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .error {
    color: var(--highlight);
  }
</style>
```

**Step 4: Verify it builds**

Run: `cd apps/web/bot-arena-ui && pnpm build`
Expected: Builds successfully

**Step 5: Commit**

```bash
git add apps/web/bot-arena-ui/
git commit -m "feat(bot-arena-ui): add layout and dashboard page"
```

---

## Phase D: Game Viewing

### Task 12: Create game browser page

**Files:**
- Create: `apps/web/bot-arena-ui/src/routes/games/+page.svelte`

**Step 1: Create games/+page.svelte**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Match } from '$lib/types';

  let matches: Match[] = $state([]);
  let loading = $state(true);
  let offset = $state(0);
  const limit = 20;

  async function loadMatches() {
    loading = true;
    try {
      matches = await api.getMatches({ limit, offset });
    } finally {
      loading = false;
    }
  }

  onMount(loadMatches);

  function prevPage() {
    if (offset >= limit) {
      offset -= limit;
      loadMatches();
    }
  }

  function nextPage() {
    if (matches.length === limit) {
      offset += limit;
      loadMatches();
    }
  }

  function formatResult(match: Match): string {
    if (match.white_score > match.black_score) return 'White wins';
    if (match.black_score > match.white_score) return 'Black wins';
    return 'Draw';
  }
</script>

<div class="games-page">
  <h1>Game Browser</h1>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Date</th>
          <th>White</th>
          <th>Black</th>
          <th>Score</th>
          <th>Games</th>
          <th>Result</th>
        </tr>
      </thead>
      <tbody>
        {#each matches as match}
          <tr>
            <td>{new Date(match.started_at).toLocaleDateString()}</td>
            <td>{match.white_bot}</td>
            <td>{match.black_bot}</td>
            <td>{match.white_score}-{match.black_score}</td>
            <td>{match.games_total}</td>
            <td>
              <a href="/games/{match.id}">{formatResult(match)}</a>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    <div class="pagination">
      <button onclick={prevPage} disabled={offset === 0}>Previous</button>
      <span>Page {Math.floor(offset / limit) + 1}</span>
      <button onclick={nextPage} disabled={matches.length < limit}>Next</button>
    </div>
  {/if}
</div>

<style>
  .games-page {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    background: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
  }

  th, td {
    padding: 1rem;
    text-align: left;
    border-bottom: 1px solid var(--accent);
  }

  th {
    background: var(--accent);
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
  }

  button {
    padding: 0.5rem 1rem;
    background: var(--accent);
    border: none;
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .loading {
    text-align: center;
    padding: 2rem;
  }
</style>
```

**Step 2: Commit**

```bash
git add apps/web/bot-arena-ui/src/routes/games/
git commit -m "feat(bot-arena-ui): add game browser page"
```

---

### Task 13: Create game detail page with board

**Files:**
- Create: `apps/web/bot-arena-ui/src/routes/games/[id]/+page.svelte`

**Step 1: Create games/[id]/+page.svelte**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { Board } from '@tmttn-chess/board';
  import type { MatchDetail, Move } from '$lib/types';

  let matchDetail: MatchDetail | null = $state(null);
  let moves: Move[] = $state([]);
  let currentPly = $state(0);
  let loading = $state(true);

  const id = $derived($page.params.id);

  onMount(async () => {
    try {
      matchDetail = await api.getMatch(id);
      if (matchDetail.games.length > 0) {
        moves = await api.getGameMoves(matchDetail.games[0].id);
      }
    } finally {
      loading = false;
    }
  });

  const currentFen = $derived(
    currentPly === 0
      ? 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
      : moves[currentPly - 1]?.fen_after ?? ''
  );

  function goTo(ply: number) {
    if (ply >= 0 && ply <= moves.length) {
      currentPly = ply;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') goTo(currentPly - 1);
    if (e.key === 'ArrowRight') goTo(currentPly + 1);
    if (e.key === 'Home') goTo(0);
    if (e.key === 'End') goTo(moves.length);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="game-detail">
  {#if loading}
    <p class="loading">Loading...</p>
  {:else if matchDetail}
    <header>
      <h1>{matchDetail.white_bot} vs {matchDetail.black_bot}</h1>
      <p class="score">{matchDetail.white_score} - {matchDetail.black_score}</p>
    </header>

    <div class="content">
      <div class="board-container">
        <Board fen={currentFen} interactive={false} />
      </div>

      <div class="moves-panel">
        <div class="controls">
          <button onclick={() => goTo(0)}>⏮</button>
          <button onclick={() => goTo(currentPly - 1)}>◀</button>
          <button onclick={() => goTo(currentPly + 1)}>▶</button>
          <button onclick={() => goTo(moves.length)}>⏭</button>
        </div>

        <div class="move-list">
          {#each moves as move, i}
            {#if i % 2 === 0}
              <span class="move-number">{Math.floor(i / 2) + 1}.</span>
            {/if}
            <button
              class="move"
              class:active={currentPly === i + 1}
              onclick={() => goTo(i + 1)}
            >
              {move.san ?? move.uci}
            </button>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .game-detail {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .score {
    font-size: 1.5rem;
    font-weight: bold;
  }

  .content {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: 2rem;
  }

  .board-container {
    aspect-ratio: 1;
    max-width: 600px;
  }

  .moves-panel {
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 1rem;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .controls button {
    flex: 1;
    padding: 0.5rem;
    background: var(--accent);
    border: none;
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
  }

  .move-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    max-height: 400px;
    overflow-y: auto;
  }

  .move-number {
    width: 2rem;
    color: var(--text-muted);
  }

  .move {
    padding: 0.25rem 0.5rem;
    background: transparent;
    border: none;
    color: var(--text);
    cursor: pointer;
    border-radius: 4px;
  }

  .move:hover {
    background: var(--accent);
  }

  .move.active {
    background: var(--highlight);
  }

  .loading {
    text-align: center;
    padding: 2rem;
  }
</style>
```

**Step 2: Verify it builds**

Run: `cd apps/web/bot-arena-ui && pnpm build`
Expected: Builds successfully

**Step 3: Commit**

```bash
git add apps/web/bot-arena-ui/src/routes/games/
git commit -m "feat(bot-arena-ui): add game detail page with board"
```

---

## Phase E: WebSocket & Live Matches

### Task 14: Add WebSocket support to server

**Files:**
- Create: `crates/bot-arena-server/src/ws.rs`
- Modify: `crates/bot-arena-server/src/main.rs`
- Modify: `crates/bot-arena-server/Cargo.toml`

**Step 1: Add WebSocket dependencies to Cargo.toml**

Add to dependencies:
```toml
axum = { version = "0.7", features = ["ws"] }
futures-util = "0.3"
tokio-tungstenite = "0.24"
```

**Step 2: Create ws.rs**

```rust
//! WebSocket handler for live match updates.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Subscribe { match_id: String },
    Unsubscribe { match_id: String },
    Move { match_id: String, uci: String, eval: Option<i32> },
    GameEnd { match_id: String, result: String, game_num: i32 },
    MatchEnd { match_id: String, score: String },
    MatchStarted { match_id: String, white: String, black: String },
}

pub type WsBroadcast = broadcast::Sender<WsMessage>;

pub fn create_broadcast() -> WsBroadcast {
    let (tx, _) = broadcast::channel(100);
    tx
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(broadcast): State<WsBroadcast>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, broadcast))
}

async fn handle_socket(socket: WebSocket, broadcast: WsBroadcast) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = broadcast.subscribe();

    // Subscribed match IDs
    let subscriptions = Arc::new(tokio::sync::RwLock::new(Vec::<String>::new()));
    let subs_clone = subscriptions.clone();

    // Task to forward broadcast messages to client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let match_id = match &msg {
                WsMessage::Move { match_id, .. } => match_id,
                WsMessage::GameEnd { match_id, .. } => match_id,
                WsMessage::MatchEnd { match_id, .. } => match_id,
                WsMessage::MatchStarted { match_id, .. } => match_id,
                _ => continue,
            };

            let subs = subs_clone.read().await;
            if subs.contains(match_id) {
                let json = serde_json::to_string(&msg).unwrap();
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages from client
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                match ws_msg {
                    WsMessage::Subscribe { match_id } => {
                        let mut subs = subscriptions.write().await;
                        if !subs.contains(&match_id) {
                            subs.push(match_id);
                        }
                    }
                    WsMessage::Unsubscribe { match_id } => {
                        let mut subs = subscriptions.write().await;
                        subs.retain(|id| id != &match_id);
                    }
                    _ => {}
                }
            }
        }
    }

    send_task.abort();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Move {
            match_id: "123".to_string(),
            uci: "e2e4".to_string(),
            eval: Some(30),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"move\""));
        assert!(json.contains("\"uci\":\"e2e4\""));
    }
}
```

**Step 3: Update main.rs to include WebSocket route**

Add `mod ws;` and add route:
```rust
.route("/ws", get(ws::ws_handler))
```

Add to AppState:
```rust
pub ws_broadcast: ws::WsBroadcast,
```

Initialize in main:
```rust
let ws_broadcast = ws::create_broadcast();
let state = AppState { db, ws_broadcast };
```

**Step 4: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/bot-arena-server/
git commit -m "feat(bot-arena-server): add WebSocket support for live updates"
```

---

### Task 15: Add live match page to frontend

**Files:**
- Create: `apps/web/bot-arena-ui/src/lib/ws.ts`
- Create: `apps/web/bot-arena-ui/src/routes/match/live/[id]/+page.svelte`

**Step 1: Create ws.ts**

```typescript
import { writable } from 'svelte/store';
import type { Move } from './types';

export interface LiveMatchState {
  connected: boolean;
  moves: Move[];
  currentGame: number;
  score: { white: number; black: number };
}

export function createLiveMatchStore(matchId: string) {
  const { subscribe, update } = writable<LiveMatchState>({
    connected: false,
    moves: [],
    currentGame: 1,
    score: { white: 0, black: 0 },
  });

  let ws: WebSocket | null = null;

  function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

    ws.onopen = () => {
      update(s => ({ ...s, connected: true }));
      ws?.send(JSON.stringify({ type: 'subscribe', match_id: matchId }));
    };

    ws.onclose = () => {
      update(s => ({ ...s, connected: false }));
    };

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);

      switch (msg.type) {
        case 'move':
          update(s => ({
            ...s,
            moves: [...s.moves, {
              ply: s.moves.length + 1,
              uci: msg.uci,
              san: null,
              fen_after: '',
              bot_eval: msg.eval,
              stockfish_eval: null,
            }],
          }));
          break;

        case 'game_end':
          update(s => ({
            ...s,
            moves: [],
            currentGame: msg.game_num + 1,
          }));
          break;

        case 'match_end':
          const [white, black] = msg.score.split('-').map(Number);
          update(s => ({
            ...s,
            score: { white, black },
          }));
          break;
      }
    };
  }

  function disconnect() {
    if (ws) {
      ws.send(JSON.stringify({ type: 'unsubscribe', match_id: matchId }));
      ws.close();
      ws = null;
    }
  }

  return {
    subscribe,
    connect,
    disconnect,
  };
}
```

**Step 2: Create match/live/[id]/+page.svelte**

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { createLiveMatchStore } from '$lib/ws';
  import { api } from '$lib/api';
  import { Board } from '@tmttn-chess/board';
  import type { Match } from '$lib/types';

  let matchInfo: Match | null = $state(null);
  const id = $derived($page.params.id);
  const store = createLiveMatchStore(id);

  let state = $state({
    connected: false,
    moves: [] as any[],
    currentGame: 1,
    score: { white: 0, black: 0 },
  });

  onMount(async () => {
    const detail = await api.getMatch(id);
    matchInfo = detail;

    store.subscribe(s => state = s);
    store.connect();
  });

  onDestroy(() => {
    store.disconnect();
  });

  // For now, show starting position (full FEN tracking requires WASM)
  const currentFen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
</script>

<div class="live-match">
  {#if matchInfo}
    <header>
      <h1>Live: {matchInfo.white_bot} vs {matchInfo.black_bot}</h1>
      <div class="status" class:connected={state.connected}>
        {state.connected ? '● Connected' : '○ Disconnected'}
      </div>
    </header>

    <div class="match-info">
      <span>Game {state.currentGame} of {matchInfo.games_total}</span>
      <span class="score">{state.score.white} - {state.score.black}</span>
    </div>

    <div class="content">
      <div class="board-container">
        <Board fen={currentFen} interactive={false} />
      </div>

      <div class="moves-panel">
        <h2>Moves</h2>
        <div class="move-list">
          {#each state.moves as move, i}
            {#if i % 2 === 0}
              <span class="move-number">{Math.floor(i / 2) + 1}.</span>
            {/if}
            <span class="move">{move.uci}</span>
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <p class="loading">Loading...</p>
  {/if}
</div>

<style>
  .live-match {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .status {
    color: var(--text-muted);
  }

  .status.connected {
    color: var(--success);
  }

  .match-info {
    display: flex;
    justify-content: space-between;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 8px;
  }

  .score {
    font-size: 1.5rem;
    font-weight: bold;
  }

  .content {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: 2rem;
  }

  .board-container {
    aspect-ratio: 1;
    max-width: 600px;
  }

  .moves-panel {
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 1rem;
  }

  .moves-panel h2 {
    margin-bottom: 1rem;
    color: var(--text-muted);
  }

  .move-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .move-number {
    width: 2rem;
    color: var(--text-muted);
  }

  .move {
    padding: 0.25rem 0.5rem;
  }

  .loading {
    text-align: center;
    padding: 2rem;
  }
</style>
```

**Step 3: Commit**

```bash
git add apps/web/bot-arena-ui/
git commit -m "feat(bot-arena-ui): add live match page with WebSocket"
```

---

## Phase F: Match Creation

### Task 16: Add match creation API endpoint

**Files:**
- Modify: `crates/bot-arena-server/src/api/matches.rs`
- Modify: `crates/bot-arena-server/src/repo/matches.rs`
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Add CreateMatch request type to api/matches.rs**

```rust
#[derive(Debug, Deserialize)]
pub struct CreateMatchRequest {
    pub white_bot: String,
    pub black_bot: String,
    pub games: i32,
    pub movetime_ms: Option<i32>,
    pub opening_id: Option<String>,
}

pub async fn create_match(
    State(state): State<AppState>,
    Json(req): Json<CreateMatchRequest>,
) -> Result<Json<Match>, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());

    // Ensure bots exist
    let bot_repo = BotRepo::new(state.db.clone());
    bot_repo.ensure(&req.white_bot).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    bot_repo.ensure(&req.black_bot).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let id = repo.create(
        &req.white_bot,
        &req.black_bot,
        req.games,
        req.movetime_ms.unwrap_or(1000),
        req.opening_id.as_deref(),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let match_info = repo.get(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: Trigger match runner (Phase G)

    Ok(Json(match_info))
}
```

**Step 2: Add create method to repo/matches.rs**

```rust
pub fn create(
    &self,
    white_bot: &str,
    black_bot: &str,
    games_total: i32,
    movetime_ms: i32,
    opening_id: Option<&str>,
) -> SqliteResult<String> {
    let conn = self.db.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO matches (id, white_bot, black_bot, games_total, movetime_ms, opening_id, started_at, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending')",
        (&id, white_bot, black_bot, games_total, movetime_ms, opening_id, &now),
    )?;

    Ok(id)
}
```

**Step 3: Add POST route to main.rs**

```rust
use axum::routing::post;

// In router:
.route("/api/matches", get(api::matches::list_matches).post(api::matches::create_match))
```

**Step 4: Run tests**

Run: `cargo test -p bot-arena-server`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/bot-arena-server/
git commit -m "feat(bot-arena-server): add match creation API endpoint"
```

---

### Task 17: Add new match page to frontend

**Files:**
- Create: `apps/web/bot-arena-ui/src/routes/match/new/+page.svelte`
- Modify: `apps/web/bot-arena-ui/src/lib/api.ts`

**Step 1: Add createMatch to api.ts**

```typescript
export interface CreateMatchRequest {
  white_bot: string;
  black_bot: string;
  games: number;
  movetime_ms?: number;
  opening_id?: string;
}

// Add to api object:
async createMatch(req: CreateMatchRequest): Promise<Match> {
  const response = await fetch(`${BASE_URL}/matches`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!response.ok) {
    throw new Error(`API error: ${response.status}`);
  }
  return response.json();
}
```

**Step 2: Create match/new/+page.svelte**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { Bot } from '$lib/types';

  let bots: Bot[] = $state([]);
  let whiteBot = $state('');
  let blackBot = $state('');
  let games = $state(10);
  let movetime = $state(1000);
  let submitting = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    bots = await api.getBots();
    if (bots.length >= 2) {
      whiteBot = bots[0].name;
      blackBot = bots[1].name;
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (whiteBot === blackBot) {
      error = 'Please select different bots';
      return;
    }

    submitting = true;
    error = null;

    try {
      const match = await api.createMatch({
        white_bot: whiteBot,
        black_bot: blackBot,
        games,
        movetime_ms: movetime,
      });
      goto(`/match/live/${match.id}`);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to create match';
      submitting = false;
    }
  }
</script>

<div class="new-match">
  <h1>New Match</h1>

  <form onsubmit={handleSubmit}>
    <div class="field">
      <label for="white">White</label>
      <select id="white" bind:value={whiteBot}>
        {#each bots as bot}
          <option value={bot.name}>{bot.name} ({bot.elo_rating})</option>
        {/each}
      </select>
    </div>

    <div class="field">
      <label for="black">Black</label>
      <select id="black" bind:value={blackBot}>
        {#each bots as bot}
          <option value={bot.name}>{bot.name} ({bot.elo_rating})</option>
        {/each}
      </select>
    </div>

    <div class="field">
      <label for="games">Number of Games</label>
      <input type="number" id="games" bind:value={games} min="1" max="100" />
    </div>

    <div class="field">
      <label for="movetime">Move Time (ms)</label>
      <input type="number" id="movetime" bind:value={movetime} min="100" max="60000" step="100" />
    </div>

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <button type="submit" disabled={submitting}>
      {submitting ? 'Starting...' : 'Start Match'}
    </button>
  </form>
</div>

<style>
  .new-match {
    max-width: 400px;
    margin: 0 auto;
  }

  h1 {
    margin-bottom: 2rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  label {
    color: var(--text-muted);
  }

  select, input {
    padding: 0.75rem;
    background: var(--bg-secondary);
    border: 1px solid var(--accent);
    border-radius: 4px;
    color: var(--text);
    font-size: 1rem;
  }

  button {
    padding: 1rem;
    background: var(--highlight);
    border: none;
    border-radius: 4px;
    color: var(--text);
    font-size: 1rem;
    cursor: pointer;
    margin-top: 1rem;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    color: var(--highlight);
    text-align: center;
  }
</style>
```

**Step 3: Verify it builds**

Run: `cd apps/web/bot-arena-ui && pnpm build`
Expected: Builds successfully

**Step 4: Commit**

```bash
git add apps/web/bot-arena-ui/
git commit -m "feat(bot-arena-ui): add new match page"
```

---

## Phase G: Integration & Polish

### Task 18: Add CORS and static file serving

**Files:**
- Modify: `crates/bot-arena-server/src/main.rs`

**Step 1: Add CORS middleware**

```rust
use tower_http::cors::{Any, CorsLayer};

// In main, before routes:
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

// Add to router:
.layer(cors)
```

**Step 2: Add static file serving (for production)**

```rust
use tower_http::services::ServeDir;

// Add fallback for SPA routing:
.fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
```

**Step 3: Verify compilation**

Run: `cargo build -p bot-arena-server`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/bot-arena-server/src/main.rs
git commit -m "feat(bot-arena-server): add CORS and static file serving"
```

---

### Task 19: Final integration test

**Step 1: Build everything**

```bash
# Build Rust server
cargo build -p bot-arena-server

# Build frontend
cd apps/web/bot-arena-ui && pnpm build

# Copy frontend build to server static dir
mkdir -p ../../crates/bot-arena-server/static
cp -r build/* ../../crates/bot-arena-server/static/
```

**Step 2: Run all tests**

```bash
cargo test --workspace
cd apps/web/bot-arena-ui && pnpm test
```

**Step 3: Manual verification**

1. Start server: `cargo run -p bot-arena-server`
2. Open browser to http://localhost:3000
3. Verify dashboard loads
4. Verify navigation works
5. Verify API calls succeed (check Network tab)

**Step 4: Final commit**

```bash
git add .
git commit -m "feat(phase5): complete UI MVP implementation"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| A | 1-6 | Server foundation: Axum, SQLite, REST API |
| B | 7-8 | Elo system with rating updates |
| C | 9-11 | Frontend scaffold, API client, dashboard |
| D | 12-13 | Game browser and detail pages |
| E | 14-15 | WebSocket for live matches |
| F | 16-17 | Match creation API and UI |
| G | 18-19 | CORS, static serving, integration |

Total: 19 tasks

---

*Last updated: 2025-01-21*
