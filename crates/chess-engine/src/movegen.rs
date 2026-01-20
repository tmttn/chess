//! Move generation.
//!
//! This module will contain the move generation logic, including:
//! - Attack table generation (magic bitboards for sliders)
//! - Legal move generation
//! - Move validation

use chess_core::Move;

/// A list of moves with a fixed maximum capacity.
///
/// Chess positions have at most 218 legal moves, so we use a fixed-size
/// array to avoid heap allocations during move generation.
#[derive(Clone)]
pub struct MoveList {
    moves: [Move; Self::MAX_MOVES],
    len: usize,
}

impl MoveList {
    /// Maximum number of legal moves in any chess position.
    pub const MAX_MOVES: usize = 256;

    /// Creates an empty move list.
    #[inline]
    pub const fn new() -> Self {
        MoveList {
            moves: [Move::NULL; Self::MAX_MOVES],
            len: 0,
        }
    }

    /// Adds a move to the list.
    #[inline]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.len < Self::MAX_MOVES);
        self.moves[self.len] = m;
        self.len += 1;
    }

    /// Returns the number of moves.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the list is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a slice of the moves.
    #[inline]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    /// Clears the move list.
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len);
        &self.moves[index]
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = std::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl std::fmt::Debug for MoveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

// TODO: Implement attack tables and move generation
//
// The full implementation will include:
// - Precomputed attack tables for knights and kings
// - Magic bitboard tables for bishops and rooks
// - Pawn push and capture generation
// - Castling move generation
// - En passant detection
// - Pin and check detection for legal move filtering

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::Square;

    #[test]
    fn movelist_push_and_iterate() {
        let mut list = MoveList::new();
        assert!(list.is_empty());

        let e2 = Square::new(chess_core::File::E, chess_core::Rank::R2);
        let e4 = Square::new(chess_core::File::E, chess_core::Rank::R4);
        let d2 = Square::new(chess_core::File::D, chess_core::Rank::R2);
        let d4 = Square::new(chess_core::File::D, chess_core::Rank::R4);

        let m1 = Move::normal(e2, e4);
        let m2 = Move::normal(d2, d4);

        list.push(m1);
        list.push(m2);

        assert_eq!(list.len(), 2);
        assert_eq!(list[0], m1);
        assert_eq!(list[1], m2);
    }
}
