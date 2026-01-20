//! Player color representation.

/// Represents the two players in chess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    /// Returns the opposite color.
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Returns the index (0 for White, 1 for Black).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns the pawn direction for this color (+1 for White, -1 for Black).
    #[inline]
    pub const fn pawn_direction(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    /// Returns the back rank for this color (0 for White, 7 for Black).
    #[inline]
    pub const fn back_rank(self) -> u8 {
        match self {
            Color::White => 0,
            Color::Black => 7,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_color() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn color_index() {
        assert_eq!(Color::White.index(), 0);
        assert_eq!(Color::Black.index(), 1);
    }

    #[test]
    fn pawn_direction() {
        assert_eq!(Color::White.pawn_direction(), 1);
        assert_eq!(Color::Black.pawn_direction(), -1);
    }

    #[test]
    fn back_rank() {
        assert_eq!(Color::White.back_rank(), 0);
        assert_eq!(Color::Black.back_rank(), 7);
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Color::White), "White");
        assert_eq!(format!("{}", Color::Black), "Black");
    }
}
