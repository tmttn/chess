//! Chess piece representation.

use crate::Color;

/// The six types of chess pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    /// All piece types in order.
    pub const ALL: [Piece; 6] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    /// Returns the index of this piece type (0-5).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns the FEN character for this piece with the given color.
    pub const fn to_fen_char(self, color: Color) -> char {
        let c = match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        };
        match color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    /// Parses a FEN character into a piece and color.
    pub const fn from_fen_char(c: char) -> Option<(Piece, Color)> {
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let piece = match c.to_ascii_lowercase() {
            'p' => Piece::Pawn,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'r' => Piece::Rook,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => return None,
        };
        Some((piece, color))
    }

    /// Returns true if this piece is a sliding piece (bishop, rook, or queen).
    #[inline]
    pub const fn is_slider(self) -> bool {
        matches!(self, Piece::Bishop | Piece::Rook | Piece::Queen)
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Piece::Pawn => "Pawn",
            Piece::Knight => "Knight",
            Piece::Bishop => "Bishop",
            Piece::Rook => "Rook",
            Piece::Queen => "Queen",
            Piece::King => "King",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_to_fen() {
        assert_eq!(Piece::Pawn.to_fen_char(Color::White), 'P');
        assert_eq!(Piece::Pawn.to_fen_char(Color::Black), 'p');
        assert_eq!(Piece::King.to_fen_char(Color::White), 'K');
        assert_eq!(Piece::Knight.to_fen_char(Color::Black), 'n');
    }

    #[test]
    fn piece_from_fen() {
        assert_eq!(Piece::from_fen_char('P'), Some((Piece::Pawn, Color::White)));
        assert_eq!(Piece::from_fen_char('p'), Some((Piece::Pawn, Color::Black)));
        assert_eq!(Piece::from_fen_char('K'), Some((Piece::King, Color::White)));
        assert_eq!(Piece::from_fen_char('x'), None);
    }

    #[test]
    fn is_slider() {
        assert!(!Piece::Pawn.is_slider());
        assert!(!Piece::Knight.is_slider());
        assert!(Piece::Bishop.is_slider());
        assert!(Piece::Rook.is_slider());
        assert!(Piece::Queen.is_slider());
        assert!(!Piece::King.is_slider());
    }

    #[test]
    fn piece_index() {
        assert_eq!(Piece::Pawn.index(), 0);
        assert_eq!(Piece::Knight.index(), 1);
        assert_eq!(Piece::Bishop.index(), 2);
        assert_eq!(Piece::Rook.index(), 3);
        assert_eq!(Piece::Queen.index(), 4);
        assert_eq!(Piece::King.index(), 5);
    }

    #[test]
    fn piece_all() {
        assert_eq!(Piece::ALL.len(), 6);
        assert_eq!(Piece::ALL[0], Piece::Pawn);
        assert_eq!(Piece::ALL[5], Piece::King);
    }

    #[test]
    fn piece_display() {
        assert_eq!(format!("{}", Piece::Pawn), "Pawn");
        assert_eq!(format!("{}", Piece::Knight), "Knight");
        assert_eq!(format!("{}", Piece::Bishop), "Bishop");
        assert_eq!(format!("{}", Piece::Rook), "Rook");
        assert_eq!(format!("{}", Piece::Queen), "Queen");
        assert_eq!(format!("{}", Piece::King), "King");
    }

    #[test]
    fn all_fen_chars() {
        // Test all piece types with both colors
        for piece in Piece::ALL {
            let white_char = piece.to_fen_char(Color::White);
            let black_char = piece.to_fen_char(Color::Black);
            assert!(white_char.is_ascii_uppercase());
            assert!(black_char.is_ascii_lowercase());
            assert_eq!(
                Piece::from_fen_char(white_char),
                Some((piece, Color::White))
            );
            assert_eq!(
                Piece::from_fen_char(black_char),
                Some((piece, Color::Black))
            );
        }
    }
}
