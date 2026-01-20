# Bot Arena Phase 1: Core CLI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the `bot-arena` CLI that can run matches between chess bots via UCI, outputting results to PGN/JSON/SQLite.

**Architecture:** A Rust CLI that spawns bot processes, communicates via UCI stdin/stdout, tracks game state using `chess-engine`, and persists results to a `data/` directory with SQLite for queries and JSON/PGN for individual games.

**Tech Stack:** Rust, clap (CLI), tokio (async), rusqlite (SQLite), chess-engine crate (existing)

---

## Task 1: Create bot-arena crate scaffold

**Files:**
- Create: `crates/bot-arena/Cargo.toml`
- Create: `crates/bot-arena/src/main.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Add crate to workspace**

In `Cargo.toml` (root), add to members:

```toml
members = [
    "crates/chess-core",
    "crates/chess-engine",
    "crates/chess-wasm",
    "crates/uci",
    "crates/bot-random",
    "crates/bot-minimax",
    "crates/bot-bridge",
    "crates/bot-arena",
]
```

**Step 2: Create Cargo.toml**

Create `crates/bot-arena/Cargo.toml`:

```toml
[package]
name = "bot-arena"
version = "0.1.0"
edition = "2021"

[dependencies]
chess-engine = { path = "../chess-engine" }
chess-core = { path = "../chess-core" }
uci = { path = "../uci" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
thiserror = "1"
```

**Step 3: Create minimal main.rs**

Create `crates/bot-arena/src/main.rs`:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bot-arena")]
#[command(about = "Chess bot comparison tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a match between two bots
    Match {
        /// White bot name
        white: String,
        /// Black bot name
        black: String,
        /// Number of games to play
        #[arg(short, long, default_value = "10")]
        games: u32,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Match { white, black, games } => {
            println!("Running {} games: {} vs {}", games, white, black);
        }
    }
}
```

**Step 4: Verify it builds**

Run: `cargo build -p bot-arena`
Expected: Build succeeds

**Step 5: Verify CLI works**

Run: `cargo run -p bot-arena -- match minimax random --games 5`
Expected: `Running 5 games: minimax vs random`

**Step 6: Commit**

```bash
git add crates/bot-arena Cargo.toml
git commit -m "feat(bot-arena): scaffold CLI with clap"
```

---

## Task 2: Add configuration file loading

**Files:**
- Create: `crates/bot-arena/src/config.rs`
- Modify: `crates/bot-arena/src/main.rs`
- Modify: `crates/bot-arena/Cargo.toml`

**Step 1: Add toml dependency**

In `crates/bot-arena/Cargo.toml`, add:

```toml
toml = "0.8"
directories = "5"
```

**Step 2: Create config module**

Create `crates/bot-arena/src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Bot not found: {0}")]
    BotNotFound(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BotConfig {
    pub path: PathBuf,
    #[serde(default = "default_time_control")]
    pub time_control: String,
}

fn default_time_control() -> String {
    "movetime 500".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PresetConfig {
    #[serde(default = "default_games")]
    pub games: u32,
    #[serde(default)]
    pub openings: Vec<String>,
    #[serde(default = "default_time_control")]
    pub time_control: String,
}

fn default_games() -> u32 {
    10
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ArenaConfig {
    #[serde(default)]
    pub bots: HashMap<String, BotConfig>,
    #[serde(default)]
    pub presets: HashMap<String, PresetConfig>,
}

impl ArenaConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_path() -> PathBuf {
        PathBuf::from("arena.toml")
    }

    pub fn get_bot(&self, name: &str) -> Result<&BotConfig, ConfigError> {
        self.bots
            .get(name)
            .ok_or_else(|| ConfigError::BotNotFound(name.to_string()))
    }
}
```

**Step 3: Update main.rs**

Add to `crates/bot-arena/src/main.rs`:

```rust
mod config;

use config::ArenaConfig;

// ... existing code ...

fn main() {
    let cli = Cli::parse();
    let config = ArenaConfig::load().unwrap_or_default();

    match cli.command {
        Commands::Match { white, black, games } => {
            println!("Running {} games: {} vs {}", games, white, black);
            if let Ok(bot) = config.get_bot(&white) {
                println!("White bot path: {:?}", bot.path);
            }
        }
    }
}
```

**Step 4: Verify it builds**

Run: `cargo build -p bot-arena`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add crates/bot-arena/src/config.rs crates/bot-arena/src/main.rs crates/bot-arena/Cargo.toml
git commit -m "feat(bot-arena): add config file loading"
```

---

## Task 3: Implement UCI process spawning

**Files:**
- Create: `crates/bot-arena/src/uci_client.rs`
- Modify: `crates/bot-arena/src/main.rs`

**Step 1: Create UCI client module**

Create `crates/bot-arena/src/uci_client.rs`:

```rust
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UciError {
    #[error("Failed to spawn process: {0}")]
    SpawnError(#[from] std::io::Error),
    #[error("Process not ready")]
    NotReady,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub struct UciClient {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    pub name: String,
}

impl UciClient {
    pub fn spawn<P: AsRef<Path>>(path: P) -> Result<Self, UciError> {
        let mut process = Command::new(path.as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = process.stdin.take().unwrap();
        let stdout = BufReader::new(process.stdout.take().unwrap());

        Ok(Self {
            process,
            stdin,
            stdout,
            name: String::new(),
        })
    }

    pub fn send(&mut self, cmd: &str) -> Result<(), UciError> {
        writeln!(self.stdin, "{}", cmd)?;
        self.stdin.flush()?;
        Ok(())
    }

    pub fn read_line(&mut self) -> Result<String, UciError> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        Ok(line.trim().to_string())
    }

    pub fn init(&mut self) -> Result<(), UciError> {
        self.send("uci")?;

        loop {
            let line = self.read_line()?;
            if line.starts_with("id name ") {
                self.name = line.strip_prefix("id name ").unwrap().to_string();
            }
            if line == "uciok" {
                break;
            }
        }

        self.send("isready")?;
        loop {
            let line = self.read_line()?;
            if line == "readyok" {
                break;
            }
        }

        Ok(())
    }

    pub fn set_position(&mut self, moves: &[String]) -> Result<(), UciError> {
        if moves.is_empty() {
            self.send("position startpos")
        } else {
            self.send(&format!("position startpos moves {}", moves.join(" ")))
        }
    }

    pub fn go(&mut self, time_control: &str) -> Result<String, UciError> {
        self.send(&format!("go {}", time_control))?;

        loop {
            let line = self.read_line()?;
            if line.starts_with("bestmove ") {
                let bestmove = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
                return Ok(bestmove);
            }
        }
    }

    pub fn quit(&mut self) -> Result<(), UciError> {
        self.send("quit")?;
        let _ = self.process.wait();
        Ok(())
    }
}

impl Drop for UciClient {
    fn drop(&mut self) {
        let _ = self.send("quit");
        let _ = self.process.kill();
    }
}
```

**Step 2: Update main.rs to add module**

Add to top of `main.rs`:

```rust
mod uci_client;
```

**Step 3: Verify it builds**

Run: `cargo build -p bot-arena`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add crates/bot-arena/src/uci_client.rs crates/bot-arena/src/main.rs
git commit -m "feat(bot-arena): add UCI process spawning"
```

---

## Task 4: Implement single game execution

**Files:**
- Create: `crates/bot-arena/src/game_runner.rs`
- Modify: `crates/bot-arena/src/main.rs`

**Step 1: Create game runner module**

Create `crates/bot-arena/src/game_runner.rs`:

```rust
use chess_engine::Game;
use crate::uci_client::{UciClient, UciError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("UCI error: {0}")]
    Uci(#[from] UciError),
    #[error("Invalid move: {0}")]
    InvalidMove(String),
}

#[derive(Debug, Clone)]
pub struct GameResult {
    pub moves: Vec<String>,
    pub result: MatchResult,
    pub white_name: String,
    pub black_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchResult {
    WhiteWins,
    BlackWins,
    Draw,
}

pub struct GameRunner {
    white: UciClient,
    black: UciClient,
    time_control: String,
}

impl GameRunner {
    pub fn new(mut white: UciClient, mut black: UciClient, time_control: String) -> Result<Self, GameError> {
        white.init()?;
        black.init()?;
        Ok(Self { white, black, time_control })
    }

    pub fn play_game(&mut self) -> Result<GameResult, GameError> {
        let mut game = Game::new();
        let mut moves: Vec<String> = Vec::new();
        let white_name = self.white.name.clone();
        let black_name = self.black.name.clone();

        loop {
            if game.is_game_over() {
                break;
            }

            let current = if game.side_to_move() == chess_core::Color::White {
                &mut self.white
            } else {
                &mut self.black
            };

            current.set_position(&moves)?;
            let bestmove = current.go(&self.time_control)?;

            if bestmove.is_empty() || bestmove == "(none)" || bestmove == "0000" {
                break;
            }

            if game.make_move(&bestmove).is_err() {
                return Err(GameError::InvalidMove(bestmove));
            }

            moves.push(bestmove);

            // Safety limit
            if moves.len() > 500 {
                break;
            }
        }

        let result = match game.result() {
            Some(r) if r.contains("white") => MatchResult::WhiteWins,
            Some(r) if r.contains("black") => MatchResult::BlackWins,
            _ => MatchResult::Draw,
        };

        Ok(GameResult {
            moves,
            result,
            white_name,
            black_name,
        })
    }
}
```

**Step 2: Update main.rs**

Add module and update match command:

```rust
mod game_runner;

use crate::uci_client::UciClient;
use crate::game_runner::{GameRunner, MatchResult};

// ... in main() ...

fn main() {
    let cli = Cli::parse();
    let config = ArenaConfig::load().unwrap_or_default();

    match cli.command {
        Commands::Match { white, black, games } => {
            let white_path = config
                .get_bot(&white)
                .map(|b| b.path.clone())
                .unwrap_or_else(|_| white.clone().into());
            let black_path = config
                .get_bot(&black)
                .map(|b| b.path.clone())
                .unwrap_or_else(|_| black.clone().into());
            let time_control = config
                .get_bot(&white)
                .map(|b| b.time_control.clone())
                .unwrap_or_else(|_| "movetime 500".to_string());

            println!("Running {} games: {} vs {}", games, white, black);

            let mut white_wins = 0;
            let mut black_wins = 0;
            let mut draws = 0;

            for i in 1..=games {
                let white_client = UciClient::spawn(&white_path).expect("Failed to spawn white");
                let black_client = UciClient::spawn(&black_path).expect("Failed to spawn black");

                let mut runner = GameRunner::new(white_client, black_client, time_control.clone())
                    .expect("Failed to init game");

                match runner.play_game() {
                    Ok(result) => {
                        match result.result {
                            MatchResult::WhiteWins => white_wins += 1,
                            MatchResult::BlackWins => black_wins += 1,
                            MatchResult::Draw => draws += 1,
                        }
                        println!(
                            "Game {}: {:?} ({} moves)",
                            i,
                            result.result,
                            result.moves.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("Game {} error: {}", i, e);
                    }
                }
            }

            println!("\nResults: W:{} D:{} L:{}", white_wins, draws, black_wins);
        }
    }
}
```

**Step 3: Verify it builds**

Run: `cargo build -p bot-arena`
Expected: Build succeeds

**Step 4: Test with actual bots**

Run: `cargo run -p bot-arena -- match ./target/debug/bot-random ./target/debug/bot-random --games 3`
Expected: Runs 3 games, shows results

**Step 5: Commit**

```bash
git add crates/bot-arena/src/game_runner.rs crates/bot-arena/src/main.rs
git commit -m "feat(bot-arena): implement single game execution"
```

---

## Task 5: Add SQLite storage

**Files:**
- Create: `crates/bot-arena/src/storage.rs`
- Modify: `crates/bot-arena/src/main.rs`

**Step 1: Create storage module**

Create `crates/bot-arena/src/storage.rs`:

```rust
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use crate::game_runner::{GameResult, MatchResult};
use chrono::Utc;
use uuid::Uuid;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

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
            "
        )
    }

    pub fn ensure_bot(&self, name: &str, path: Option<&str>) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO bots (id, name, path) VALUES (?1, ?1, ?2)",
            [name, path.unwrap_or("")],
        )?;
        Ok(())
    }

    pub fn save_game(&self, result: &GameResult) -> SqliteResult<String> {
        let id = Uuid::new_v4().to_string();
        let result_str = match result.result {
            MatchResult::WhiteWins => "white",
            MatchResult::BlackWins => "black",
            MatchResult::Draw => "draw",
        };

        self.conn.execute(
            "INSERT INTO games (id, white_bot, black_bot, result, move_count, moves, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &id,
                &result.white_name,
                &result.black_name,
                result_str,
                result.moves.len() as i32,
                result.moves.join(" "),
                Utc::now().to_rfc3339(),
            ),
        )?;

        self.update_stats(&result.white_name, &result.black_name, result.result)?;

        Ok(id)
    }

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
                if result == MatchResult::WhiteWins { 1 } else { 0 },
                if result == MatchResult::Draw { 1 } else { 0 },
                if result == MatchResult::BlackWins { 1 } else { 0 },
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
                if result == MatchResult::BlackWins { 1 } else { 0 },
                if result == MatchResult::Draw { 1 } else { 0 },
                if result == MatchResult::WhiteWins { 1 } else { 0 },
            ),
        )?;

        Ok(())
    }

    pub fn get_stats(&self, bot: &str) -> SqliteResult<(i32, i32, i32, i32)> {
        let mut stmt = self.conn.prepare(
            "SELECT SUM(games), SUM(wins), SUM(draws), SUM(losses)
             FROM bot_stats WHERE bot_id = ?1"
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
```

**Step 2: Update main.rs to use storage**

Add module and integrate:

```rust
mod storage;

use crate::storage::Storage;

// In main(), after results loop:

fn main() {
    // ... existing setup ...

    match cli.command {
        Commands::Match { white, black, games } => {
            // ... existing path resolution ...

            // Create data directory and storage
            std::fs::create_dir_all("data").ok();
            let storage = Storage::open("data/arena.db").expect("Failed to open database");
            storage.ensure_bot(&white, Some(white_path.to_str().unwrap())).ok();
            storage.ensure_bot(&black, Some(black_path.to_str().unwrap())).ok();

            println!("Running {} games: {} vs {}", games, white, black);

            // ... game loop, but add storage.save_game(&result) after each game ...

            for i in 1..=games {
                // ... existing game code ...

                match runner.play_game() {
                    Ok(result) => {
                        storage.save_game(&result).ok();
                        // ... existing result handling ...
                    }
                    // ...
                }
            }

            // Print final stats from database
            if let Ok((g, w, d, l)) = storage.get_stats(&white) {
                println!("\n{} total: {} games, {} wins, {} draws, {} losses", white, g, w, d, l);
            }
        }
    }
}
```

**Step 3: Verify it builds**

Run: `cargo build -p bot-arena`
Expected: Build succeeds

**Step 4: Test storage**

Run: `cargo run -p bot-arena -- match ./target/debug/bot-random ./target/debug/bot-random --games 2`
Then: `ls data/` should show `arena.db`

**Step 5: Commit**

```bash
git add crates/bot-arena/src/storage.rs crates/bot-arena/src/main.rs
git commit -m "feat(bot-arena): add SQLite storage for game results"
```

---

## Task 6: Add PGN output

**Files:**
- Create: `crates/bot-arena/src/pgn.rs`
- Modify: `crates/bot-arena/src/game_runner.rs`
- Modify: `crates/bot-arena/src/main.rs`

**Step 1: Create PGN module**

Create `crates/bot-arena/src/pgn.rs`:

```rust
use crate::game_runner::{GameResult, MatchResult};
use chrono::Utc;
use std::io::Write;
use std::path::Path;

pub fn write_pgn<P: AsRef<Path>>(path: P, result: &GameResult) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;

    let result_str = match result.result {
        MatchResult::WhiteWins => "1-0",
        MatchResult::BlackWins => "0-1",
        MatchResult::Draw => "1/2-1/2",
    };

    writeln!(file, "[Event \"Bot Arena Match\"]")?;
    writeln!(file, "[Site \"local\"]")?;
    writeln!(file, "[Date \"{}\"]", Utc::now().format("%Y.%m.%d"))?;
    writeln!(file, "[White \"{}\"]", result.white_name)?;
    writeln!(file, "[Black \"{}\"]", result.black_name)?;
    writeln!(file, "[Result \"{}\"]", result_str)?;
    writeln!(file)?;

    // Write moves in PGN format (UCI for now, SAN conversion later)
    let mut move_text = String::new();
    for (i, mv) in result.moves.iter().enumerate() {
        if i % 2 == 0 {
            move_text.push_str(&format!("{}. ", i / 2 + 1));
        }
        move_text.push_str(mv);
        move_text.push(' ');
    }
    move_text.push_str(result_str);

    // Wrap at 80 chars
    for chunk in move_text.as_bytes().chunks(80) {
        file.write_all(chunk)?;
        writeln!(file)?;
    }

    Ok(())
}
```

**Step 2: Update main.rs to save PGN**

Add module and save after each game:

```rust
mod pgn;

// In game loop:
match runner.play_game() {
    Ok(result) => {
        let game_id = storage.save_game(&result).unwrap_or_else(|_| Uuid::new_v4().to_string());

        // Save PGN
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let pgn_dir = format!("data/games/{}", date);
        std::fs::create_dir_all(&pgn_dir).ok();
        let pgn_path = format!("{}/{}.pgn", pgn_dir, game_id);
        pgn::write_pgn(&pgn_path, &result).ok();

        // ... existing result handling ...
    }
}
```

**Step 3: Verify it builds**

Run: `cargo build -p bot-arena`
Expected: Build succeeds

**Step 4: Test PGN output**

Run: `cargo run -p bot-arena -- match ./target/debug/bot-random ./target/debug/bot-random --games 1`
Then: `cat data/games/*//*.pgn` should show PGN content

**Step 5: Commit**

```bash
git add crates/bot-arena/src/pgn.rs crates/bot-arena/src/main.rs
git commit -m "feat(bot-arena): add PGN output for games"
```

---

## Task 7: Add JSON output with UCI info

**Files:**
- Create: `crates/bot-arena/src/json_output.rs`
- Modify: `crates/bot-arena/src/uci_client.rs`
- Modify: `crates/bot-arena/src/game_runner.rs`
- Modify: `crates/bot-arena/src/main.rs`

**Step 1: Add SearchInfo struct to uci_client.rs**

Add to `crates/bot-arena/src/uci_client.rs`:

```rust
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SearchInfo {
    pub depth: Option<u32>,
    pub score_cp: Option<i32>,
    pub score_mate: Option<i32>,
    pub nodes: Option<u64>,
    pub time_ms: Option<u64>,
    pub pv: Vec<String>,
}

impl SearchInfo {
    pub fn parse(line: &str) -> Option<Self> {
        if !line.starts_with("info ") {
            return None;
        }

        let mut info = SearchInfo::default();
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 1;

        while i < parts.len() {
            match parts[i] {
                "depth" => {
                    i += 1;
                    info.depth = parts.get(i).and_then(|s| s.parse().ok());
                }
                "score" => {
                    i += 1;
                    match parts.get(i) {
                        Some(&"cp") => {
                            i += 1;
                            info.score_cp = parts.get(i).and_then(|s| s.parse().ok());
                        }
                        Some(&"mate") => {
                            i += 1;
                            info.score_mate = parts.get(i).and_then(|s| s.parse().ok());
                        }
                        _ => {}
                    }
                }
                "nodes" => {
                    i += 1;
                    info.nodes = parts.get(i).and_then(|s| s.parse().ok());
                }
                "time" => {
                    i += 1;
                    info.time_ms = parts.get(i).and_then(|s| s.parse().ok());
                }
                "pv" => {
                    info.pv = parts[i + 1..].iter().map(|s| s.to_string()).collect();
                    break;
                }
                _ => {}
            }
            i += 1;
        }

        if info.depth.is_some() {
            Some(info)
        } else {
            None
        }
    }
}
```

**Step 2: Update go() to capture search info**

Modify `go()` in `uci_client.rs`:

```rust
pub fn go(&mut self, time_control: &str) -> Result<(String, Option<SearchInfo>), UciError> {
    self.send(&format!("go {}", time_control))?;

    let mut last_info: Option<SearchInfo> = None;

    loop {
        let line = self.read_line()?;

        if let Some(info) = SearchInfo::parse(&line) {
            last_info = Some(info);
        }

        if line.starts_with("bestmove ") {
            let bestmove = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .to_string();
            return Ok((bestmove, last_info));
        }
    }
}
```

**Step 3: Update GameResult and runner**

Add to `game_runner.rs`:

```rust
use crate::uci_client::SearchInfo;

#[derive(Debug, Clone, serde::Serialize)]
pub struct MoveRecord {
    pub uci: String,
    pub search_info: Option<SearchInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GameResult {
    pub moves: Vec<MoveRecord>,
    pub result: MatchResult,
    pub white_name: String,
    pub black_name: String,
}
```

Update `play_game()` to capture search info with each move.

**Step 4: Create JSON output module**

Create `crates/bot-arena/src/json_output.rs`:

```rust
use crate::game_runner::GameResult;
use chrono::Utc;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct GameJson<'a> {
    id: &'a str,
    white: &'a str,
    black: &'a str,
    result: &'a str,
    moves: &'a [crate::game_runner::MoveRecord],
    created_at: String,
}

pub fn write_json<P: AsRef<Path>>(path: P, id: &str, result: &GameResult) -> std::io::Result<()> {
    let result_str = match result.result {
        crate::game_runner::MatchResult::WhiteWins => "white",
        crate::game_runner::MatchResult::BlackWins => "black",
        crate::game_runner::MatchResult::Draw => "draw",
    };

    let json = GameJson {
        id,
        white: &result.white_name,
        black: &result.black_name,
        result: result_str,
        moves: &result.moves,
        created_at: Utc::now().to_rfc3339(),
    };

    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &json)?;
    Ok(())
}
```

**Step 5: Update main.rs to save JSON**

Add JSON save after PGN:

```rust
mod json_output;

// After PGN save:
let json_path = format!("{}/{}.json", pgn_dir, game_id);
json_output::write_json(&json_path, &game_id, &result).ok();
```

**Step 6: Verify and commit**

Run: `cargo build -p bot-arena`
Test: `cargo run -p bot-arena -- match ./target/debug/bot-minimax ./target/debug/bot-random --games 1`
Check: `cat data/games/*/*.json` should show search info

```bash
git add crates/bot-arena/src/
git commit -m "feat(bot-arena): add JSON output with UCI search info"
```

---

## Task 8: Add preset support

**Files:**
- Modify: `crates/bot-arena/src/config.rs`
- Modify: `crates/bot-arena/src/main.rs`
- Create: `arena.toml` (example config)

**Step 1: Add preset CLI argument**

Update `Commands::Match` in `main.rs`:

```rust
Commands::Match {
    white: String,
    black: String,
    #[arg(short, long, default_value = "10")]
    games: u32,
    #[arg(short, long)]
    preset: Option<String>,
},
```

**Step 2: Apply preset in main**

```rust
Commands::Match { white, black, games, preset } => {
    let (games, time_control) = if let Some(preset_name) = preset {
        if let Some(p) = config.presets.get(&preset_name) {
            (p.games, p.time_control.clone())
        } else {
            eprintln!("Unknown preset: {}", preset_name);
            return;
        }
    } else {
        (games, "movetime 500".to_string())
    };
    // ... rest of match logic ...
}
```

**Step 3: Create example arena.toml**

Create `arena.toml` in project root:

```toml
[bots.minimax]
path = "./target/release/bot-minimax"
time_control = "movetime 500"

[bots.random]
path = "./target/release/bot-random"

[presets.quick]
games = 10
time_control = "movetime 100"

[presets.standard]
games = 100
time_control = "movetime 500"

[presets.thorough]
games = 1000
time_control = "movetime 500"
```

**Step 4: Verify and commit**

Run: `cargo run -p bot-arena -- match minimax random --preset quick`
Expected: Uses preset settings

```bash
git add crates/bot-arena/src/main.rs arena.toml
git commit -m "feat(bot-arena): add preset support"
```

---

## Summary

After completing all 8 tasks, you will have a working `bot-arena` CLI that can:

1. Run matches between any two bots via UCI
2. Load bot configurations and presets from `arena.toml`
3. Save game results to SQLite for querying
4. Output PGN files for sharing/analysis
5. Output JSON files with full UCI search info

**Next phases (separate plans):**
- Phase 2: Analysis (Stockfish integration, move quality)
- Phase 3: Opening database
- Phase 4: Shared UI libraries
- Phase 5-7: Comparison UI
