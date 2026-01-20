//! Chess position evaluation types.

/// Represents a chess position evaluation.
///
/// Evaluations can be either centipawn scores (for normal positions)
/// or mate scores (when a forced mate is found).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evaluation {
    /// Centipawn evaluation (positive = white advantage)
    Centipawns(i32),
    /// Mate in N moves (positive = white wins, negative = black wins)
    Mate(i32),
}
