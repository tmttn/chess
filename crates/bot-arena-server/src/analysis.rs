//! Stockfish analysis pool.
//!
//! Provides on-demand position analysis using Stockfish engines.
//! Uses a semaphore to limit concurrent engine processes.

use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Semaphore;

/// Result of a Stockfish analysis.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Search depth reached.
    pub depth: i32,
    /// Score in centipawns (positive = white advantage).
    pub score_cp: Option<i32>,
    /// Mate score (positive = white mates in N, negative = black mates in N).
    pub score_mate: Option<i32>,
    /// Best move in UCI notation.
    pub best_move: String,
    /// Principal variation - expected best line.
    pub pv: Vec<String>,
}

/// Pool of Stockfish engines for concurrent analysis.
pub struct EnginePool {
    semaphore: Arc<Semaphore>,
    stockfish_path: String,
}

impl EnginePool {
    /// Create a new engine pool.
    ///
    /// # Arguments
    /// * `stockfish_path` - Path to the Stockfish executable
    /// * `pool_size` - Maximum number of concurrent analyses
    pub fn new(stockfish_path: String, pool_size: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(pool_size)),
            stockfish_path,
        }
    }

    /// Analyze a position.
    ///
    /// # Arguments
    /// * `fen` - Position in FEN notation
    /// * `depth` - Search depth
    ///
    /// # Returns
    /// Analysis result with best move, score, and principal variation.
    pub async fn analyze(&self, fen: &str, depth: i32) -> anyhow::Result<AnalysisResult> {
        // Acquire permit to limit concurrency
        let _permit = self.semaphore.acquire().await?;

        // Spawn Stockfish process
        let mut child = Command::new(&self.stockfish_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");
        let mut reader = BufReader::new(stdout).lines();

        // Send UCI commands
        stdin.write_all(b"uci\n").await?;
        stdin
            .write_all(format!("position fen {}\n", fen).as_bytes())
            .await?;
        stdin
            .write_all(format!("go depth {}\n", depth).as_bytes())
            .await?;

        let mut result = AnalysisResult {
            depth: 0,
            score_cp: None,
            score_mate: None,
            best_move: String::new(),
            pv: Vec::new(),
        };

        // Parse output
        while let Some(line) = reader.next_line().await? {
            if line.starts_with("info depth")
                && !line.contains("upperbound")
                && !line.contains("lowerbound")
            {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    match *part {
                        "depth" => {
                            if let Some(d) = parts.get(i + 1).and_then(|s| s.parse().ok()) {
                                result.depth = d;
                            }
                        }
                        "cp" => {
                            result.score_cp = parts.get(i + 1).and_then(|s| s.parse().ok());
                        }
                        "mate" => {
                            result.score_mate = parts.get(i + 1).and_then(|s| s.parse().ok());
                        }
                        "pv" => {
                            result.pv = parts[i + 1..].iter().map(|s| (*s).to_string()).collect();
                        }
                        _ => {}
                    }
                }
            } else if line.starts_with("bestmove") {
                result.best_move = line.split_whitespace().nth(1).unwrap_or("").to_string();
                break;
            }
        }

        // Clean up
        stdin.write_all(b"quit\n").await?;
        child.wait().await?;

        Ok(result)
    }
}
