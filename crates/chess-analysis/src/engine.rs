//! Stockfish engine wrapper for position analysis.

use crate::Evaluation;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use thiserror::Error;

/// Maximum number of lines to read before giving up on a UCI response.
pub const MAX_UCI_LINES: usize = 1000;

/// Errors that can occur when working with chess engines.
#[derive(Error, Debug)]
pub enum EngineError {
    /// Failed to spawn the engine process.
    #[error("Failed to spawn engine: {0}")]
    SpawnError(#[from] std::io::Error),
    /// Engine executable was not found at the specified path.
    #[error("Engine not found at path: {0}")]
    NotFound(String),
    /// Engine failed to initialize properly (UCI handshake failed).
    #[error("Engine initialization failed")]
    InitFailed,
    /// Engine returned an invalid or unexpected response.
    #[error("Invalid engine response: {0}")]
    InvalidResponse(String),
}

/// Result of analyzing a chess position.
#[derive(Debug, Clone)]
pub struct PositionAnalysis {
    /// The best move found (in UCI notation, e.g., "e2e4").
    pub best_move: String,
    /// The position evaluation.
    pub evaluation: Evaluation,
    /// The search depth reached.
    pub depth: u32,
    /// The number of nodes searched.
    pub nodes: u64,
    /// The principal variation (sequence of best moves).
    pub pv: Vec<String>,
}

/// Wrapper for UCI-compatible analysis engines like Stockfish.
///
/// This struct manages communication with an external chess engine
/// to obtain position evaluations and best move recommendations.
pub struct AnalysisEngine {
    /// The engine process handle.
    process: Child,
    /// Writer for sending commands to the engine.
    stdin: ChildStdin,
    /// Reader for receiving responses from the engine.
    stdout: BufReader<ChildStdout>,
    /// The engine's name (reported via UCI id).
    name: String,
}

impl AnalysisEngine {
    /// Create a new analysis engine.
    ///
    /// Spawns the engine process and performs UCI initialization handshake.
    ///
    /// # Arguments
    ///
    /// * `engine_path` - Path to the UCI engine executable
    ///
    /// # Returns
    ///
    /// A new `AnalysisEngine` instance ready for analysis.
    ///
    /// # Errors
    ///
    /// - `EngineError::NotFound` if the engine path doesn't exist
    /// - `EngineError::SpawnError` if the engine process fails to start
    /// - `EngineError::InitFailed` if UCI initialization fails
    pub fn new(engine_path: &str) -> Result<Self, EngineError> {
        // Check if the engine path exists
        if !std::path::Path::new(engine_path).exists() {
            return Err(EngineError::NotFound(engine_path.to_string()));
        }

        // Spawn the engine process
        let mut process = Command::new(engine_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = process.stdin.take().ok_or(EngineError::InitFailed)?;
        let stdout = process.stdout.take().ok_or(EngineError::InitFailed)?;
        let stdout = BufReader::new(stdout);

        let mut engine = Self {
            process,
            stdin,
            stdout,
            name: String::new(),
        };

        // Initialize UCI protocol
        engine.init_uci()?;

        Ok(engine)
    }

    /// Initialize the UCI protocol with the engine.
    fn init_uci(&mut self) -> Result<(), EngineError> {
        // Send "uci" and wait for "uciok"
        self.send_command("uci")?;

        let mut name = String::new();
        let mut lines_read = 0;
        loop {
            if lines_read > MAX_UCI_LINES {
                return Err(EngineError::InitFailed);
            }
            lines_read += 1;
            let line = self.read_line()?;
            if line.starts_with("id name ") {
                name = line.strip_prefix("id name ").unwrap_or("").to_string();
            } else if line == "uciok" {
                break;
            }
        }

        self.name = if name.is_empty() {
            "Unknown Engine".to_string()
        } else {
            name
        };

        // Send "isready" and wait for "readyok"
        self.send_command("isready")?;
        let mut lines_read = 0;
        loop {
            if lines_read > MAX_UCI_LINES {
                return Err(EngineError::InitFailed);
            }
            lines_read += 1;
            let line = self.read_line()?;
            if line == "readyok" {
                break;
            }
        }

        Ok(())
    }

    /// Returns the engine's name as reported via UCI protocol.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Analyze a position given in FEN notation.
    ///
    /// # Arguments
    ///
    /// * `fen` - Position in FEN notation
    /// * `depth` - Maximum search depth
    ///
    /// # Returns
    ///
    /// Position analysis including best move, evaluation, and principal variation.
    pub fn analyze_fen(&mut self, fen: &str, depth: u32) -> Result<PositionAnalysis, EngineError> {
        self.send_command(&format!("position fen {}", fen))?;
        self.run_analysis(depth)
    }

    /// Analyze a position given as a sequence of moves from the starting position.
    ///
    /// # Arguments
    ///
    /// * `moves` - Sequence of moves in UCI notation (e.g., ["e2e4", "e7e5"])
    /// * `depth` - Maximum search depth
    ///
    /// # Returns
    ///
    /// Position analysis including best move, evaluation, and principal variation.
    pub fn analyze_moves(
        &mut self,
        moves: &[String],
        depth: u32,
    ) -> Result<PositionAnalysis, EngineError> {
        if moves.is_empty() {
            self.send_command("position startpos")?;
        } else {
            let moves_str = moves.join(" ");
            self.send_command(&format!("position startpos moves {}", moves_str))?;
        }
        self.run_analysis(depth)
    }

    /// Run the analysis for the current position.
    fn run_analysis(&mut self, depth: u32) -> Result<PositionAnalysis, EngineError> {
        self.send_command(&format!("go depth {}", depth))?;

        let mut best_move = String::new();
        let mut evaluation = Evaluation::Centipawn(0);
        let mut best_depth: u32 = 0;
        let mut nodes: u64 = 0;
        let mut pv: Vec<String> = Vec::new();

        let mut lines_read = 0;
        loop {
            if lines_read > MAX_UCI_LINES {
                return Err(EngineError::InvalidResponse(
                    "Too many lines without bestmove".to_string(),
                ));
            }
            lines_read += 1;
            let line = self.read_line()?;

            if line.starts_with("info depth ") {
                // Parse info line: "info depth X score cp Y nodes Z pv ..."
                if let Some(parsed) = Self::parse_info_line(&line) {
                    best_depth = parsed.0;
                    evaluation = parsed.1;
                    nodes = parsed.2;
                    pv = parsed.3;
                }
            } else if line.starts_with("bestmove ") {
                // Parse bestmove: "bestmove e2e4 ponder e7e5"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    best_move = parts[1].to_string();
                }
                break;
            }
        }

        if best_move.is_empty() {
            return Err(EngineError::InvalidResponse(
                "No best move received".to_string(),
            ));
        }

        Ok(PositionAnalysis {
            best_move,
            evaluation,
            depth: best_depth,
            nodes,
            pv,
        })
    }

    /// Parse a UCI info line to extract depth, score, nodes, and PV.
    ///
    /// Format: "info depth X score cp Y nodes Z pv move1 move2 ..."
    /// or: "info depth X score mate Y nodes Z pv move1 move2 ..."
    fn parse_info_line(line: &str) -> Option<(u32, Evaluation, u64, Vec<String>)> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let mut depth: Option<u32> = None;
        let mut cp: Option<i32> = None;
        let mut mate: Option<i32> = None;
        let mut nodes: u64 = 0;
        let mut pv: Vec<String> = Vec::new();
        let mut in_pv = false;

        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                "depth" => {
                    if i + 1 < parts.len() {
                        depth = parts[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "score" => {
                    if i + 2 < parts.len() {
                        match parts[i + 1] {
                            "cp" => {
                                cp = parts[i + 2].parse().ok();
                                i += 2;
                            }
                            "mate" => {
                                mate = parts[i + 2].parse().ok();
                                i += 2;
                            }
                            _ => {}
                        }
                    }
                }
                "nodes" => {
                    if i + 1 < parts.len() {
                        nodes = parts[i + 1].parse().unwrap_or(0);
                        i += 1;
                    }
                }
                "pv" => {
                    in_pv = true;
                }
                _ => {
                    if in_pv {
                        pv.push(parts[i].to_string());
                    }
                }
            }
            i += 1;
        }

        let d = depth?;
        let eval = Evaluation::from_uci_score(cp, mate)?;

        Some((d, eval, nodes, pv))
    }

    /// Stop the current search.
    pub fn stop(&mut self) -> Result<(), EngineError> {
        self.send_command("stop")
    }

    /// Clear the engine's hash tables and prepare for a new game.
    pub fn clear_hash(&mut self) -> Result<(), EngineError> {
        self.send_command("ucinewgame")?;
        // Wait for engine to be ready after clearing
        self.send_command("isready")?;
        let mut lines_read = 0;
        loop {
            if lines_read > MAX_UCI_LINES {
                return Err(EngineError::InitFailed);
            }
            lines_read += 1;
            let line = self.read_line()?;
            if line == "readyok" {
                break;
            }
        }
        Ok(())
    }

    /// Send a command to the engine.
    fn send_command(&mut self, command: &str) -> Result<(), EngineError> {
        writeln!(self.stdin, "{}", command)?;
        self.stdin.flush()?;
        Ok(())
    }

    /// Read a line from the engine's output.
    fn read_line(&mut self) -> Result<String, EngineError> {
        let mut line = String::new();
        let bytes = self.stdout.read_line(&mut line)?;
        if bytes == 0 {
            return Err(EngineError::InvalidResponse(
                "Engine closed unexpectedly".to_string(),
            ));
        }
        Ok(line.trim().to_string())
    }
}

impl Drop for AnalysisEngine {
    fn drop(&mut self) {
        // Try to send quit command to gracefully terminate the engine
        let _ = self.send_command("quit");
        // Give the engine a moment to exit gracefully
        let _ = self.process.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_not_found() {
        let result = AnalysisEngine::new("/nonexistent/path/to/stockfish");
        assert!(result.is_err());
        match result {
            Err(EngineError::NotFound(path)) => {
                assert_eq!(path, "/nonexistent/path/to/stockfish");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_position_analysis_clone() {
        let analysis = PositionAnalysis {
            best_move: "e2e4".to_string(),
            evaluation: Evaluation::Centipawn(35),
            depth: 20,
            nodes: 1_000_000,
            pv: vec!["e2e4".to_string(), "e7e5".to_string()],
        };

        let cloned = analysis.clone();
        assert_eq!(cloned.best_move, "e2e4");
        assert_eq!(cloned.depth, 20);
        assert_eq!(cloned.nodes, 1_000_000);
        assert_eq!(cloned.pv.len(), 2);
    }

    #[test]
    fn test_engine_error_display() {
        let spawn_err = EngineError::SpawnError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(spawn_err.to_string().contains("Failed to spawn engine"));

        let not_found = EngineError::NotFound("/path/to/engine".to_string());
        assert!(not_found.to_string().contains("/path/to/engine"));

        let init_failed = EngineError::InitFailed;
        assert_eq!(init_failed.to_string(), "Engine initialization failed");

        let invalid = EngineError::InvalidResponse("bad response".to_string());
        assert!(invalid.to_string().contains("bad response"));
    }

    #[test]
    fn test_parse_info_line_centipawn() {
        let line = "info depth 15 score cp 35 nodes 50000 pv e2e4 e7e5 g1f3";
        let result = AnalysisEngine::parse_info_line(line);
        assert!(result.is_some());
        let (depth, eval, nodes, pv) = result.unwrap();
        assert_eq!(depth, 15);
        assert_eq!(eval, Evaluation::Centipawn(35));
        assert_eq!(nodes, 50000);
        assert_eq!(pv, vec!["e2e4", "e7e5", "g1f3"]);
    }

    #[test]
    fn test_parse_info_line_mate() {
        let line = "info depth 12 score mate 3 nodes 10000 pv d1h5 g6h5";
        let result = AnalysisEngine::parse_info_line(line);
        assert!(result.is_some());
        let (depth, eval, _nodes, pv) = result.unwrap();
        assert_eq!(depth, 12);
        assert_eq!(eval, Evaluation::Mate(3));
        assert_eq!(pv.len(), 2);
    }

    #[test]
    fn test_parse_info_line_negative_score() {
        let line = "info depth 10 score cp -150 nodes 25000 pv e7e5";
        let result = AnalysisEngine::parse_info_line(line);
        assert!(result.is_some());
        let (_depth, eval, _nodes, _pv) = result.unwrap();
        assert_eq!(eval, Evaluation::Centipawn(-150));
    }

    #[test]
    fn test_parse_info_line_no_pv() {
        let line = "info depth 5 score cp 0 nodes 1000";
        let result = AnalysisEngine::parse_info_line(line);
        assert!(result.is_some());
        let (_depth, _eval, _nodes, pv) = result.unwrap();
        assert!(pv.is_empty());
    }

    #[test]
    fn test_parse_info_line_missing_depth() {
        let line = "info score cp 35 nodes 50000 pv e2e4";
        let result = AnalysisEngine::parse_info_line(line);
        assert!(result.is_none()); // Should return None if depth is missing
    }

    #[test]
    fn test_parse_info_line_missing_score() {
        let line = "info depth 15 nodes 50000 pv e2e4";
        let result = AnalysisEngine::parse_info_line(line);
        assert!(result.is_none()); // Should return None if score is missing
    }

    #[test]
    fn test_max_iterations_constant_exists() {
        // Verify the constant exists and has a reasonable value
        assert!(MAX_UCI_LINES > 0);
        assert!(MAX_UCI_LINES >= 1000);
    }
}
