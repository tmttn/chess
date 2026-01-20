//! Bitboard representation and operations.
//!
//! A bitboard is a 64-bit integer where each bit represents a square on the
//! chess board. This allows efficient parallel operations on multiple squares.

use chess_core::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// A 64-bit board representation.
///
/// Bit 0 = a1, bit 1 = b1, ..., bit 63 = h8 (little-endian rank-file mapping).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// Empty bitboard (no squares set).
    pub const EMPTY: Bitboard = Bitboard(0);

    /// Full bitboard (all squares set).
    pub const FULL: Bitboard = Bitboard(!0);

    // File masks
    pub const FILE_A: Bitboard = Bitboard(0x0101_0101_0101_0101);
    pub const FILE_B: Bitboard = Bitboard(0x0202_0202_0202_0202);
    pub const FILE_G: Bitboard = Bitboard(0x4040_4040_4040_4040);
    pub const FILE_H: Bitboard = Bitboard(0x8080_8080_8080_8080);

    // Rank masks
    pub const RANK_1: Bitboard = Bitboard(0x0000_0000_0000_00FF);
    pub const RANK_2: Bitboard = Bitboard(0x0000_0000_0000_FF00);
    pub const RANK_7: Bitboard = Bitboard(0x00FF_0000_0000_0000);
    pub const RANK_8: Bitboard = Bitboard(0xFF00_0000_0000_0000);

    /// Creates a bitboard from a raw u64.
    #[inline]
    pub const fn new(bits: u64) -> Self {
        Bitboard(bits)
    }

    /// Creates a bitboard with a single square set.
    #[inline]
    pub const fn from_square(sq: Square) -> Self {
        Bitboard(1u64 << sq.index())
    }

    /// Returns true if the bitboard is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns true if the bitboard is not empty.
    #[inline]
    pub const fn is_not_empty(self) -> bool {
        self.0 != 0
    }

    /// Returns the number of set bits (population count).
    #[inline]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns true if the given square is set.
    #[inline]
    pub const fn contains(self, sq: Square) -> bool {
        (self.0 & (1u64 << sq.index())) != 0
    }

    /// Sets the given square.
    #[inline]
    pub fn set(&mut self, sq: Square) {
        self.0 |= 1u64 << sq.index();
    }

    /// Clears the given square.
    #[inline]
    pub fn clear(&mut self, sq: Square) {
        self.0 &= !(1u64 << sq.index());
    }

    /// Toggles the given square.
    #[inline]
    pub fn toggle(&mut self, sq: Square) {
        self.0 ^= 1u64 << sq.index();
    }

    /// Returns the index of the least significant bit (0-63).
    /// Returns None if the bitboard is empty.
    #[inline]
    pub const fn lsb(self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some(self.0.trailing_zeros() as u8)
        }
    }

    /// Pops and returns the least significant bit.
    #[inline]
    pub fn pop_lsb(&mut self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            let sq = self.0.trailing_zeros() as u8;
            self.0 &= self.0 - 1; // Clear the LSB
            Some(unsafe { Square::from_index_unchecked(sq) })
        }
    }

    /// Shifts the bitboard north (toward rank 8).
    #[inline]
    pub const fn north(self) -> Bitboard {
        Bitboard(self.0 << 8)
    }

    /// Shifts the bitboard south (toward rank 1).
    #[inline]
    pub const fn south(self) -> Bitboard {
        Bitboard(self.0 >> 8)
    }

    /// Shifts the bitboard east (toward file H).
    #[inline]
    pub const fn east(self) -> Bitboard {
        Bitboard((self.0 << 1) & !Self::FILE_A.0)
    }

    /// Shifts the bitboard west (toward file A).
    #[inline]
    pub const fn west(self) -> Bitboard {
        Bitboard((self.0 >> 1) & !Self::FILE_H.0)
    }

    /// Shifts the bitboard northeast.
    #[inline]
    pub const fn north_east(self) -> Bitboard {
        Bitboard((self.0 << 9) & !Self::FILE_A.0)
    }

    /// Shifts the bitboard northwest.
    #[inline]
    pub const fn north_west(self) -> Bitboard {
        Bitboard((self.0 << 7) & !Self::FILE_H.0)
    }

    /// Shifts the bitboard southeast.
    #[inline]
    pub const fn south_east(self) -> Bitboard {
        Bitboard((self.0 >> 7) & !Self::FILE_A.0)
    }

    /// Shifts the bitboard southwest.
    #[inline]
    pub const fn south_west(self) -> Bitboard {
        Bitboard((self.0 >> 9) & !Self::FILE_H.0)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Bitboard({:#018x})", self.0)?;
        for rank in (0..8).rev() {
            write!(f, "{} ", rank + 1)?;
            for file in 0..8 {
                let sq = rank * 8 + file;
                if (self.0 >> sq) & 1 == 1 {
                    write!(f, "X ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "  a b c d e f g h")
    }
}

/// Iterator over set squares in a bitboard.
pub struct BitboardIter(Bitboard);

impl Iterator for BitboardIter {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_lsb()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.0.count() as usize;
        (count, Some(count))
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = BitboardIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitboardIter(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::{File, Rank};

    #[test]
    fn bitboard_from_square() {
        let bb = Bitboard::from_square(Square::A1);
        assert_eq!(bb.0, 1);
        assert!(bb.contains(Square::A1));
        assert!(!bb.contains(Square::B1));
    }

    #[test]
    fn bitboard_count() {
        assert_eq!(Bitboard::EMPTY.count(), 0);
        assert_eq!(Bitboard::FULL.count(), 64);
        assert_eq!(Bitboard::FILE_A.count(), 8);
        assert_eq!(Bitboard::RANK_1.count(), 8);
    }

    #[test]
    fn bitboard_shifts() {
        let a1 = Bitboard::from_square(Square::A1);
        assert!(a1.north().contains(Square::new(File::A, Rank::R2)));
        assert!(a1.east().contains(Square::B1));
        assert!(a1.north_east().contains(Square::new(File::B, Rank::R2)));
    }

    #[test]
    fn bitboard_iterator() {
        let bb = Bitboard::FILE_A;
        let squares: Vec<Square> = bb.into_iter().collect();
        assert_eq!(squares.len(), 8);
        assert_eq!(squares[0], Square::A1);
    }

    #[test]
    fn bitboard_pop_lsb() {
        let mut bb = Bitboard::new(0b1010);
        assert_eq!(bb.pop_lsb().map(|s| s.index()), Some(1));
        assert_eq!(bb.pop_lsb().map(|s| s.index()), Some(3));
        assert_eq!(bb.pop_lsb(), None);
    }

    #[test]
    fn bitboard_empty_full() {
        assert!(Bitboard::EMPTY.is_empty());
        assert!(!Bitboard::EMPTY.is_not_empty());
        assert!(!Bitboard::FULL.is_empty());
        assert!(Bitboard::FULL.is_not_empty());
    }

    #[test]
    fn bitboard_set_clear_toggle() {
        let mut bb = Bitboard::EMPTY;
        let e4 = Square::new(File::E, Rank::R4);

        bb.set(e4);
        assert!(bb.contains(e4));
        assert_eq!(bb.count(), 1);

        bb.clear(e4);
        assert!(!bb.contains(e4));
        assert!(bb.is_empty());

        bb.toggle(Square::A1);
        assert!(bb.contains(Square::A1));
        bb.toggle(Square::A1);
        assert!(!bb.contains(Square::A1));
    }

    #[test]
    fn bitboard_lsb() {
        assert_eq!(Bitboard::EMPTY.lsb(), None);
        assert_eq!(Bitboard::from_square(Square::A1).lsb(), Some(0));
        assert_eq!(Bitboard::from_square(Square::H8).lsb(), Some(63));
        assert_eq!(Bitboard::new(0b1000).lsb(), Some(3));
    }

    #[test]
    fn bitboard_shifts_comprehensive() {
        let e4 = Square::new(File::E, Rank::R4);
        let bb = Bitboard::from_square(e4);

        // Test all 8 directions from e4
        assert!(bb.north().contains(Square::new(File::E, Rank::R5)));
        assert!(bb.south().contains(Square::new(File::E, Rank::R3)));
        assert!(bb.east().contains(Square::new(File::F, Rank::R4)));
        assert!(bb.west().contains(Square::new(File::D, Rank::R4)));
        assert!(bb.north_east().contains(Square::new(File::F, Rank::R5)));
        assert!(bb.north_west().contains(Square::new(File::D, Rank::R5)));
        assert!(bb.south_east().contains(Square::new(File::F, Rank::R3)));
        assert!(bb.south_west().contains(Square::new(File::D, Rank::R3)));
    }

    #[test]
    fn bitboard_edge_shifts() {
        // Test that edge shifts don't wrap around
        let a_file = Bitboard::FILE_A;
        assert!((a_file.west()).is_empty()); // Can't go west from A file

        let h_file = Bitboard::FILE_H;
        assert!((h_file.east()).is_empty()); // Can't go east from H file

        // North/south edge cases
        let rank_1 = Bitboard::RANK_1;
        // South from rank 1 is empty
        assert_eq!(rank_1.south().0, 0);
    }

    #[test]
    fn bitboard_bitwise_ops() {
        let a = Bitboard::new(0b1100);
        let b = Bitboard::new(0b1010);

        // BitAnd
        assert_eq!((a & b).0, 0b1000);

        // BitOr
        assert_eq!((a | b).0, 0b1110);

        // BitXor
        assert_eq!((a ^ b).0, 0b0110);

        // Not
        assert_eq!((!Bitboard::EMPTY).0, !0u64);
        assert_eq!((!Bitboard::FULL).0, 0u64);
    }

    #[test]
    fn bitboard_bitwise_assign_ops() {
        let mut bb = Bitboard::new(0b1100);
        bb &= Bitboard::new(0b1010);
        assert_eq!(bb.0, 0b1000);

        let mut bb = Bitboard::new(0b1100);
        bb |= Bitboard::new(0b0011);
        assert_eq!(bb.0, 0b1111);

        let mut bb = Bitboard::new(0b1100);
        bb ^= Bitboard::new(0b1010);
        assert_eq!(bb.0, 0b0110);
    }

    #[test]
    fn bitboard_file_rank_constants() {
        // Test file constants
        assert_eq!(Bitboard::FILE_A.count(), 8);
        assert_eq!(Bitboard::FILE_B.count(), 8);
        assert_eq!(Bitboard::FILE_G.count(), 8);
        assert_eq!(Bitboard::FILE_H.count(), 8);

        // Test rank constants
        assert_eq!(Bitboard::RANK_1.count(), 8);
        assert_eq!(Bitboard::RANK_2.count(), 8);
        assert_eq!(Bitboard::RANK_7.count(), 8);
        assert_eq!(Bitboard::RANK_8.count(), 8);

        // Test specific squares in files/ranks
        assert!(Bitboard::FILE_A.contains(Square::A1));
        assert!(Bitboard::FILE_A.contains(Square::A8));
        assert!(!Bitboard::FILE_A.contains(Square::B1));

        assert!(Bitboard::RANK_1.contains(Square::A1));
        assert!(Bitboard::RANK_1.contains(Square::H1));
        assert!(!Bitboard::RANK_1.contains(Square::new(File::A, Rank::R2)));
    }

    #[test]
    fn bitboard_iterator_size_hint() {
        let bb = Bitboard::new(0b1010101);
        let iter = bb.into_iter();
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 4);
        assert_eq!(upper, Some(4));
    }

    #[test]
    fn bitboard_iterator_empty() {
        let bb = Bitboard::EMPTY;
        let squares: Vec<Square> = bb.into_iter().collect();
        assert!(squares.is_empty());
    }

    #[test]
    fn bitboard_debug() {
        let e4 = Square::new(File::E, Rank::R4);
        let bb = Bitboard::from_square(e4);
        let debug_str = format!("{:?}", bb);
        assert!(debug_str.contains("Bitboard("));
        assert!(debug_str.contains("X")); // The set square
        assert!(debug_str.contains(".")); // Empty squares
        assert!(debug_str.contains("a b c d e f g h")); // File labels
    }

    #[test]
    fn bitboard_default() {
        let bb = Bitboard::default();
        assert_eq!(bb, Bitboard::EMPTY);
    }

    #[test]
    fn bitboard_new() {
        let bb = Bitboard::new(0x12345678);
        assert_eq!(bb.0, 0x12345678);
    }
}
