//! Attack table generation and lookup for all piece types.

use crate::Bitboard;
use chess_core::{Color, Square};

pub use super::magics::{bishop_attacks, queen_attacks, rook_attacks};

/// Precomputed knight attack tables.
const KNIGHT_ATTACKS: [Bitboard; 64] = compute_knight_attacks();

/// Precomputed king attack tables.
const KING_ATTACKS: [Bitboard; 64] = compute_king_attacks();

/// Precomputed pawn attack tables [color][square].
const PAWN_ATTACKS: [[Bitboard; 64]; 2] = compute_pawn_attacks();

/// Returns knight attacks from the given square.
#[inline]
pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq.index() as usize]
}

/// Returns king attacks from the given square.
#[inline]
pub fn king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS[sq.index() as usize]
}

/// Returns pawn attacks from the given square for the given color.
#[inline]
pub fn pawn_attacks(sq: Square, color: Color) -> Bitboard {
    PAWN_ATTACKS[color.index()][sq.index() as usize]
}

/// Computes knight attacks for all squares at compile time.
const fn compute_knight_attacks() -> [Bitboard; 64] {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut sq = 0u8;

    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut bb = 0u64;

        // Knight move offsets: (rank_delta, file_delta)
        // (+2, +1), (+2, -1), (-2, +1), (-2, -1)
        // (+1, +2), (+1, -2), (-1, +2), (-1, -2)

        if rank < 6 && file < 7 {
            bb |= 1u64 << (sq + 17);
        } // +2, +1
        if rank < 6 && file > 0 {
            bb |= 1u64 << (sq + 15);
        } // +2, -1
        if rank > 1 && file < 7 {
            bb |= 1u64 << (sq - 15);
        } // -2, +1
        if rank > 1 && file > 0 {
            bb |= 1u64 << (sq - 17);
        } // -2, -1
        if rank < 7 && file < 6 {
            bb |= 1u64 << (sq + 10);
        } // +1, +2
        if rank < 7 && file > 1 {
            bb |= 1u64 << (sq + 6);
        } // +1, -2
        if rank > 0 && file < 6 {
            bb |= 1u64 << (sq - 6);
        } // -1, +2
        if rank > 0 && file > 1 {
            bb |= 1u64 << (sq - 10);
        } // -1, -2

        attacks[sq as usize] = Bitboard(bb);
        sq += 1;
    }

    attacks
}

/// Computes king attacks for all squares at compile time.
const fn compute_king_attacks() -> [Bitboard; 64] {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut sq = 0u8;

    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut bb = 0u64;

        // King moves in all 8 directions
        if rank < 7 {
            bb |= 1u64 << (sq + 8);
        } // North
        if rank > 0 {
            bb |= 1u64 << (sq - 8);
        } // South
        if file < 7 {
            bb |= 1u64 << (sq + 1);
        } // East
        if file > 0 {
            bb |= 1u64 << (sq - 1);
        } // West
        if rank < 7 && file < 7 {
            bb |= 1u64 << (sq + 9);
        } // NE
        if rank < 7 && file > 0 {
            bb |= 1u64 << (sq + 7);
        } // NW
        if rank > 0 && file < 7 {
            bb |= 1u64 << (sq - 7);
        } // SE
        if rank > 0 && file > 0 {
            bb |= 1u64 << (sq - 9);
        } // SW

        attacks[sq as usize] = Bitboard(bb);
        sq += 1;
    }

    attacks
}

/// Computes pawn attacks for all squares at compile time.
const fn compute_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut attacks = [[Bitboard::EMPTY; 64]; 2];
    let mut sq = 0u8;

    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;

        // White pawns attack diagonally forward (north)
        let mut white_bb = 0u64;
        if rank < 7 && file < 7 {
            white_bb |= 1u64 << (sq + 9);
        } // NE
        if rank < 7 && file > 0 {
            white_bb |= 1u64 << (sq + 7);
        } // NW
        attacks[0][sq as usize] = Bitboard(white_bb);

        // Black pawns attack diagonally forward (south)
        let mut black_bb = 0u64;
        if rank > 0 && file < 7 {
            black_bb |= 1u64 << (sq - 7);
        } // SE
        if rank > 0 && file > 0 {
            black_bb |= 1u64 << (sq - 9);
        } // SW
        attacks[1][sq as usize] = Bitboard(black_bb);

        sq += 1;
    }

    attacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::{File, Rank};

    #[test]
    fn knight_attacks_center() {
        let sq = Square::new(File::D, Rank::R4);
        let attacks = knight_attacks(sq);
        assert_eq!(attacks.count(), 8); // Knight in center has 8 moves
    }

    #[test]
    fn knight_attacks_corner() {
        let attacks = knight_attacks(Square::A1);
        assert_eq!(attacks.count(), 2); // Corner knight has 2 moves
    }

    #[test]
    fn knight_attacks_edge() {
        let sq = Square::new(File::A, Rank::R4);
        let attacks = knight_attacks(sq);
        assert_eq!(attacks.count(), 4); // Edge knight has 4 moves
    }

    #[test]
    fn king_attacks_center() {
        let sq = Square::new(File::D, Rank::R4);
        let attacks = king_attacks(sq);
        assert_eq!(attacks.count(), 8); // King in center has 8 moves
    }

    #[test]
    fn king_attacks_corner() {
        let attacks = king_attacks(Square::A1);
        assert_eq!(attacks.count(), 3); // Corner king has 3 moves
    }

    #[test]
    fn king_attacks_edge() {
        let sq = Square::new(File::A, Rank::R4);
        let attacks = king_attacks(sq);
        assert_eq!(attacks.count(), 5); // Edge king has 5 moves
    }

    #[test]
    fn pawn_attacks_white() {
        let sq = Square::new(File::D, Rank::R4);
        let attacks = pawn_attacks(sq, Color::White);
        assert_eq!(attacks.count(), 2);
        assert!(attacks.contains(Square::new(File::C, Rank::R5)));
        assert!(attacks.contains(Square::new(File::E, Rank::R5)));
    }

    #[test]
    fn pawn_attacks_black() {
        let sq = Square::new(File::D, Rank::R4);
        let attacks = pawn_attacks(sq, Color::Black);
        assert_eq!(attacks.count(), 2);
        assert!(attacks.contains(Square::new(File::C, Rank::R3)));
        assert!(attacks.contains(Square::new(File::E, Rank::R3)));
    }

    #[test]
    fn pawn_attacks_edge_file() {
        // A-file pawn can only attack one direction
        let sq = Square::new(File::A, Rank::R4);
        let white_attacks = pawn_attacks(sq, Color::White);
        assert_eq!(white_attacks.count(), 1);
        assert!(white_attacks.contains(Square::new(File::B, Rank::R5)));
    }

    #[test]
    fn pawn_attacks_promotion_rank() {
        // White pawn on rank 8 has no attacks (shouldn't happen in real game)
        let sq = Square::new(File::D, Rank::R8);
        let attacks = pawn_attacks(sq, Color::White);
        assert_eq!(attacks.count(), 0);
    }

    #[test]
    fn knight_specific_squares() {
        // Test knight on e4 attacks specific squares
        let sq = Square::new(File::E, Rank::R4);
        let attacks = knight_attacks(sq);
        assert!(attacks.contains(Square::new(File::D, Rank::R6))); // c6? no, d6
        assert!(attacks.contains(Square::new(File::F, Rank::R6)));
        assert!(attacks.contains(Square::new(File::G, Rank::R5)));
        assert!(attacks.contains(Square::new(File::G, Rank::R3)));
        assert!(attacks.contains(Square::new(File::F, Rank::R2)));
        assert!(attacks.contains(Square::new(File::D, Rank::R2)));
        assert!(attacks.contains(Square::new(File::C, Rank::R3)));
        assert!(attacks.contains(Square::new(File::C, Rank::R5)));
    }
}
