//! Core opening types and structures.

use serde::{Deserialize, Serialize};

/// Represents a chess opening with its name and move sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opening {
    /// The ECO code for this opening (e.g., "B20", "C44").
    pub eco: String,
    /// The name of the opening.
    pub name: String,
    /// The sequence of moves in UCI notation.
    pub moves: Vec<String>,
}

/// A single move from an opening book with associated metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpeningMove {
    /// The move in UCI notation (e.g., "e2e4").
    pub uci: String,
    /// Weight/frequency of this move (higher = more common).
    pub weight: u32,
}

impl Opening {
    /// Creates a new opening with the given ECO code, name, and moves.
    #[must_use]
    pub fn new(eco: impl Into<String>, name: impl Into<String>, moves: Vec<String>) -> Self {
        Self {
            eco: eco.into(),
            name: name.into(),
            moves,
        }
    }
}

impl OpeningMove {
    /// Creates a new opening move with the given UCI notation and weight.
    #[must_use]
    pub fn new(uci: impl Into<String>, weight: u32) -> Self {
        Self {
            uci: uci.into(),
            weight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opening_new() {
        let opening = Opening::new(
            "C44",
            "King's Pawn Game",
            vec!["e2e4".to_string(), "e7e5".to_string()],
        );
        assert_eq!(opening.eco, "C44");
        assert_eq!(opening.name, "King's Pawn Game");
        assert_eq!(opening.moves.len(), 2);
    }

    #[test]
    fn test_opening_move_new() {
        let mv = OpeningMove::new("e2e4", 100);
        assert_eq!(mv.uci, "e2e4");
        assert_eq!(mv.weight, 100);
    }
}
