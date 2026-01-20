//! Built-in opening book data.
//!
//! This module provides access to the built-in opening database
//! that is compiled into the library.

use crate::database::OpeningDatabase;
use crate::opening::OpeningMove;

/// Creates the built-in opening database with common chess openings.
///
/// This database includes popular openings and their main lines,
/// weighted by frequency of play at master level.
#[must_use]
pub fn builtin_database() -> OpeningDatabase {
    let mut db = OpeningDatabase::new();

    // Starting position - most common first moves
    db.add_position(
        "",
        vec![
            OpeningMove::new("e2e4", 100), // King's Pawn
            OpeningMove::new("d2d4", 90),  // Queen's Pawn
            OpeningMove::new("c2c4", 40),  // English
            OpeningMove::new("g1f3", 30),  // Reti
        ],
    );

    // After 1.e4
    db.add_position(
        "e2e4",
        vec![
            OpeningMove::new("e7e5", 80), // Open Game
            OpeningMove::new("c7c5", 70), // Sicilian
            OpeningMove::new("e7e6", 40), // French
            OpeningMove::new("c7c6", 30), // Caro-Kann
            OpeningMove::new("d7d5", 20), // Scandinavian
        ],
    );

    // After 1.d4
    db.add_position(
        "d2d4",
        vec![
            OpeningMove::new("d7d5", 80), // Closed Game
            OpeningMove::new("g8f6", 70), // Indian Defenses
            OpeningMove::new("e7e6", 30), // Dutch setup
            OpeningMove::new("f7f5", 10), // Dutch Defense
        ],
    );

    // After 1.e4 e5
    db.add_position(
        "e2e4 e7e5",
        vec![
            OpeningMove::new("g1f3", 90), // King's Knight
            OpeningMove::new("f1c4", 30), // Bishop's Opening
            OpeningMove::new("b1c3", 20), // Vienna Game
        ],
    );

    // After 1.e4 e5 2.Nf3
    db.add_position(
        "e2e4 e7e5 g1f3",
        vec![
            OpeningMove::new("b8c6", 90), // Knight's Defense
            OpeningMove::new("g8f6", 40), // Petrov's Defense
            OpeningMove::new("d7d6", 20), // Philidor Defense
        ],
    );

    // After 1.e4 c5 (Sicilian)
    db.add_position(
        "e2e4 c7c5",
        vec![
            OpeningMove::new("g1f3", 80), // Open Sicilian
            OpeningMove::new("b1c3", 40), // Closed Sicilian
            OpeningMove::new("c2c3", 30), // Alapin
        ],
    );

    // After 1.d4 d5
    db.add_position(
        "d2d4 d7d5",
        vec![
            OpeningMove::new("c2c4", 90), // Queen's Gambit
            OpeningMove::new("g1f3", 40), // London/Colle
            OpeningMove::new("c1f4", 30), // London System
        ],
    );

    // After 1.d4 Nf6
    db.add_position(
        "d2d4 g8f6",
        vec![
            OpeningMove::new("c2c4", 90), // Main line
            OpeningMove::new("g1f3", 40), // Slow approach
            OpeningMove::new("c1f4", 30), // London System
        ],
    );

    db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_database_not_empty() {
        let db = builtin_database();
        assert!(!db.is_empty());
    }

    #[test]
    fn test_starting_position_has_moves() {
        let db = builtin_database();
        let moves = db.lookup("").unwrap();
        assert!(!moves.is_empty());
        // e2e4 should be the most common
        assert_eq!(moves[0].uci, "e2e4");
    }

    #[test]
    fn test_e4_response() {
        let db = builtin_database();
        let moves = db.lookup("e2e4").unwrap();
        assert!(!moves.is_empty());
        // e7e5 should be available as a response
        assert!(moves.iter().any(|m| m.uci == "e7e5"));
    }

    #[test]
    fn test_sicilian_continuation() {
        let db = builtin_database();
        let moves = db.lookup("e2e4 c7c5").unwrap();
        assert!(!moves.is_empty());
        // Nf3 should be the main response
        assert!(moves.iter().any(|m| m.uci == "g1f3"));
    }
}
