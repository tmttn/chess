//! Board square representation.

use std::fmt;

/// A file (column) on the chess board, from A to H.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {
    /// All files in order.
    pub const ALL: [File; 8] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    /// Creates a file from index (0-7).
    #[inline]
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(File::A),
            1 => Some(File::B),
            2 => Some(File::C),
            3 => Some(File::D),
            4 => Some(File::E),
            5 => Some(File::F),
            6 => Some(File::G),
            7 => Some(File::H),
            _ => None,
        }
    }

    /// Creates a file from a character ('a'-'h' or 'A'-'H').
    #[inline]
    pub const fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'a' => Some(File::A),
            'b' => Some(File::B),
            'c' => Some(File::C),
            'd' => Some(File::D),
            'e' => Some(File::E),
            'f' => Some(File::F),
            'g' => Some(File::G),
            'h' => Some(File::H),
            _ => None,
        }
    }

    /// Returns the index (0-7).
    #[inline]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Returns the character representation.
    #[inline]
    pub const fn to_char(self) -> char {
        (b'a' + self as u8) as char
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// A rank (row) on the chess board, from 1 to 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Rank {
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7,
}

impl Rank {
    /// All ranks in order.
    pub const ALL: [Rank; 8] = [
        Rank::R1,
        Rank::R2,
        Rank::R3,
        Rank::R4,
        Rank::R5,
        Rank::R6,
        Rank::R7,
        Rank::R8,
    ];

    /// Creates a rank from index (0-7).
    #[inline]
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Rank::R1),
            1 => Some(Rank::R2),
            2 => Some(Rank::R3),
            3 => Some(Rank::R4),
            4 => Some(Rank::R5),
            5 => Some(Rank::R6),
            6 => Some(Rank::R7),
            7 => Some(Rank::R8),
            _ => None,
        }
    }

    /// Creates a rank from a character ('1'-'8').
    #[inline]
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            '1' => Some(Rank::R1),
            '2' => Some(Rank::R2),
            '3' => Some(Rank::R3),
            '4' => Some(Rank::R4),
            '5' => Some(Rank::R5),
            '6' => Some(Rank::R6),
            '7' => Some(Rank::R7),
            '8' => Some(Rank::R8),
            _ => None,
        }
    }

    /// Returns the index (0-7).
    #[inline]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Returns the character representation.
    #[inline]
    pub const fn to_char(self) -> char {
        (b'1' + self as u8) as char
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// A square on the chess board, indexed 0-63.
///
/// Squares are indexed in little-endian rank-file mapping:
/// - a1 = 0, b1 = 1, ..., h1 = 7
/// - a2 = 8, ..., h8 = 63
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(u8);

impl Square {
    /// Creates a square from file and rank.
    #[inline]
    pub const fn new(file: File, rank: Rank) -> Self {
        Square(rank.index() * 8 + file.index())
    }

    /// Creates a square from index (0-63).
    #[inline]
    pub const fn from_index(index: u8) -> Option<Self> {
        if index < 64 {
            Some(Square(index))
        } else {
            None
        }
    }

    /// Creates a square from index without bounds checking.
    ///
    /// # Safety
    /// The index must be in the range 0-63.
    #[inline]
    pub const unsafe fn from_index_unchecked(index: u8) -> Self {
        debug_assert!(index < 64);
        Square(index)
    }

    /// Parses a square from algebraic notation (e.g., "e4").
    pub const fn from_algebraic(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let file = match File::from_char(bytes[0] as char) {
            Some(f) => f,
            None => return None,
        };
        let rank = match Rank::from_char(bytes[1] as char) {
            Some(r) => r,
            None => return None,
        };
        Some(Square::new(file, rank))
    }

    /// Returns the index (0-63).
    #[inline]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Returns the file of this square.
    #[inline]
    pub const fn file(self) -> File {
        // SAFETY: self.0 % 8 is always in 0-7
        match File::from_index(self.0 % 8) {
            Some(f) => f,
            None => unreachable!(),
        }
    }

    /// Returns the rank of this square.
    #[inline]
    pub const fn rank(self) -> Rank {
        // SAFETY: self.0 / 8 is always in 0-7
        match Rank::from_index(self.0 / 8) {
            Some(r) => r,
            None => unreachable!(),
        }
    }

    /// Returns the algebraic notation for this square.
    pub fn to_algebraic(self) -> String {
        format!("{}{}", self.file(), self.rank())
    }

    /// Returns a bitboard with only this square set.
    #[inline]
    pub const fn bitboard(self) -> u64 {
        1u64 << self.0
    }

    // Common squares
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);
    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);
}

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Square({})", self.to_algebraic())
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_new() {
        let e4 = Square::new(File::E, Rank::R4);
        assert_eq!(e4.file(), File::E);
        assert_eq!(e4.rank(), Rank::R4);
        assert_eq!(e4.index(), 28);
    }

    #[test]
    fn square_from_algebraic() {
        assert_eq!(Square::from_algebraic("a1"), Some(Square::A1));
        assert_eq!(
            Square::from_algebraic("e4"),
            Some(Square::new(File::E, Rank::R4))
        );
        assert_eq!(Square::from_algebraic("h8"), Some(Square::H8));
        assert_eq!(Square::from_algebraic("i1"), None);
        assert_eq!(Square::from_algebraic("a9"), None);
        assert_eq!(Square::from_algebraic(""), None);
    }

    #[test]
    fn square_to_algebraic() {
        assert_eq!(Square::A1.to_algebraic(), "a1");
        assert_eq!(Square::H8.to_algebraic(), "h8");
        assert_eq!(Square::new(File::E, Rank::R4).to_algebraic(), "e4");
    }

    #[test]
    fn square_bitboard() {
        assert_eq!(Square::A1.bitboard(), 1);
        assert_eq!(Square::H1.bitboard(), 128);
        assert_eq!(Square::A8.bitboard(), 1 << 56);
    }
}
