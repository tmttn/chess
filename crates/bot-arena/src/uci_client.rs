//! UCI (Universal Chess Interface) client for communicating with chess engines.
//!
//! This module provides functionality to spawn and communicate with UCI-compatible
//! chess engines as subprocesses. It handles the UCI protocol handshake, position
//! setup, and move generation.
//!
//! # Example
//!
//! ```no_run
//! use bot_arena::uci_client::UciClient;
//!
//! let mut client = UciClient::spawn("/path/to/engine").unwrap();
//! client.init().unwrap();
//! client.set_position(&[]).unwrap();
//! let (best_move, search_info) = client.go("movetime 1000").unwrap();
//! println!("Best move: {}", best_move);
//! if let Some(info) = search_info {
//!     println!("Search depth: {:?}", info.depth);
//! }
//! client.quit().unwrap();
//! ```

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use thiserror::Error;

/// Information extracted from UCI `info` lines during engine search.
///
/// This struct captures key search metrics that UCI engines report while
/// calculating moves, including depth, score, nodes searched, and the
/// principal variation (PV).
///
/// # Example
///
/// ```
/// use bot_arena::uci_client::SearchInfo;
///
/// let info = SearchInfo::parse("info depth 20 score cp 35 nodes 1234567 time 1500 pv e2e4 e7e5");
/// assert!(info.is_some());
/// let info = info.unwrap();
/// assert_eq!(info.depth, Some(20));
/// assert_eq!(info.score_cp, Some(35));
/// ```
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SearchInfo {
    /// The search depth reached (in plies).
    pub depth: Option<u32>,
    /// The score in centipawns (100 = 1 pawn advantage).
    pub score_cp: Option<i32>,
    /// Mate score: positive means mate in N moves, negative means getting mated.
    pub score_mate: Option<i32>,
    /// Number of nodes searched.
    pub nodes: Option<u64>,
    /// Time spent searching in milliseconds.
    pub time_ms: Option<u64>,
    /// Principal variation - the expected best line of play.
    pub pv: Vec<String>,
}

impl SearchInfo {
    /// Parses a UCI `info` line into a `SearchInfo` struct.
    ///
    /// Returns `None` if the line doesn't start with "info " or doesn't
    /// contain depth information (which indicates it's not a substantive
    /// search info line).
    ///
    /// # Arguments
    ///
    /// * `line` - A UCI info line from the engine.
    ///
    /// # Returns
    ///
    /// `Some(SearchInfo)` if the line is a valid info line with depth,
    /// `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use bot_arena::uci_client::SearchInfo;
    ///
    /// // Basic info line
    /// let info = SearchInfo::parse("info depth 10 score cp 25 nodes 50000 time 100 pv e2e4 e7e5");
    /// assert!(info.is_some());
    ///
    /// // Mate score
    /// let mate_info = SearchInfo::parse("info depth 15 score mate 3 pv e2e4");
    /// assert!(mate_info.is_some());
    /// assert_eq!(mate_info.unwrap().score_mate, Some(3));
    ///
    /// // Non-info line returns None
    /// assert!(SearchInfo::parse("bestmove e2e4").is_none());
    /// ```
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

/// Errors that can occur when communicating with a UCI engine.
///
/// This enum covers process spawning errors, communication errors,
/// and protocol-level errors.
#[derive(Error, Debug)]
pub enum UciError {
    /// Failed to spawn the engine process or perform I/O operations.
    #[error("Failed to spawn process: {0}")]
    SpawnError(#[from] std::io::Error),
    /// The engine process is not ready to receive commands.
    #[error("Process not ready")]
    NotReady,
    /// The engine returned an invalid or unexpected response.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// A client for communicating with a UCI-compatible chess engine.
///
/// `UciClient` manages a subprocess running a chess engine and provides
/// methods to send UCI commands and receive responses. The engine is
/// communicated with via stdin/stdout pipes.
///
/// # Lifecycle
///
/// 1. Spawn the engine with [`UciClient::spawn`]
/// 2. Initialize the UCI protocol with [`UciClient::init`]
/// 3. Set positions and request moves with [`UciClient::set_position`] and [`UciClient::go`]
/// 4. Clean up with [`UciClient::quit`] (or rely on [`Drop`] implementation)
pub struct UciClient {
    /// The child process handle.
    process: Child,
    /// Handle to write commands to the engine's stdin.
    stdin: ChildStdin,
    /// Buffered reader for the engine's stdout.
    stdout: BufReader<ChildStdout>,
    /// The engine's name as reported during UCI initialization.
    pub name: String,
}

impl UciClient {
    /// Spawns a new UCI engine process.
    ///
    /// Creates a new subprocess for the specified engine executable with
    /// pipes connected to stdin and stdout. The process is not yet initialized
    /// for UCI communication; call [`init`](Self::init) after spawning.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the UCI-compatible chess engine executable.
    ///
    /// # Errors
    ///
    /// Returns [`UciError::SpawnError`] if the process cannot be spawned,
    /// typically because the executable doesn't exist or lacks permissions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bot_arena::uci_client::UciClient;
    ///
    /// let client = UciClient::spawn("/usr/bin/stockfish")?;
    /// # Ok::<(), bot_arena::uci_client::UciError>(())
    /// ```
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

    /// Sends a command to the UCI engine.
    ///
    /// Writes the command followed by a newline to the engine's stdin
    /// and flushes the buffer.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The UCI command to send (without trailing newline).
    ///
    /// # Errors
    ///
    /// Returns [`UciError::SpawnError`] if writing to stdin fails.
    pub fn send(&mut self, cmd: &str) -> Result<(), UciError> {
        writeln!(self.stdin, "{}", cmd)?;
        self.stdin.flush()?;
        Ok(())
    }

    /// Reads a single line from the engine's stdout.
    ///
    /// Blocks until a complete line is available. The returned string
    /// has leading and trailing whitespace trimmed.
    ///
    /// # Errors
    ///
    /// Returns [`UciError::SpawnError`] if reading from stdout fails.
    pub fn read_line(&mut self) -> Result<String, UciError> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        Ok(line.trim().to_string())
    }

    /// Initializes the UCI protocol with the engine.
    ///
    /// Sends the `uci` command and waits for `uciok`, capturing the engine's
    /// name from the `id name` response. Then sends `isready` and waits for
    /// `readyok` to ensure the engine is ready for commands.
    ///
    /// After successful initialization, the engine's name is available via
    /// the [`name`](Self::name) field.
    ///
    /// # Errors
    ///
    /// Returns [`UciError::SpawnError`] if communication with the engine fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bot_arena::uci_client::UciClient;
    ///
    /// let mut client = UciClient::spawn("/usr/bin/stockfish")?;
    /// client.init()?;
    /// println!("Engine name: {}", client.name);
    /// # Ok::<(), bot_arena::uci_client::UciError>(())
    /// ```
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

    /// Sets the current position for the engine.
    ///
    /// Sends a `position startpos moves ...` command to set up the board.
    /// If no moves are provided, sets up the standard starting position.
    ///
    /// # Arguments
    ///
    /// * `moves` - A slice of moves in UCI notation (e.g., `["e2e4", "e7e5"]`).
    ///
    /// # Errors
    ///
    /// Returns [`UciError::SpawnError`] if sending the command fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bot_arena::uci_client::UciClient;
    ///
    /// let mut client = UciClient::spawn("/usr/bin/stockfish")?;
    /// client.init()?;
    ///
    /// // Set up starting position
    /// client.set_position(&[])?;
    ///
    /// // Set up position after 1. e4 e5 2. Nf3
    /// client.set_position(&["e2e4".to_string(), "e7e5".to_string(), "g1f3".to_string()])?;
    /// # Ok::<(), bot_arena::uci_client::UciError>(())
    /// ```
    pub fn set_position(&mut self, moves: &[String]) -> Result<(), UciError> {
        if moves.is_empty() {
            self.send("position startpos")
        } else {
            self.send(&format!("position startpos moves {}", moves.join(" ")))
        }
    }

    /// Requests the engine to calculate the best move.
    ///
    /// Sends a `go` command with the specified time control and waits for
    /// the engine to respond with `bestmove`. Returns the best move in
    /// UCI notation along with the last search info received.
    ///
    /// # Arguments
    ///
    /// * `time_control` - Time control parameters (e.g., `"movetime 1000"`,
    ///   `"depth 10"`, `"wtime 60000 btime 60000"`).
    ///
    /// # Returns
    ///
    /// A tuple of `(bestmove, Option<SearchInfo>)` where `bestmove` is the
    /// UCI move string and `SearchInfo` contains the last search metrics
    /// reported by the engine before returning the best move.
    ///
    /// # Errors
    ///
    /// Returns [`UciError::SpawnError`] if communication fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bot_arena::uci_client::UciClient;
    ///
    /// let mut client = UciClient::spawn("/usr/bin/stockfish")?;
    /// client.init()?;
    /// client.set_position(&[])?;
    ///
    /// // Get best move with 1 second thinking time
    /// let (best_move, search_info) = client.go("movetime 1000")?;
    /// println!("Best move: {}", best_move);
    /// if let Some(info) = search_info {
    ///     println!("Depth: {:?}, Score: {:?}", info.depth, info.score_cp);
    /// }
    /// # Ok::<(), bot_arena::uci_client::UciError>(())
    /// ```
    pub fn go(&mut self, time_control: &str) -> Result<(String, Option<SearchInfo>), UciError> {
        self.send(&format!("go {}", time_control))?;

        let mut last_info: Option<SearchInfo> = None;

        loop {
            let line = self.read_line()?;
            if line.starts_with("bestmove ") {
                let bestmove = line.split_whitespace().nth(1).unwrap_or("").to_string();
                return Ok((bestmove, last_info));
            }
            // Capture search info lines
            if let Some(info) = SearchInfo::parse(&line) {
                last_info = Some(info);
            }
        }
    }

    /// Gracefully shuts down the UCI engine.
    ///
    /// Sends the `quit` command and waits for the process to exit.
    /// This is the preferred way to terminate the engine.
    ///
    /// # Errors
    ///
    /// Returns [`UciError::SpawnError`] if sending the quit command fails.
    ///
    /// # Note
    ///
    /// If this method is not called, the [`Drop`] implementation will
    /// attempt to terminate the engine, but may use forced termination.
    pub fn quit(&mut self) -> Result<(), UciError> {
        self.send("quit")?;
        let _ = self.process.wait();
        Ok(())
    }
}

impl Drop for UciClient {
    /// Ensures the engine process is terminated when the client is dropped.
    ///
    /// Attempts to send a `quit` command first for graceful shutdown,
    /// then forcefully kills the process if necessary.
    fn drop(&mut self) {
        let _ = self.send("quit");
        let _ = self.process.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uci_error_display() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let spawn_error = UciError::SpawnError(io_error);
        assert!(spawn_error.to_string().contains("Failed to spawn process"));

        let not_ready = UciError::NotReady;
        assert_eq!(not_ready.to_string(), "Process not ready");

        let invalid = UciError::InvalidResponse("bad data".to_string());
        assert_eq!(invalid.to_string(), "Invalid response: bad data");
    }

    #[test]
    fn test_spawn_nonexistent_executable_returns_error() {
        let result = UciClient::spawn("/nonexistent/path/to/engine");
        assert!(result.is_err());
        match result {
            Err(UciError::SpawnError(_)) => {}
            _ => panic!("Expected SpawnError"),
        }
    }

    #[test]
    fn test_uci_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let uci_error: UciError = io_error.into();
        match uci_error {
            UciError::SpawnError(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected SpawnError variant"),
        }
    }

    #[test]
    fn test_uci_error_debug_format() {
        let error = UciError::InvalidResponse("test response".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidResponse"));
        assert!(debug_str.contains("test response"));
    }

    #[test]
    fn test_search_info_parse_basic() {
        let line = "info depth 20 score cp 35 nodes 1234567 time 1500 pv e2e4 e7e5 g1f3";
        let info = SearchInfo::parse(line);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.depth, Some(20));
        assert_eq!(info.score_cp, Some(35));
        assert_eq!(info.score_mate, None);
        assert_eq!(info.nodes, Some(1234567));
        assert_eq!(info.time_ms, Some(1500));
        assert_eq!(info.pv, vec!["e2e4", "e7e5", "g1f3"]);
    }

    #[test]
    fn test_search_info_parse_with_mate() {
        let line = "info depth 15 score mate 3 nodes 500000 pv e2e4 e7e5 d1h5";
        let info = SearchInfo::parse(line);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.depth, Some(15));
        assert_eq!(info.score_cp, None);
        assert_eq!(info.score_mate, Some(3));
        assert_eq!(info.nodes, Some(500000));
        assert_eq!(info.pv, vec!["e2e4", "e7e5", "d1h5"]);
    }

    #[test]
    fn test_search_info_parse_negative_mate() {
        let line = "info depth 12 score mate -5 pv a2a3";
        let info = SearchInfo::parse(line);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.score_mate, Some(-5));
    }

    #[test]
    fn test_search_info_parse_negative_score() {
        let line = "info depth 10 score cp -150 nodes 10000";
        let info = SearchInfo::parse(line);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.score_cp, Some(-150));
    }

    #[test]
    fn test_search_info_parse_invalid() {
        // Non-info line
        assert!(SearchInfo::parse("bestmove e2e4").is_none());
        assert!(SearchInfo::parse("uciok").is_none());
        assert!(SearchInfo::parse("readyok").is_none());

        // Info line without depth
        assert!(SearchInfo::parse("info string Loading weights").is_none());
        assert!(SearchInfo::parse("info currmove e2e4 currmovenumber 1").is_none());
    }

    #[test]
    fn test_search_info_parse_empty_pv() {
        let line = "info depth 5 score cp 10 nodes 100";
        let info = SearchInfo::parse(line);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.depth, Some(5));
        assert!(info.pv.is_empty());
    }

    #[test]
    fn test_search_info_serialize() {
        let info = SearchInfo {
            depth: Some(10),
            score_cp: Some(25),
            score_mate: None,
            nodes: Some(50000),
            time_ms: Some(500),
            pv: vec!["e2e4".to_string(), "e7e5".to_string()],
        };

        let json = serde_json::to_string(&info).expect("Failed to serialize");
        assert!(json.contains("\"depth\":10"));
        assert!(json.contains("\"score_cp\":25"));
        assert!(json.contains("\"nodes\":50000"));
        assert!(json.contains("\"time_ms\":500"));
        assert!(json.contains("\"pv\":[\"e2e4\",\"e7e5\"]"));
    }

    #[test]
    fn test_search_info_default() {
        let info = SearchInfo::default();
        assert_eq!(info.depth, None);
        assert_eq!(info.score_cp, None);
        assert_eq!(info.score_mate, None);
        assert_eq!(info.nodes, None);
        assert_eq!(info.time_ms, None);
        assert!(info.pv.is_empty());
    }

    #[test]
    fn test_search_info_clone() {
        let info = SearchInfo {
            depth: Some(10),
            score_cp: Some(25),
            score_mate: None,
            nodes: Some(50000),
            time_ms: Some(500),
            pv: vec!["e2e4".to_string()],
        };

        let cloned = info.clone();
        assert_eq!(cloned.depth, info.depth);
        assert_eq!(cloned.score_cp, info.score_cp);
        assert_eq!(cloned.pv, info.pv);
    }
}
