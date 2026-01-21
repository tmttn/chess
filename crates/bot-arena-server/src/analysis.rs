//! Stockfish analysis pool.
//!
//! Provides on-demand position analysis using Stockfish engines.
//! Uses a semaphore to limit concurrent engine processes.
//! Supports lazy initialization to defer engine validation until first use.

use std::process::Stdio;
use std::sync::{Arc, OnceLock};
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

    /// Get the Stockfish executable path.
    #[cfg(test)]
    pub fn stockfish_path(&self) -> &str {
        &self.stockfish_path
    }

    /// Get the pool size (number of concurrent analyses).
    pub fn pool_size(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// Lazy-initialized engine pool.
///
/// Defers the creation and validation of the Stockfish engine pool
/// until the first analysis request. This allows the server to start
/// even if Stockfish is not yet available, and provides better error
/// messages when analysis is actually needed.
pub struct LazyEnginePool {
    pool: OnceLock<EnginePool>,
    stockfish_path: String,
    pool_size: usize,
}

impl LazyEnginePool {
    /// Create a new lazy engine pool.
    ///
    /// The actual engine pool is not created until the first call to
    /// `get()` or `analyze()`.
    ///
    /// # Arguments
    /// * `stockfish_path` - Path to the Stockfish executable
    /// * `pool_size` - Maximum number of concurrent analyses
    pub fn new(stockfish_path: String, pool_size: usize) -> Self {
        Self {
            pool: OnceLock::new(),
            stockfish_path,
            pool_size,
        }
    }

    /// Get or initialize the engine pool.
    ///
    /// On first call, creates the underlying `EnginePool`. Subsequent
    /// calls return the same pool instance.
    pub fn get(&self) -> &EnginePool {
        self.pool
            .get_or_init(|| EnginePool::new(self.stockfish_path.clone(), self.pool_size))
    }

    /// Analyze a position using the lazy-initialized pool.
    ///
    /// # Arguments
    /// * `fen` - Position in FEN notation
    /// * `depth` - Search depth
    ///
    /// # Returns
    /// Analysis result with best move, score, and principal variation.
    pub async fn analyze(&self, fen: &str, depth: i32) -> anyhow::Result<AnalysisResult> {
        self.get().analyze(fen, depth).await
    }

    /// Get the configured Stockfish path.
    pub fn stockfish_path(&self) -> &str {
        &self.stockfish_path
    }

    /// Get the configured pool size.
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }

    /// Check if the pool has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.pool.get().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_result_default() {
        let result = AnalysisResult {
            depth: 0,
            score_cp: None,
            score_mate: None,
            best_move: String::new(),
            pv: Vec::new(),
        };
        assert_eq!(result.depth, 0);
        assert!(result.score_cp.is_none());
        assert!(result.best_move.is_empty());
    }

    #[test]
    fn test_analysis_result_with_values() {
        let result = AnalysisResult {
            depth: 20,
            score_cp: Some(125),
            score_mate: None,
            best_move: "e2e4".to_string(),
            pv: vec!["e2e4".to_string(), "e7e5".to_string()],
        };
        assert_eq!(result.depth, 20);
        assert_eq!(result.score_cp, Some(125));
        assert_eq!(result.best_move, "e2e4");
        assert_eq!(result.pv.len(), 2);
    }

    #[test]
    fn test_engine_pool_new() {
        let pool = EnginePool::new("stockfish".to_string(), 2);
        assert_eq!(pool.stockfish_path(), "stockfish");
    }

    #[test]
    fn test_engine_pool_size() {
        let pool = EnginePool::new("stockfish".to_string(), 4);
        assert_eq!(pool.pool_size(), 4);
    }

    #[test]
    fn test_lazy_engine_pool_new() {
        let lazy_pool = LazyEnginePool::new("/usr/bin/stockfish".to_string(), 3);
        assert_eq!(lazy_pool.stockfish_path(), "/usr/bin/stockfish");
        assert_eq!(lazy_pool.pool_size(), 3);
        assert!(!lazy_pool.is_initialized());
    }

    #[test]
    fn test_lazy_engine_pool_initialization() {
        let lazy_pool = LazyEnginePool::new("stockfish".to_string(), 2);
        assert!(!lazy_pool.is_initialized());

        // Trigger initialization
        let _pool = lazy_pool.get();
        assert!(lazy_pool.is_initialized());

        // Second call returns same pool
        let _pool2 = lazy_pool.get();
        assert!(lazy_pool.is_initialized());
    }

    #[test]
    fn test_lazy_engine_pool_configured_values() {
        let lazy_pool = LazyEnginePool::new("/opt/stockfish/bin/stockfish".to_string(), 8);

        // Get the pool and verify configuration is passed through
        let pool = lazy_pool.get();
        assert_eq!(pool.stockfish_path(), "/opt/stockfish/bin/stockfish");
        assert_eq!(pool.pool_size(), 8);
    }
}
