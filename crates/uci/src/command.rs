//! UCI command parsing.

use crate::UciError;

/// Commands sent from GUI to engine.
#[derive(Debug, Clone, PartialEq)]
pub enum GuiCommand {
    /// Initialize UCI mode.
    Uci,
    /// Query supported extensions (custom).
    Extensions,
    /// Check if engine is ready.
    IsReady,
    /// Set up position.
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },
    /// Start calculating.
    Go(GoOptions),
    /// Stop calculating.
    Stop,
    /// Quit the engine.
    Quit,
    /// Unknown command (for forward compatibility).
    Unknown(String),
}

/// Options for the `go` command.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GoOptions {
    /// Search for exactly this time in milliseconds.
    pub movetime: Option<u64>,
    /// Search to this depth.
    pub depth: Option<u32>,
    /// White time remaining in milliseconds.
    pub wtime: Option<u64>,
    /// Black time remaining in milliseconds.
    pub btime: Option<u64>,
    /// White increment per move in milliseconds.
    pub winc: Option<u64>,
    /// Black increment per move in milliseconds.
    pub binc: Option<u64>,
    /// Moves to go until next time control.
    pub movestogo: Option<u32>,
    /// Search indefinitely until `stop`.
    pub infinite: bool,
}

impl GuiCommand {
    /// Parse a UCI command string.
    pub fn parse(input: &str) -> Result<Self, UciError> {
        let input = input.trim();
        let mut parts = input.split_whitespace();

        let cmd = parts.next().unwrap_or("");

        match cmd {
            "uci" => Ok(GuiCommand::Uci),
            "extensions" => Ok(GuiCommand::Extensions),
            "isready" => Ok(GuiCommand::IsReady),
            "stop" => Ok(GuiCommand::Stop),
            "quit" => Ok(GuiCommand::Quit),
            "position" => Self::parse_position(parts),
            "go" => Self::parse_go(parts),
            "" => Ok(GuiCommand::Unknown(String::new())),
            _ => Ok(GuiCommand::Unknown(input.to_string())),
        }
    }

    fn parse_position<'a>(mut parts: impl Iterator<Item = &'a str>) -> Result<Self, UciError> {
        let mut fen = None;
        let mut moves = Vec::new();

        // Parse "startpos" or "fen <fen>"
        match parts.next() {
            Some("startpos") => {
                fen = None; // Use default starting position
            }
            Some("fen") => {
                // Collect FEN parts until "moves" or end
                let mut fen_parts = Vec::new();
                for part in parts.by_ref() {
                    if part == "moves" {
                        break;
                    }
                    fen_parts.push(part);
                }
                if !fen_parts.is_empty() {
                    fen = Some(fen_parts.join(" "));
                }
            }
            Some(other) => {
                return Err(UciError::ParseError(format!(
                    "Expected 'startpos' or 'fen', got '{}'",
                    other
                )));
            }
            None => {
                return Err(UciError::ParseError(
                    "Expected 'startpos' or 'fen'".to_string(),
                ));
            }
        }

        // Check if we're at "moves" or need to find it
        // (for startpos case, need to advance to moves)
        let remaining: Vec<&str> = parts.collect();
        let moves_start = remaining.iter().position(|&s| s == "moves");

        if let Some(idx) = moves_start {
            moves = remaining[idx + 1..].iter().map(|s| s.to_string()).collect();
        } else if fen.is_some() {
            // FEN case already consumed "moves" in the loop
            moves = remaining.iter().map(|s| s.to_string()).collect();
        }

        Ok(GuiCommand::Position { fen, moves })
    }

    fn parse_go<'a>(parts: impl Iterator<Item = &'a str>) -> Result<Self, UciError> {
        let mut opts = GoOptions::default();
        let parts: Vec<&str> = parts.collect();
        let mut i = 0;

        while i < parts.len() {
            match parts[i] {
                "movetime" => {
                    i += 1;
                    if i < parts.len() {
                        opts.movetime = parts[i].parse().ok();
                    }
                }
                "depth" => {
                    i += 1;
                    if i < parts.len() {
                        opts.depth = parts[i].parse().ok();
                    }
                }
                "wtime" => {
                    i += 1;
                    if i < parts.len() {
                        opts.wtime = parts[i].parse().ok();
                    }
                }
                "btime" => {
                    i += 1;
                    if i < parts.len() {
                        opts.btime = parts[i].parse().ok();
                    }
                }
                "winc" => {
                    i += 1;
                    if i < parts.len() {
                        opts.winc = parts[i].parse().ok();
                    }
                }
                "binc" => {
                    i += 1;
                    if i < parts.len() {
                        opts.binc = parts[i].parse().ok();
                    }
                }
                "movestogo" => {
                    i += 1;
                    if i < parts.len() {
                        opts.movestogo = parts[i].parse().ok();
                    }
                }
                "infinite" => {
                    opts.infinite = true;
                }
                _ => {}
            }
            i += 1;
        }

        Ok(GuiCommand::Go(opts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uci() {
        assert_eq!(GuiCommand::parse("uci").unwrap(), GuiCommand::Uci);
    }

    #[test]
    fn parse_isready() {
        assert_eq!(GuiCommand::parse("isready").unwrap(), GuiCommand::IsReady);
    }

    #[test]
    fn parse_position_startpos() {
        let cmd = GuiCommand::parse("position startpos").unwrap();
        assert_eq!(
            cmd,
            GuiCommand::Position {
                fen: None,
                moves: vec![]
            }
        );
    }

    #[test]
    fn parse_position_startpos_with_moves() {
        let cmd = GuiCommand::parse("position startpos moves e2e4 e7e5").unwrap();
        assert_eq!(
            cmd,
            GuiCommand::Position {
                fen: None,
                moves: vec!["e2e4".to_string(), "e7e5".to_string()]
            }
        );
    }

    #[test]
    fn parse_position_fen() {
        let cmd = GuiCommand::parse(
            "position fen rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        )
        .unwrap();
        assert_eq!(
            cmd,
            GuiCommand::Position {
                fen: Some(
                    "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string()
                ),
                moves: vec![]
            }
        );
    }

    #[test]
    fn parse_go_movetime() {
        let cmd = GuiCommand::parse("go movetime 1000").unwrap();
        if let GuiCommand::Go(opts) = cmd {
            assert_eq!(opts.movetime, Some(1000));
        } else {
            panic!("Expected Go command");
        }
    }

    #[test]
    fn parse_go_depth() {
        let cmd = GuiCommand::parse("go depth 10").unwrap();
        if let GuiCommand::Go(opts) = cmd {
            assert_eq!(opts.depth, Some(10));
        } else {
            panic!("Expected Go command");
        }
    }

    #[test]
    fn parse_go_infinite() {
        let cmd = GuiCommand::parse("go infinite").unwrap();
        if let GuiCommand::Go(opts) = cmd {
            assert!(opts.infinite);
        } else {
            panic!("Expected Go command");
        }
    }
}
