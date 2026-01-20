//! Opening database storage and lookup.

use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::Rng;
use thiserror::Error;

use crate::opening::OpeningMove;

/// Errors that can occur when working with opening databases.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Failed to parse the opening database.
    #[error("failed to parse opening database: {0}")]
    ParseError(String),

    /// Failed to read the opening database file.
    #[error("failed to read opening database: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON deserialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// An opening book database that maps positions to candidate moves.
#[derive(Debug, Clone, Default)]
pub struct OpeningDatabase {
    /// Maps position keys (move history as string) to candidate moves.
    positions: HashMap<String, Vec<OpeningMove>>,
}

impl OpeningDatabase {
    /// Creates a new empty opening database.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the database is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Returns the number of positions in the database.
    #[must_use]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Adds a position with its candidate moves to the database.
    pub fn add_position(&mut self, position_key: impl Into<String>, moves: Vec<OpeningMove>) {
        self.positions.insert(position_key.into(), moves);
    }

    /// Looks up candidate moves for a position.
    #[must_use]
    pub fn lookup(&self, position_key: &str) -> Option<&[OpeningMove]> {
        self.positions.get(position_key).map(|v| v.as_slice())
    }

    /// Selects a random move from the candidates, weighted by their weights.
    pub fn select_move<R: Rng>(&self, position_key: &str, rng: &mut R) -> Option<&OpeningMove> {
        let moves = self.lookup(position_key)?;
        if moves.is_empty() {
            return None;
        }

        // Calculate total weight
        let total_weight: u32 = moves.iter().map(|m| m.weight).sum();
        if total_weight == 0 {
            // If all weights are zero, select uniformly
            return moves.choose(rng);
        }

        // Weighted random selection
        let mut choice = rng.gen_range(0..total_weight);
        for mv in moves {
            if choice < mv.weight {
                return Some(mv);
            }
            choice -= mv.weight;
        }

        // Fallback (shouldn't happen)
        moves.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_database() {
        let db = OpeningDatabase::new();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
        assert!(db.lookup("e2e4").is_none());
    }

    #[test]
    fn test_add_and_lookup() {
        let mut db = OpeningDatabase::new();
        db.add_position(
            "",
            vec![OpeningMove::new("e2e4", 100), OpeningMove::new("d2d4", 80)],
        );

        assert!(!db.is_empty());
        assert_eq!(db.len(), 1);

        let moves = db.lookup("").unwrap();
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].uci, "e2e4");
    }

    #[test]
    fn test_select_move() {
        let mut db = OpeningDatabase::new();
        db.add_position("", vec![OpeningMove::new("e2e4", 100)]);

        let mut rng = rand::thread_rng();
        let selected = db.select_move("", &mut rng).unwrap();
        assert_eq!(selected.uci, "e2e4");
    }
}
