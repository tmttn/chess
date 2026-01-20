# Bot Arena Phase 2: Analysis Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Stockfish integration and move quality analysis to classify each move as Best/Excellent/Good/Inaccuracy/Mistake/Blunder.

**Architecture:** Create a `chess-analysis` crate that wraps Stockfish for position evaluation, then integrate it with bot-arena to analyze games and classify move quality.

**Tech Stack:** Rust, Stockfish (external UCI engine), existing chess-core/chess-engine crates

**Note:** Phase 1 already captures bot self-evaluation (UCI info with depth, score, nodes, pv) in JSON output. Phase 2 adds Stockfish evaluation to compare against.

---

## Task 1: Create chess-analysis crate scaffold

**Files:**
- Create: `crates/chess-analysis/Cargo.toml`
- Create: `crates/chess-analysis/src/lib.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Add crate to workspace**

In `Cargo.toml` (root), add to members:

```toml
members = [
    # ... existing members ...
    "crates/chess-analysis",
]
```

**Step 2: Create Cargo.toml**

Create `crates/chess-analysis/Cargo.toml`:

```toml
[package]
name = "chess-analysis"
version = "0.1.0"
edition = "2021"
description = "Chess position analysis with Stockfish integration"

[dependencies]
chess-core = { path = "../chess-core" }
chess-engine = { path = "../chess-engine" }
thiserror = "1"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
tempfile = "3"
```

**Step 3: Create lib.rs with module structure**

Create `crates/chess-analysis/src/lib.rs`:

```rust
//! Chess position analysis with Stockfish integration.
//!
//! Provides move quality classification and game analysis.

pub mod engine;
pub mod evaluation;
pub mod quality;

pub use engine::AnalysisEngine;
pub use evaluation::Evaluation;
pub use quality::{MoveQuality, MoveAnalysis, GameAnalysis, PlayerStats};
```

**Step 4: Verify it builds**

Run: `cargo build -p chess-analysis`

**Step 5: Commit**

```bash
git add crates/chess-analysis Cargo.toml
git commit -m "feat(chess-analysis): scaffold analysis crate"
```

---

## Task 2: Add Evaluation types

**Files:**
- Create: `crates/chess-analysis/src/evaluation.rs`

**Step 1: Create evaluation module**

Create `crates/chess-analysis/src/evaluation.rs`:

```rust
//! Chess position evaluation types.

use serde::{Deserialize, Serialize};

/// A chess position evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Evaluation {
    /// Centipawn evaluation (positive = white advantage).
    Centipawn(i32),
    /// Mate in N moves (positive = white wins, negative = black wins).
    Mate(i32),
}

impl Evaluation {
    /// Parses evaluation from UCI info score.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::Evaluation;
    ///
    /// let eval = Evaluation::from_uci_score(Some(35), None);
    /// assert_eq!(eval, Some(Evaluation::Centipawn(35)));
    ///
    /// let mate = Evaluation::from_uci_score(None, Some(3));
    /// assert_eq!(mate, Some(Evaluation::Mate(3)));
    /// ```
    pub fn from_uci_score(cp: Option<i32>, mate: Option<i32>) -> Option<Self> {
        if let Some(m) = mate {
            Some(Evaluation::Mate(m))
        } else {
            cp.map(Evaluation::Centipawn)
        }
    }

    /// Returns the centipawn value, converting mate to a large value.
    ///
    /// Mate scores are converted to ±10000 (capped).
    pub fn to_centipawns(&self) -> i32 {
        match self {
            Evaluation::Centipawn(cp) => *cp,
            Evaluation::Mate(n) => {
                if *n > 0 {
                    10000 - (*n * 10) // Closer mate = higher score
                } else {
                    -10000 - (*n * 10) // Closer mate = lower score
                }
            }
        }
    }

    /// Returns true if this evaluation is better for white than the other.
    pub fn is_better_for_white(&self, other: &Evaluation) -> bool {
        self.to_centipawns() > other.to_centipawns()
    }

    /// Returns true if this evaluation is better for black than the other.
    pub fn is_better_for_black(&self, other: &Evaluation) -> bool {
        self.to_centipawns() < other.to_centipawns()
    }
}

impl std::fmt::Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::Centipawn(cp) => {
                let sign = if *cp >= 0 { "+" } else { "" };
                write!(f, "{}{:.2}", sign, *cp as f32 / 100.0)
            }
            Evaluation::Mate(n) => {
                if *n > 0 {
                    write!(f, "#{}",  n)
                } else {
                    write!(f, "#-{}", -n)
                }
            }
        }
    }
}
```

**Step 2: Add tests**

Add to `evaluation.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_uci_score_centipawn() {
        let eval = Evaluation::from_uci_score(Some(35), None);
        assert_eq!(eval, Some(Evaluation::Centipawn(35)));
    }

    #[test]
    fn test_from_uci_score_mate() {
        let eval = Evaluation::from_uci_score(Some(100), Some(3));
        assert_eq!(eval, Some(Evaluation::Mate(3))); // Mate takes precedence
    }

    #[test]
    fn test_from_uci_score_none() {
        let eval = Evaluation::from_uci_score(None, None);
        assert_eq!(eval, None);
    }

    #[test]
    fn test_to_centipawns() {
        assert_eq!(Evaluation::Centipawn(50).to_centipawns(), 50);
        assert_eq!(Evaluation::Centipawn(-100).to_centipawns(), -100);
        assert!(Evaluation::Mate(1).to_centipawns() > 9000);
        assert!(Evaluation::Mate(-1).to_centipawns() < -9000);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Evaluation::Centipawn(35)), "+0.35");
        assert_eq!(format!("{}", Evaluation::Centipawn(-150)), "-1.50");
        assert_eq!(format!("{}", Evaluation::Mate(3)), "#3");
        assert_eq!(format!("{}", Evaluation::Mate(-2)), "#-2");
    }

    #[test]
    fn test_is_better_for_white() {
        let good = Evaluation::Centipawn(100);
        let bad = Evaluation::Centipawn(-50);
        assert!(good.is_better_for_white(&bad));
        assert!(!bad.is_better_for_white(&good));
    }
}
```

**Step 3: Verify and commit**

Run: `cargo test -p chess-analysis`

```bash
git add crates/chess-analysis/src/evaluation.rs
git commit -m "feat(chess-analysis): add Evaluation types"
```

---

## Task 3: Add MoveQuality classification

**Files:**
- Create: `crates/chess-analysis/src/quality.rs`

**Step 1: Create quality module**

Create `crates/chess-analysis/src/quality.rs`:

```rust
//! Move quality classification and game analysis.

use crate::evaluation::Evaluation;
use serde::{Deserialize, Serialize};

/// Classification of move quality based on centipawn loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveQuality {
    /// Matches engine's top choice.
    Best,
    /// Within 10cp of best move.
    Excellent,
    /// Within 30cp of best move.
    Good,
    /// 30-100cp worse than best (minor mistake).
    Inaccuracy,
    /// 100-300cp worse than best (significant error).
    Mistake,
    /// More than 300cp worse than best (game-changing error).
    Blunder,
    /// Forced move (only legal move or opening book).
    Forced,
}

impl MoveQuality {
    /// Classifies a move based on centipawn loss.
    ///
    /// # Arguments
    ///
    /// * `cp_loss` - The centipawn loss (always positive or zero).
    /// * `is_forced` - Whether this move was forced (only legal move or book).
    ///
    /// # Examples
    ///
    /// ```
    /// use chess_analysis::MoveQuality;
    ///
    /// assert_eq!(MoveQuality::from_cp_loss(0, false), MoveQuality::Best);
    /// assert_eq!(MoveQuality::from_cp_loss(5, false), MoveQuality::Excellent);
    /// assert_eq!(MoveQuality::from_cp_loss(50, false), MoveQuality::Inaccuracy);
    /// assert_eq!(MoveQuality::from_cp_loss(200, false), MoveQuality::Mistake);
    /// assert_eq!(MoveQuality::from_cp_loss(500, false), MoveQuality::Blunder);
    /// assert_eq!(MoveQuality::from_cp_loss(500, true), MoveQuality::Forced);
    /// ```
    pub fn from_cp_loss(cp_loss: i32, is_forced: bool) -> Self {
        if is_forced {
            return MoveQuality::Forced;
        }

        match cp_loss {
            0 => MoveQuality::Best,
            1..=10 => MoveQuality::Excellent,
            11..=30 => MoveQuality::Good,
            31..=100 => MoveQuality::Inaccuracy,
            101..=300 => MoveQuality::Mistake,
            _ => MoveQuality::Blunder,
        }
    }

    /// Returns true if this is a negative classification (inaccuracy or worse).
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            MoveQuality::Inaccuracy | MoveQuality::Mistake | MoveQuality::Blunder
        )
    }
}

/// Analysis of a single move.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAnalysis {
    /// The move in UCI format.
    pub uci: String,
    /// The move in SAN format (if available).
    pub san: Option<String>,
    /// Quality classification.
    pub quality: MoveQuality,

    // Bot's own evaluation (from UCI info during game)
    /// Bot's evaluation of the position.
    pub bot_eval: Option<Evaluation>,
    /// Bot's search depth.
    pub bot_depth: Option<u32>,
    /// Bot's nodes searched.
    pub bot_nodes: Option<u64>,
    /// Bot's thinking time in milliseconds.
    pub bot_time_ms: Option<u64>,
    /// Bot's principal variation.
    pub bot_pv: Vec<String>,

    // Stockfish evaluation (post-game analysis)
    /// Engine's evaluation of the position before the move.
    pub engine_eval_before: Option<Evaluation>,
    /// Engine's evaluation after the move.
    pub engine_eval_after: Option<Evaluation>,
    /// Engine's best move for this position.
    pub engine_best_move: Option<String>,
    /// Engine's principal variation.
    pub engine_pv: Vec<String>,
    /// Centipawn loss compared to best move.
    pub centipawn_loss: Option<i32>,
}

/// Aggregate statistics for a player in a game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Average centipawn loss per move.
    pub avg_centipawn_loss: f32,
    /// Number of blunders.
    pub blunders: u32,
    /// Number of mistakes.
    pub mistakes: u32,
    /// Number of inaccuracies.
    pub inaccuracies: u32,
    /// Average search depth.
    pub avg_depth: f32,
    /// Average nodes searched.
    pub avg_nodes: u64,
    /// Average thinking time in milliseconds.
    pub avg_time_ms: u64,
    /// Accuracy percentage (0-100).
    pub accuracy_percent: f32,
}

impl PlayerStats {
    /// Calculates stats from a list of move analyses.
    pub fn from_moves(moves: &[MoveAnalysis]) -> Self {
        if moves.is_empty() {
            return Self::default();
        }

        let mut total_cp_loss = 0i32;
        let mut total_depth = 0u32;
        let mut total_nodes = 0u64;
        let mut total_time = 0u64;
        let mut blunders = 0u32;
        let mut mistakes = 0u32;
        let mut inaccuracies = 0u32;
        let mut depth_count = 0u32;
        let mut nodes_count = 0u32;
        let mut time_count = 0u32;

        for m in moves {
            if let Some(cp_loss) = m.centipawn_loss {
                total_cp_loss += cp_loss;
            }
            if let Some(d) = m.bot_depth {
                total_depth += d;
                depth_count += 1;
            }
            if let Some(n) = m.bot_nodes {
                total_nodes += n;
                nodes_count += 1;
            }
            if let Some(t) = m.bot_time_ms {
                total_time += t;
                time_count += 1;
            }
            match m.quality {
                MoveQuality::Blunder => blunders += 1,
                MoveQuality::Mistake => mistakes += 1,
                MoveQuality::Inaccuracy => inaccuracies += 1,
                _ => {}
            }
        }

        let move_count = moves.len() as f32;
        let avg_cp_loss = total_cp_loss as f32 / move_count;

        // Accuracy formula: 100 * e^(-0.005 * avg_cp_loss)
        // This gives ~100% at 0 cp loss, ~60% at 100 cp loss, ~13% at 400 cp loss
        let accuracy = 100.0 * (-0.005 * avg_cp_loss).exp();

        Self {
            avg_centipawn_loss: avg_cp_loss,
            blunders,
            mistakes,
            inaccuracies,
            avg_depth: if depth_count > 0 {
                total_depth as f32 / depth_count as f32
            } else {
                0.0
            },
            avg_nodes: if nodes_count > 0 {
                total_nodes / nodes_count as u64
            } else {
                0
            },
            avg_time_ms: if time_count > 0 {
                total_time / time_count as u64
            } else {
                0
            },
            accuracy_percent: accuracy.min(100.0).max(0.0),
        }
    }
}

/// Complete analysis of a chess game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalysis {
    /// Unique game identifier.
    pub game_id: String,
    /// White player/bot name.
    pub white_bot: String,
    /// Black player/bot name.
    pub black_bot: String,
    /// Opening name (if identified).
    pub opening: Option<String>,
    /// Game result.
    pub result: String,
    /// All analyzed moves.
    pub moves: Vec<MoveAnalysis>,
    /// White's aggregate statistics.
    pub white_stats: PlayerStats,
    /// Black's aggregate statistics.
    pub black_stats: PlayerStats,
}
```

**Step 2: Add tests**

Add to bottom of `quality.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_quality_from_cp_loss() {
        assert_eq!(MoveQuality::from_cp_loss(0, false), MoveQuality::Best);
        assert_eq!(MoveQuality::from_cp_loss(5, false), MoveQuality::Excellent);
        assert_eq!(MoveQuality::from_cp_loss(10, false), MoveQuality::Excellent);
        assert_eq!(MoveQuality::from_cp_loss(15, false), MoveQuality::Good);
        assert_eq!(MoveQuality::from_cp_loss(30, false), MoveQuality::Good);
        assert_eq!(MoveQuality::from_cp_loss(50, false), MoveQuality::Inaccuracy);
        assert_eq!(MoveQuality::from_cp_loss(150, false), MoveQuality::Mistake);
        assert_eq!(MoveQuality::from_cp_loss(400, false), MoveQuality::Blunder);
    }

    #[test]
    fn test_move_quality_forced() {
        assert_eq!(MoveQuality::from_cp_loss(500, true), MoveQuality::Forced);
        assert_eq!(MoveQuality::from_cp_loss(0, true), MoveQuality::Forced);
    }

    #[test]
    fn test_move_quality_is_negative() {
        assert!(!MoveQuality::Best.is_negative());
        assert!(!MoveQuality::Excellent.is_negative());
        assert!(!MoveQuality::Good.is_negative());
        assert!(MoveQuality::Inaccuracy.is_negative());
        assert!(MoveQuality::Mistake.is_negative());
        assert!(MoveQuality::Blunder.is_negative());
        assert!(!MoveQuality::Forced.is_negative());
    }

    #[test]
    fn test_player_stats_from_empty_moves() {
        let stats = PlayerStats::from_moves(&[]);
        assert_eq!(stats.blunders, 0);
        assert_eq!(stats.avg_centipawn_loss, 0.0);
    }

    #[test]
    fn test_player_stats_accuracy() {
        // Create moves with known cp loss
        let moves = vec![
            MoveAnalysis {
                uci: "e2e4".to_string(),
                san: Some("e4".to_string()),
                quality: MoveQuality::Best,
                bot_eval: None,
                bot_depth: Some(20),
                bot_nodes: Some(1000000),
                bot_time_ms: Some(500),
                bot_pv: vec![],
                engine_eval_before: None,
                engine_eval_after: None,
                engine_best_move: None,
                engine_pv: vec![],
                centipawn_loss: Some(0),
            },
            MoveAnalysis {
                uci: "d2d4".to_string(),
                san: Some("d4".to_string()),
                quality: MoveQuality::Inaccuracy,
                bot_eval: None,
                bot_depth: Some(18),
                bot_nodes: Some(800000),
                bot_time_ms: Some(450),
                bot_pv: vec![],
                engine_eval_before: None,
                engine_eval_after: None,
                engine_best_move: None,
                engine_pv: vec![],
                centipawn_loss: Some(50),
            },
        ];

        let stats = PlayerStats::from_moves(&moves);
        assert_eq!(stats.inaccuracies, 1);
        assert_eq!(stats.avg_centipawn_loss, 25.0);
        assert!(stats.accuracy_percent > 80.0); // ~88% for 25cp avg loss
        assert_eq!(stats.avg_depth, 19.0);
    }
}
```

**Step 3: Verify and commit**

Run: `cargo test -p chess-analysis`

```bash
git add crates/chess-analysis/src/quality.rs
git commit -m "feat(chess-analysis): add MoveQuality classification and PlayerStats"
```

---

## Task 4: Add Stockfish engine wrapper

**Files:**
- Create: `crates/chess-analysis/src/engine.rs`

**Step 1: Create engine module**

Create `crates/chess-analysis/src/engine.rs`:

```rust
//! Stockfish engine wrapper for position analysis.

use crate::evaluation::Evaluation;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use thiserror::Error;

/// Errors that can occur during engine operations.
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Failed to spawn engine: {0}")]
    SpawnError(#[from] std::io::Error),
    #[error("Engine not found at path: {0}")]
    NotFound(String),
    #[error("Engine initialization failed")]
    InitFailed,
    #[error("Invalid engine response: {0}")]
    InvalidResponse(String),
}

/// Result of analyzing a position.
#[derive(Debug, Clone)]
pub struct PositionAnalysis {
    /// Best move in UCI format.
    pub best_move: String,
    /// Position evaluation.
    pub evaluation: Evaluation,
    /// Search depth reached.
    pub depth: u32,
    /// Nodes searched.
    pub nodes: u64,
    /// Principal variation (best line).
    pub pv: Vec<String>,
}

/// Wrapper for UCI-compatible analysis engine (e.g., Stockfish).
pub struct AnalysisEngine {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    name: String,
}

impl AnalysisEngine {
    /// Spawns a new analysis engine from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the engine executable (e.g., "/usr/local/bin/stockfish").
    ///
    /// # Errors
    ///
    /// Returns an error if the engine cannot be spawned or initialized.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, EngineError> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(EngineError::NotFound(path_ref.display().to_string()));
        }

        let mut process = Command::new(path_ref)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = process.stdin.take().unwrap();
        let stdout = BufReader::new(process.stdout.take().unwrap());

        let mut engine = Self {
            process,
            stdin,
            stdout,
            name: String::new(),
        };

        engine.init()?;
        Ok(engine)
    }

    fn send(&mut self, cmd: &str) -> Result<(), EngineError> {
        writeln!(self.stdin, "{}", cmd)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_line(&mut self) -> Result<String, EngineError> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        Ok(line.trim().to_string())
    }

    fn init(&mut self) -> Result<(), EngineError> {
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

        // Set some sensible defaults
        self.send("setoption name Threads value 1")?;
        self.send("setoption name Hash value 128")?;

        Ok(())
    }

    /// Returns the engine name (e.g., "Stockfish 16").
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Analyzes a position given by FEN.
    ///
    /// # Arguments
    ///
    /// * `fen` - The position in FEN notation.
    /// * `depth` - Search depth.
    ///
    /// # Returns
    ///
    /// Analysis results including best move, evaluation, and PV.
    pub fn analyze_fen(&mut self, fen: &str, depth: u32) -> Result<PositionAnalysis, EngineError> {
        self.send(&format!("position fen {}", fen))?;
        self.analyze_current_position(depth)
    }

    /// Analyzes a position given by move sequence from start.
    ///
    /// # Arguments
    ///
    /// * `moves` - UCI moves from starting position.
    /// * `depth` - Search depth.
    pub fn analyze_moves(
        &mut self,
        moves: &[String],
        depth: u32,
    ) -> Result<PositionAnalysis, EngineError> {
        if moves.is_empty() {
            self.send("position startpos")?;
        } else {
            self.send(&format!("position startpos moves {}", moves.join(" ")))?;
        }
        self.analyze_current_position(depth)
    }

    fn analyze_current_position(&mut self, depth: u32) -> Result<PositionAnalysis, EngineError> {
        self.send(&format!("go depth {}", depth))?;

        let mut best_move = String::new();
        let mut eval = Evaluation::Centipawn(0);
        let mut reached_depth = 0u32;
        let mut nodes = 0u64;
        let mut pv = Vec::new();

        loop {
            let line = self.read_line()?;

            if line.starts_with("info ") && line.contains(" depth ") {
                // Parse info line
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut i = 1;
                while i < parts.len() {
                    match parts[i] {
                        "depth" => {
                            i += 1;
                            if let Some(d) = parts.get(i).and_then(|s| s.parse().ok()) {
                                reached_depth = d;
                            }
                        }
                        "score" => {
                            i += 1;
                            match parts.get(i) {
                                Some(&"cp") => {
                                    i += 1;
                                    if let Some(cp) = parts.get(i).and_then(|s| s.parse().ok()) {
                                        eval = Evaluation::Centipawn(cp);
                                    }
                                }
                                Some(&"mate") => {
                                    i += 1;
                                    if let Some(m) = parts.get(i).and_then(|s| s.parse().ok()) {
                                        eval = Evaluation::Mate(m);
                                    }
                                }
                                _ => {}
                            }
                        }
                        "nodes" => {
                            i += 1;
                            if let Some(n) = parts.get(i).and_then(|s| s.parse().ok()) {
                                nodes = n;
                            }
                        }
                        "pv" => {
                            pv = parts[i + 1..].iter().map(|s| s.to_string()).collect();
                            break;
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }

            if line.starts_with("bestmove ") {
                best_move = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
                break;
            }
        }

        Ok(PositionAnalysis {
            best_move,
            evaluation: eval,
            depth: reached_depth,
            nodes,
            pv,
        })
    }

    /// Stops any ongoing search.
    pub fn stop(&mut self) -> Result<(), EngineError> {
        self.send("stop")
    }

    /// Clears the hash table.
    pub fn clear_hash(&mut self) -> Result<(), EngineError> {
        self.send("ucinewgame")?;
        self.send("isready")?;
        loop {
            let line = self.read_line()?;
            if line == "readyok" {
                break;
            }
        }
        Ok(())
    }
}

impl Drop for AnalysisEngine {
    fn drop(&mut self) {
        let _ = self.send("quit");
        let _ = self.process.kill();
    }
}
```

**Step 2: Add tests**

Add to `engine.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_not_found() {
        let result = AnalysisEngine::new("/nonexistent/path/to/engine");
        assert!(matches!(result, Err(EngineError::NotFound(_))));
    }

    #[test]
    fn test_position_analysis_clone() {
        let analysis = PositionAnalysis {
            best_move: "e2e4".to_string(),
            evaluation: Evaluation::Centipawn(35),
            depth: 20,
            nodes: 1000000,
            pv: vec!["e2e4".to_string(), "e7e5".to_string()],
        };
        let cloned = analysis.clone();
        assert_eq!(cloned.best_move, "e2e4");
    }

    // Note: Integration tests with actual Stockfish would go in tests/ directory
    // as they require Stockfish to be installed.
}
```

**Step 3: Verify and commit**

Run: `cargo test -p chess-analysis`

```bash
git add crates/chess-analysis/src/engine.rs
git commit -m "feat(chess-analysis): add Stockfish engine wrapper"
```

---

## Task 5: Add game analyzer

**Files:**
- Create: `crates/chess-analysis/src/analyzer.rs`
- Modify: `crates/chess-analysis/src/lib.rs`

**Step 1: Create analyzer module**

Create `crates/chess-analysis/src/analyzer.rs`:

```rust
//! Game analysis combining bot evaluation with Stockfish analysis.

use crate::engine::{AnalysisEngine, EngineError};
use crate::evaluation::Evaluation;
use crate::quality::{GameAnalysis, MoveAnalysis, MoveQuality, PlayerStats};
use chess_engine::Game;
use thiserror::Error;

/// Errors that can occur during game analysis.
#[derive(Error, Debug)]
pub enum AnalyzerError {
    #[error("Engine error: {0}")]
    Engine(#[from] EngineError),
    #[error("Invalid game data: {0}")]
    InvalidGame(String),
}

/// Input data for a move to be analyzed.
#[derive(Debug, Clone)]
pub struct MoveInput {
    pub uci: String,
    pub bot_eval_cp: Option<i32>,
    pub bot_eval_mate: Option<i32>,
    pub bot_depth: Option<u32>,
    pub bot_nodes: Option<u64>,
    pub bot_time_ms: Option<u64>,
    pub bot_pv: Vec<String>,
}

/// Configuration for game analysis.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Search depth for Stockfish analysis.
    pub depth: u32,
    /// Number of opening moves to skip analysis (mark as Forced).
    pub opening_book_moves: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            depth: 15,
            opening_book_moves: 0,
        }
    }
}

/// Analyzes chess games using Stockfish.
pub struct GameAnalyzer {
    engine: AnalysisEngine,
    config: AnalysisConfig,
}

impl GameAnalyzer {
    /// Creates a new analyzer with the given Stockfish path.
    pub fn new<P: AsRef<std::path::Path>>(
        stockfish_path: P,
        config: AnalysisConfig,
    ) -> Result<Self, AnalyzerError> {
        let engine = AnalysisEngine::new(stockfish_path)?;
        Ok(Self { engine, config })
    }

    /// Analyzes a complete game.
    ///
    /// # Arguments
    ///
    /// * `game_id` - Unique identifier for the game.
    /// * `white_name` - Name of white player/bot.
    /// * `black_name` - Name of black player/bot.
    /// * `moves` - List of moves with bot evaluation data.
    /// * `result` - Game result (e.g., "white", "black", "draw").
    ///
    /// # Returns
    ///
    /// Complete game analysis with move classifications and statistics.
    pub fn analyze_game(
        &mut self,
        game_id: &str,
        white_name: &str,
        black_name: &str,
        moves: &[MoveInput],
        result: &str,
    ) -> Result<GameAnalysis, AnalyzerError> {
        self.engine.clear_hash()?;

        let mut analyzed_moves = Vec::with_capacity(moves.len());
        let mut game = Game::new();
        let mut move_history: Vec<String> = Vec::new();

        for (i, move_input) in moves.iter().enumerate() {
            let is_opening = i < self.config.opening_book_moves * 2; // *2 because each player moves
            let is_white_move = i % 2 == 0;

            // Get engine evaluation of position BEFORE the move
            let pre_analysis = if !is_opening {
                Some(self.engine.analyze_moves(&move_history, self.config.depth)?)
            } else {
                None
            };

            // Make the move
            move_history.push(move_input.uci.clone());

            // Get engine evaluation AFTER the move (for centipawn loss calculation)
            let post_analysis = if !is_opening {
                Some(self.engine.analyze_moves(&move_history, self.config.depth)?)
            } else {
                None
            };

            // Calculate centipawn loss
            let (cp_loss, quality) = if is_opening {
                (None, MoveQuality::Forced)
            } else if let (Some(pre), Some(post)) = (&pre_analysis, &post_analysis) {
                // CP loss = difference between best move eval and actual move eval
                // Need to flip sign for black's perspective
                let best_eval = pre.evaluation.to_centipawns();
                let actual_eval = -post.evaluation.to_centipawns(); // Flip because it's opponent's turn

                let loss = if is_white_move {
                    best_eval - actual_eval
                } else {
                    actual_eval - best_eval
                };

                let loss = loss.max(0); // CP loss is always positive
                let is_only_legal = self.is_only_legal_move(&game, &move_input.uci);
                (Some(loss), MoveQuality::from_cp_loss(loss, is_only_legal))
            } else {
                (None, MoveQuality::Forced)
            };

            // Get SAN notation
            let san = game.move_to_san(&move_input.uci).ok();

            // Actually make the move in our game state
            let _ = game.make_move(&move_input.uci);

            analyzed_moves.push(MoveAnalysis {
                uci: move_input.uci.clone(),
                san,
                quality,
                bot_eval: Evaluation::from_uci_score(move_input.bot_eval_cp, move_input.bot_eval_mate),
                bot_depth: move_input.bot_depth,
                bot_nodes: move_input.bot_nodes,
                bot_time_ms: move_input.bot_time_ms,
                bot_pv: move_input.bot_pv.clone(),
                engine_eval_before: pre_analysis.as_ref().map(|a| a.evaluation.clone()),
                engine_eval_after: post_analysis.as_ref().map(|a| a.evaluation.clone()),
                engine_best_move: pre_analysis.as_ref().map(|a| a.best_move.clone()),
                engine_pv: pre_analysis
                    .as_ref()
                    .map(|a| a.pv.clone())
                    .unwrap_or_default(),
                centipawn_loss: cp_loss,
            });
        }

        // Calculate stats for each player
        let white_moves: Vec<_> = analyzed_moves.iter().step_by(2).cloned().collect();
        let black_moves: Vec<_> = analyzed_moves.iter().skip(1).step_by(2).cloned().collect();

        let white_stats = PlayerStats::from_moves(&white_moves);
        let black_stats = PlayerStats::from_moves(&black_moves);

        Ok(GameAnalysis {
            game_id: game_id.to_string(),
            white_bot: white_name.to_string(),
            black_bot: black_name.to_string(),
            opening: None, // Would be filled in by opening detection
            result: result.to_string(),
            moves: analyzed_moves,
            white_stats,
            black_stats,
        })
    }

    fn is_only_legal_move(&self, game: &Game, uci: &str) -> bool {
        let legal_moves = game.legal_moves();
        legal_moves.len() == 1 && legal_moves.first().map(|m| m.as_str()) == Some(uci)
    }
}
```

**Step 2: Update lib.rs**

Add to `lib.rs`:

```rust
pub mod analyzer;
pub use analyzer::{GameAnalyzer, AnalysisConfig, MoveInput, AnalyzerError};
```

**Step 3: Add tests**

Add to `analyzer.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert_eq!(config.depth, 15);
        assert_eq!(config.opening_book_moves, 0);
    }

    #[test]
    fn test_move_input_clone() {
        let input = MoveInput {
            uci: "e2e4".to_string(),
            bot_eval_cp: Some(35),
            bot_eval_mate: None,
            bot_depth: Some(20),
            bot_nodes: Some(1000000),
            bot_time_ms: Some(500),
            bot_pv: vec!["e2e4".to_string()],
        };
        let cloned = input.clone();
        assert_eq!(cloned.uci, "e2e4");
    }
}
```

**Step 4: Verify and commit**

Run: `cargo test -p chess-analysis`

```bash
git add crates/chess-analysis/src/analyzer.rs crates/chess-analysis/src/lib.rs
git commit -m "feat(chess-analysis): add GameAnalyzer for move quality analysis"
```

---

## Task 6: Add analyze command to bot-arena CLI

**Files:**
- Modify: `crates/bot-arena/Cargo.toml`
- Modify: `crates/bot-arena/src/main.rs`

**Step 1: Add chess-analysis dependency**

In `crates/bot-arena/Cargo.toml`, add:

```toml
chess-analysis = { path = "../chess-analysis" }
```

**Step 2: Add analyze command**

Update `main.rs` to add the analyze command:

```rust
use chess_analysis::{AnalysisConfig, GameAnalyzer, MoveInput};

#[derive(Subcommand)]
enum Commands {
    /// Run a match between two bots
    Match { /* existing fields */ },

    /// Analyze a completed game with Stockfish
    Analyze {
        /// Game ID to analyze
        #[arg(short, long)]
        game_id: String,

        /// Path to Stockfish executable
        #[arg(long, default_value = "stockfish")]
        engine: String,

        /// Analysis depth
        #[arg(short, long, default_value = "15")]
        depth: u32,

        /// Number of opening book moves to skip
        #[arg(long, default_value = "0")]
        book_moves: usize,
    },
}

// In main(), add handler:
Commands::Analyze {
    game_id,
    engine,
    depth,
    book_moves,
} => {
    // Load game from JSON
    let json_path = find_game_json(&game_id);
    if json_path.is_none() {
        eprintln!("Game not found: {}", game_id);
        std::process::exit(1);
    }

    let game_data: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(json_path.unwrap()).unwrap()).unwrap();

    // Convert to MoveInput
    let moves: Vec<MoveInput> = game_data["moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| MoveInput {
            uci: m["uci"].as_str().unwrap().to_string(),
            bot_eval_cp: m["search_info"]["score_cp"].as_i64().map(|v| v as i32),
            bot_eval_mate: m["search_info"]["score_mate"].as_i64().map(|v| v as i32),
            bot_depth: m["search_info"]["depth"].as_u64().map(|v| v as u32),
            bot_nodes: m["search_info"]["nodes"].as_u64(),
            bot_time_ms: m["search_info"]["time_ms"].as_u64(),
            bot_pv: m["search_info"]["pv"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        })
        .collect();

    let config = AnalysisConfig {
        depth,
        opening_book_moves: book_moves,
    };

    println!("Analyzing game {} with {} at depth {}...", game_id, engine, depth);

    let mut analyzer = GameAnalyzer::new(&engine, config).expect("Failed to start Stockfish");
    let analysis = analyzer
        .analyze_game(
            &game_id,
            game_data["white"].as_str().unwrap(),
            game_data["black"].as_str().unwrap(),
            &moves,
            game_data["result"].as_str().unwrap(),
        )
        .expect("Analysis failed");

    // Output results
    println!("\n=== Analysis Results ===\n");
    println!("White ({}): {:.1}% accuracy", analysis.white_bot, analysis.white_stats.accuracy_percent);
    println!("  Blunders: {}, Mistakes: {}, Inaccuracies: {}",
        analysis.white_stats.blunders, analysis.white_stats.mistakes, analysis.white_stats.inaccuracies);
    println!("  Avg CP loss: {:.1}", analysis.white_stats.avg_centipawn_loss);

    println!("\nBlack ({}): {:.1}% accuracy", analysis.black_bot, analysis.black_stats.accuracy_percent);
    println!("  Blunders: {}, Mistakes: {}, Inaccuracies: {}",
        analysis.black_stats.blunders, analysis.black_stats.mistakes, analysis.black_stats.inaccuracies);
    println!("  Avg CP loss: {:.1}", analysis.black_stats.avg_centipawn_loss);

    // Save analysis to JSON
    let analysis_path = format!("data/analysis/{}.json", game_id);
    std::fs::create_dir_all("data/analysis").ok();
    let file = std::fs::File::create(&analysis_path).expect("Failed to create analysis file");
    serde_json::to_writer_pretty(file, &analysis).expect("Failed to write analysis");
    println!("\nAnalysis saved to: {}", analysis_path);
}

fn find_game_json(game_id: &str) -> Option<std::path::PathBuf> {
    // Search in data/games/*/{game_id}.json
    for entry in std::fs::read_dir("data/games").ok()?.flatten() {
        if entry.path().is_dir() {
            let json_path = entry.path().join(format!("{}.json", game_id));
            if json_path.exists() {
                return Some(json_path);
            }
        }
    }
    None
}
```

**Step 3: Add config option for Stockfish path**

Update `config.rs` to support stockfish path in arena.toml:

```rust
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ArenaConfig {
    #[serde(default)]
    pub bots: HashMap<String, BotConfig>,
    #[serde(default)]
    pub presets: HashMap<String, PresetConfig>,
    #[serde(default)]
    pub stockfish_path: Option<String>,
}
```

**Step 4: Verify and commit**

Run: `cargo build -p bot-arena`

```bash
git add crates/bot-arena/Cargo.toml crates/bot-arena/src/main.rs crates/bot-arena/src/config.rs
git commit -m "feat(bot-arena): add analyze command for Stockfish analysis"
```

---

## Task 7: Add analysis integration tests

**Files:**
- Create: `crates/chess-analysis/tests/integration.rs`

**Step 1: Create integration test**

Create `crates/chess-analysis/tests/integration.rs`:

```rust
//! Integration tests for chess-analysis.
//!
//! These tests require Stockfish to be installed and available in PATH.
//! Run with: cargo test -p chess-analysis --test integration -- --ignored

use chess_analysis::{AnalysisConfig, AnalysisEngine, Evaluation, GameAnalyzer, MoveInput, MoveQuality};

fn stockfish_available() -> bool {
    std::process::Command::new("stockfish")
        .arg("--version")
        .output()
        .is_ok()
}

#[test]
#[ignore = "requires Stockfish"]
fn test_engine_basic_analysis() {
    if !stockfish_available() {
        eprintln!("Stockfish not found, skipping test");
        return;
    }

    let mut engine = AnalysisEngine::new("stockfish").unwrap();
    assert!(engine.name().contains("Stockfish"));

    let analysis = engine.analyze_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 10).unwrap();
    assert!(!analysis.best_move.is_empty());
    assert!(analysis.depth >= 10);
}

#[test]
#[ignore = "requires Stockfish"]
fn test_game_analyzer_scholars_mate() {
    if !stockfish_available() {
        return;
    }

    // Scholar's mate: 1.e4 e5 2.Qh5 Nc6 3.Bc4 Nf6?? 4.Qxf7#
    let moves = vec![
        MoveInput { uci: "e2e4".into(), bot_eval_cp: Some(30), bot_eval_mate: None, bot_depth: Some(10), bot_nodes: None, bot_time_ms: None, bot_pv: vec![] },
        MoveInput { uci: "e7e5".into(), bot_eval_cp: Some(-20), bot_eval_mate: None, bot_depth: Some(10), bot_nodes: None, bot_time_ms: None, bot_pv: vec![] },
        MoveInput { uci: "d1h5".into(), bot_eval_cp: Some(0), bot_eval_mate: None, bot_depth: Some(10), bot_nodes: None, bot_time_ms: None, bot_pv: vec![] },
        MoveInput { uci: "b8c6".into(), bot_eval_cp: Some(40), bot_eval_mate: None, bot_depth: Some(10), bot_nodes: None, bot_time_ms: None, bot_pv: vec![] },
        MoveInput { uci: "f1c4".into(), bot_eval_cp: Some(50), bot_eval_mate: None, bot_depth: Some(10), bot_nodes: None, bot_time_ms: None, bot_pv: vec![] },
        MoveInput { uci: "g8f6".into(), bot_eval_cp: Some(-100), bot_eval_mate: None, bot_depth: Some(10), bot_nodes: None, bot_time_ms: None, bot_pv: vec![] }, // Blunder!
        MoveInput { uci: "h5f7".into(), bot_eval_cp: None, bot_eval_mate: Some(0), bot_depth: Some(10), bot_nodes: None, bot_time_ms: None, bot_pv: vec![] }, // Mate
    ];

    let config = AnalysisConfig { depth: 12, opening_book_moves: 0 };
    let mut analyzer = GameAnalyzer::new("stockfish", config).unwrap();
    let analysis = analyzer.analyze_game("test", "white", "black", &moves, "white").unwrap();

    // Nf6 should be classified as a blunder
    assert!(analysis.moves.iter().any(|m| m.uci == "g8f6" && m.quality == MoveQuality::Blunder));
}
```

**Step 2: Verify and commit**

Run (requires Stockfish): `cargo test -p chess-analysis --test integration -- --ignored`

```bash
git add crates/chess-analysis/tests/
git commit -m "test(chess-analysis): add integration tests requiring Stockfish"
```

---

## Summary

After completing all 7 tasks, you will have:

1. **chess-analysis crate** with:
   - `Evaluation` type for centipawn/mate scores
   - `MoveQuality` classification (Best/Excellent/Good/Inaccuracy/Mistake/Blunder)
   - `AnalysisEngine` wrapper for Stockfish
   - `GameAnalyzer` for full game analysis
   - `PlayerStats` for aggregate statistics

2. **bot-arena analyze command** that:
   - Loads a game from JSON
   - Analyzes every position with Stockfish
   - Classifies each move's quality
   - Calculates accuracy percentages
   - Saves detailed analysis to JSON

**Usage:**
```bash
# After running a match
bot-arena match minimax random --games 1

# Analyze the game (find game_id from output or data/games/)
bot-arena analyze --game-id abc123 --depth 15

# Output:
# === Analysis Results ===
# White (minimax): 78.5% accuracy
#   Blunders: 0, Mistakes: 2, Inaccuracies: 5
# ...
```

**Next Phase:** Phase 3 - Opening Database
