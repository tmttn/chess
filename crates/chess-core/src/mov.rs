//! Move representation.

use crate::{Piece, Square};
use std::fmt;

/// Flags for special move types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MoveFlag {
    /// Normal move (no special action).
    Normal = 0,
    /// Pawn double push from starting rank.
    DoublePush = 1,
    /// Kingside castling (O-O).
    CastleKingside = 2,
    /// Queenside castling (O-O-O).
    CastleQueenside = 3,
    /// En passant capture.
    EnPassant = 4,
    /// Pawn promotion to knight.
    PromoteKnight = 5,
    /// Pawn promotion to bishop.
    PromoteBishop = 6,
    /// Pawn promotion to rook.
    PromoteRook = 7,
    /// Pawn promotion to queen.
    PromoteQueen = 8,
}

impl MoveFlag {
    /// Returns the promotion piece if this is a promotion move.
    #[inline]
    pub const fn promotion_piece(self) -> Option<Piece> {
        match self {
            MoveFlag::PromoteKnight => Some(Piece::Knight),
            MoveFlag::PromoteBishop => Some(Piece::Bishop),
            MoveFlag::PromoteRook => Some(Piece::Rook),
            MoveFlag::PromoteQueen => Some(Piece::Queen),
            _ => None,
        }
    }

    /// Returns true if this is a promotion move.
    #[inline]
    pub const fn is_promotion(self) -> bool {
        matches!(
            self,
            MoveFlag::PromoteKnight
                | MoveFlag::PromoteBishop
                | MoveFlag::PromoteRook
                | MoveFlag::PromoteQueen
        )
    }

    /// Returns true if this is a castling move.
    #[inline]
    pub const fn is_castling(self) -> bool {
        matches!(self, MoveFlag::CastleKingside | MoveFlag::CastleQueenside)
    }
}

/// A chess move.
///
/// Encoded compactly: 6 bits from, 6 bits to, 4 bits flags = 16 bits total.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(u16);

impl Move {
    /// Creates a new move.
    #[inline]
    pub const fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        let encoded = (from.index() as u16) | ((to.index() as u16) << 6) | ((flag as u16) << 12);
        Move(encoded)
    }

    /// Creates a normal move (no special flags).
    #[inline]
    pub const fn normal(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveFlag::Normal)
    }

    /// Returns the source square.
    #[inline]
    pub const fn from(self) -> Square {
        // SAFETY: masked to 6 bits, always valid square index
        unsafe { Square::from_index_unchecked((self.0 & 0x3F) as u8) }
    }

    /// Returns the destination square.
    #[inline]
    pub const fn to(self) -> Square {
        // SAFETY: masked to 6 bits, always valid square index
        unsafe { Square::from_index_unchecked(((self.0 >> 6) & 0x3F) as u8) }
    }

    /// Returns the move flag.
    #[inline]
    pub const fn flag(self) -> MoveFlag {
        match (self.0 >> 12) as u8 {
            0 => MoveFlag::Normal,
            1 => MoveFlag::DoublePush,
            2 => MoveFlag::CastleKingside,
            3 => MoveFlag::CastleQueenside,
            4 => MoveFlag::EnPassant,
            5 => MoveFlag::PromoteKnight,
            6 => MoveFlag::PromoteBishop,
            7 => MoveFlag::PromoteRook,
            8 => MoveFlag::PromoteQueen,
            _ => MoveFlag::Normal, // Should never happen
        }
    }

    /// Returns the UCI notation for this move (e.g., "e2e4", "e7e8q").
    pub fn to_uci(self) -> String {
        let promo = match self.flag() {
            MoveFlag::PromoteKnight => "n",
            MoveFlag::PromoteBishop => "b",
            MoveFlag::PromoteRook => "r",
            MoveFlag::PromoteQueen => "q",
            _ => "",
        };
        format!("{}{}{}", self.from(), self.to(), promo)
    }

    /// Parses a move from UCI notation.
    ///
    /// Note: This creates a basic move without full flag inference.
    /// The engine should validate and set proper flags based on position.
    pub fn from_uci(s: &str) -> Option<Self> {
        if s.len() < 4 || s.len() > 5 {
            return None;
        }
        let from = Square::from_algebraic(&s[0..2])?;
        let to = Square::from_algebraic(&s[2..4])?;
        let flag = if s.len() == 5 {
            match s.chars().nth(4)? {
                'n' | 'N' => MoveFlag::PromoteKnight,
                'b' | 'B' => MoveFlag::PromoteBishop,
                'r' | 'R' => MoveFlag::PromoteRook,
                'q' | 'Q' => MoveFlag::PromoteQueen,
                _ => return None,
            }
        } else {
            MoveFlag::Normal
        };
        Some(Move::new(from, to, flag))
    }

    /// A null move (used as placeholder, not a legal move).
    pub const NULL: Move = Move(0);
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Move({})", self.to_uci())
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{File, Rank};

    #[test]
    fn move_encoding() {
        let e2 = Square::new(File::E, Rank::R2);
        let e4 = Square::new(File::E, Rank::R4);
        let m = Move::new(e2, e4, MoveFlag::DoublePush);

        assert_eq!(m.from(), e2);
        assert_eq!(m.to(), e4);
        assert_eq!(m.flag(), MoveFlag::DoublePush);
    }

    #[test]
    fn move_uci() {
        let e2 = Square::new(File::E, Rank::R2);
        let e4 = Square::new(File::E, Rank::R4);
        let m = Move::normal(e2, e4);
        assert_eq!(m.to_uci(), "e2e4");

        let e7 = Square::new(File::E, Rank::R7);
        let e8 = Square::new(File::E, Rank::R8);
        let promo = Move::new(e7, e8, MoveFlag::PromoteQueen);
        assert_eq!(promo.to_uci(), "e7e8q");
    }

    #[test]
    fn move_from_uci() {
        let m = Move::from_uci("e2e4").unwrap();
        assert_eq!(m.from().to_algebraic(), "e2");
        assert_eq!(m.to().to_algebraic(), "e4");

        let promo = Move::from_uci("e7e8q").unwrap();
        assert_eq!(promo.flag(), MoveFlag::PromoteQueen);

        assert!(Move::from_uci("invalid").is_none());
        assert!(Move::from_uci("e2e9").is_none());
    }

    #[test]
    fn move_flag_promotion_piece() {
        assert_eq!(MoveFlag::Normal.promotion_piece(), None);
        assert_eq!(MoveFlag::DoublePush.promotion_piece(), None);
        assert_eq!(MoveFlag::CastleKingside.promotion_piece(), None);
        assert_eq!(MoveFlag::CastleQueenside.promotion_piece(), None);
        assert_eq!(MoveFlag::EnPassant.promotion_piece(), None);
        assert_eq!(
            MoveFlag::PromoteKnight.promotion_piece(),
            Some(crate::Piece::Knight)
        );
        assert_eq!(
            MoveFlag::PromoteBishop.promotion_piece(),
            Some(crate::Piece::Bishop)
        );
        assert_eq!(
            MoveFlag::PromoteRook.promotion_piece(),
            Some(crate::Piece::Rook)
        );
        assert_eq!(
            MoveFlag::PromoteQueen.promotion_piece(),
            Some(crate::Piece::Queen)
        );
    }

    #[test]
    fn move_flag_is_promotion() {
        assert!(!MoveFlag::Normal.is_promotion());
        assert!(!MoveFlag::DoublePush.is_promotion());
        assert!(!MoveFlag::CastleKingside.is_promotion());
        assert!(!MoveFlag::EnPassant.is_promotion());
        assert!(MoveFlag::PromoteKnight.is_promotion());
        assert!(MoveFlag::PromoteBishop.is_promotion());
        assert!(MoveFlag::PromoteRook.is_promotion());
        assert!(MoveFlag::PromoteQueen.is_promotion());
    }

    #[test]
    fn move_flag_is_castling() {
        assert!(!MoveFlag::Normal.is_castling());
        assert!(!MoveFlag::DoublePush.is_castling());
        assert!(MoveFlag::CastleKingside.is_castling());
        assert!(MoveFlag::CastleQueenside.is_castling());
        assert!(!MoveFlag::EnPassant.is_castling());
        assert!(!MoveFlag::PromoteQueen.is_castling());
    }

    #[test]
    fn move_all_promotions_uci() {
        let e7 = Square::new(File::E, Rank::R7);
        let e8 = Square::new(File::E, Rank::R8);

        assert_eq!(Move::new(e7, e8, MoveFlag::PromoteKnight).to_uci(), "e7e8n");
        assert_eq!(Move::new(e7, e8, MoveFlag::PromoteBishop).to_uci(), "e7e8b");
        assert_eq!(Move::new(e7, e8, MoveFlag::PromoteRook).to_uci(), "e7e8r");
        assert_eq!(Move::new(e7, e8, MoveFlag::PromoteQueen).to_uci(), "e7e8q");
    }

    #[test]
    fn move_from_uci_all_promotions() {
        assert_eq!(
            Move::from_uci("e7e8n").unwrap().flag(),
            MoveFlag::PromoteKnight
        );
        assert_eq!(
            Move::from_uci("e7e8N").unwrap().flag(),
            MoveFlag::PromoteKnight
        );
        assert_eq!(
            Move::from_uci("e7e8b").unwrap().flag(),
            MoveFlag::PromoteBishop
        );
        assert_eq!(
            Move::from_uci("e7e8B").unwrap().flag(),
            MoveFlag::PromoteBishop
        );
        assert_eq!(
            Move::from_uci("e7e8r").unwrap().flag(),
            MoveFlag::PromoteRook
        );
        assert_eq!(
            Move::from_uci("e7e8R").unwrap().flag(),
            MoveFlag::PromoteRook
        );
        assert_eq!(
            Move::from_uci("e7e8q").unwrap().flag(),
            MoveFlag::PromoteQueen
        );
        assert_eq!(
            Move::from_uci("e7e8Q").unwrap().flag(),
            MoveFlag::PromoteQueen
        );
        // Invalid promotion character
        assert!(Move::from_uci("e7e8x").is_none());
    }

    #[test]
    fn move_null() {
        let null = Move::NULL;
        assert_eq!(null.from().index(), 0);
        assert_eq!(null.to().index(), 0);
    }

    #[test]
    fn move_debug_display() {
        let e2 = Square::new(File::E, Rank::R2);
        let e4 = Square::new(File::E, Rank::R4);
        let m = Move::normal(e2, e4);
        assert_eq!(format!("{:?}", m), "Move(e2e4)");
        assert_eq!(format!("{}", m), "e2e4");
    }

    #[test]
    fn move_from_uci_edge_cases() {
        // Too short
        assert!(Move::from_uci("e2").is_none());
        assert!(Move::from_uci("e2e").is_none());
        // Too long
        assert!(Move::from_uci("e2e4qq").is_none());
    }
}
