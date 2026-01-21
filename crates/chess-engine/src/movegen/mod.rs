//! Move generation.
//!
//! This module provides legal move generation for chess positions using
//! magic bitboards for efficient sliding piece attack calculation.

mod attacks;
mod magics;
pub mod perft;

use crate::{Bitboard, Position};
use chess_core::{Color, Move, MoveFlag, Piece, Rank, Square};

pub use attacks::{
    bishop_attacks, king_attacks, knight_attacks, pawn_attacks, queen_attacks, rook_attacks,
};

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

    /// Retains only moves for which the predicate returns true.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Move) -> bool,
    {
        let mut write = 0;
        for read in 0..self.len {
            if f(&self.moves[read]) {
                self.moves[write] = self.moves[read];
                write += 1;
            }
        }
        self.len = write;
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

/// Generates all legal moves for the given position.
pub fn generate_moves(position: &Position) -> MoveList {
    let mut moves = MoveList::new();

    generate_pawn_moves(position, &mut moves);
    generate_knight_moves(position, &mut moves);
    generate_bishop_moves(position, &mut moves);
    generate_rook_moves(position, &mut moves);
    generate_queen_moves(position, &mut moves);
    generate_king_moves(position, &mut moves);
    generate_castling_moves(position, &mut moves);

    // Filter out moves that leave king in check
    let us = position.side_to_move;
    moves.retain(|m| {
        let new_pos = make_move(position, *m);
        !is_king_attacked(&new_pos, us)
    });

    moves
}

/// Generates pseudo-legal pawn moves.
fn generate_pawn_moves(position: &Position, moves: &mut MoveList) {
    let us = position.side_to_move;
    let them = us.opposite();
    let our_pieces = position.colors[us.index()];
    let their_pieces = position.colors[them.index()];
    let occupied = our_pieces | their_pieces;
    let empty = !occupied;

    let pawns = position.pieces_of(Piece::Pawn, us);

    let (push_dir, _start_rank, promo_rank, _double_rank) = match us {
        Color::White => (8i8, Rank::R2, Rank::R8, Rank::R4),
        Color::Black => (-8i8, Rank::R7, Rank::R1, Rank::R5),
    };

    // Single pushes
    let single_pushes = if us == Color::White {
        pawns.north() & empty
    } else {
        pawns.south() & empty
    };

    for to in single_pushes {
        let from = unsafe { Square::from_index_unchecked((to.index() as i8 - push_dir) as u8) };
        if to.rank() == promo_rank {
            // Promotion
            moves.push(Move::new(from, to, MoveFlag::PromoteQueen));
            moves.push(Move::new(from, to, MoveFlag::PromoteRook));
            moves.push(Move::new(from, to, MoveFlag::PromoteBishop));
            moves.push(Move::new(from, to, MoveFlag::PromoteKnight));
        } else {
            moves.push(Move::normal(from, to));
        }
    }

    // Double pushes
    let double_pushes = if us == Color::White {
        (pawns & Bitboard::RANK_2).north().north() & empty & (single_pushes.north())
    } else {
        (pawns & Bitboard::RANK_7).south().south() & empty & (single_pushes.south())
    };

    for to in double_pushes {
        let from = unsafe { Square::from_index_unchecked((to.index() as i8 - 2 * push_dir) as u8) };
        moves.push(Move::new(from, to, MoveFlag::DoublePush));
    }

    // Captures (left and right)
    for from in pawns {
        let attacks = pawn_attacks(from, us) & their_pieces;
        for to in attacks {
            if to.rank() == promo_rank {
                moves.push(Move::new(from, to, MoveFlag::PromoteQueen));
                moves.push(Move::new(from, to, MoveFlag::PromoteRook));
                moves.push(Move::new(from, to, MoveFlag::PromoteBishop));
                moves.push(Move::new(from, to, MoveFlag::PromoteKnight));
            } else {
                moves.push(Move::normal(from, to));
            }
        }
    }

    // En passant
    if let Some(ep_square) = position.en_passant {
        for from in pawns {
            if pawn_attacks(from, us).contains(ep_square) {
                moves.push(Move::new(from, ep_square, MoveFlag::EnPassant));
            }
        }
    }
}

/// Generates pseudo-legal knight moves.
fn generate_knight_moves(position: &Position, moves: &mut MoveList) {
    let us = position.side_to_move;
    let our_pieces = position.colors[us.index()];
    let knights = position.pieces_of(Piece::Knight, us);

    for from in knights {
        let attacks = knight_attacks(from) & !our_pieces;
        for to in attacks {
            moves.push(Move::normal(from, to));
        }
    }
}

/// Generates pseudo-legal bishop moves.
fn generate_bishop_moves(position: &Position, moves: &mut MoveList) {
    let us = position.side_to_move;
    let our_pieces = position.colors[us.index()];
    let occupied = position.occupied();
    let bishops = position.pieces_of(Piece::Bishop, us);

    for from in bishops {
        let attacks = bishop_attacks(from, occupied) & !our_pieces;
        for to in attacks {
            moves.push(Move::normal(from, to));
        }
    }
}

/// Generates pseudo-legal rook moves.
fn generate_rook_moves(position: &Position, moves: &mut MoveList) {
    let us = position.side_to_move;
    let our_pieces = position.colors[us.index()];
    let occupied = position.occupied();
    let rooks = position.pieces_of(Piece::Rook, us);

    for from in rooks {
        let attacks = rook_attacks(from, occupied) & !our_pieces;
        for to in attacks {
            moves.push(Move::normal(from, to));
        }
    }
}

/// Generates pseudo-legal queen moves.
fn generate_queen_moves(position: &Position, moves: &mut MoveList) {
    let us = position.side_to_move;
    let our_pieces = position.colors[us.index()];
    let occupied = position.occupied();
    let queens = position.pieces_of(Piece::Queen, us);

    for from in queens {
        let attacks = queen_attacks(from, occupied) & !our_pieces;
        for to in attacks {
            moves.push(Move::normal(from, to));
        }
    }
}

/// Generates pseudo-legal king moves (not including castling).
fn generate_king_moves(position: &Position, moves: &mut MoveList) {
    let us = position.side_to_move;
    let our_pieces = position.colors[us.index()];
    let king_sq = position.pieces_of(Piece::King, us).lsb();

    if let Some(idx) = king_sq {
        let from = unsafe { Square::from_index_unchecked(idx) };
        let attacks = king_attacks(from) & !our_pieces;
        for to in attacks {
            moves.push(Move::normal(from, to));
        }
    }
}

/// Generates castling moves if legal.
fn generate_castling_moves(position: &Position, moves: &mut MoveList) {
    let us = position.side_to_move;
    let occupied = position.occupied();

    // Can't castle if in check
    if is_king_attacked(position, us) {
        return;
    }

    let (king_start, king_side_target, queen_side_target, _king_side_rook, _queen_side_rook) =
        match us {
            Color::White => (Square::E1, Square::G1, Square::C1, Square::H1, Square::A1),
            Color::Black => (Square::E8, Square::G8, Square::C8, Square::H8, Square::A8),
        };

    // Kingside castling
    if position.castling.can_castle_kingside(us) {
        let between = match us {
            Color::White => Bitboard::from_square(Square::F1) | Bitboard::from_square(Square::G1),
            Color::Black => Bitboard::from_square(Square::F8) | Bitboard::from_square(Square::G8),
        };
        let pass_through = match us {
            Color::White => Square::F1,
            Color::Black => Square::F8,
        };

        if (occupied & between).is_empty()
            && !is_square_attacked(position, pass_through, us.opposite())
        {
            moves.push(Move::new(
                king_start,
                king_side_target,
                MoveFlag::CastleKingside,
            ));
        }
    }

    // Queenside castling
    if position.castling.can_castle_queenside(us) {
        let between = match us {
            Color::White => {
                Bitboard::from_square(Square::B1)
                    | Bitboard::from_square(Square::C1)
                    | Bitboard::from_square(Square::D1)
            }
            Color::Black => {
                Bitboard::from_square(Square::B8)
                    | Bitboard::from_square(Square::C8)
                    | Bitboard::from_square(Square::D8)
            }
        };
        let pass_through = match us {
            Color::White => Square::D1,
            Color::Black => Square::D8,
        };

        if (occupied & between).is_empty()
            && !is_square_attacked(position, pass_through, us.opposite())
        {
            moves.push(Move::new(
                king_start,
                queen_side_target,
                MoveFlag::CastleQueenside,
            ));
        }
    }
}

/// Returns true if the given square is attacked by the given color.
pub fn is_square_attacked(position: &Position, sq: Square, by_color: Color) -> bool {
    let occupied = position.occupied();

    // Pawn attacks
    let enemy_pawns = position.pieces_of(Piece::Pawn, by_color);
    if (pawn_attacks(sq, by_color.opposite()) & enemy_pawns).is_not_empty() {
        return true;
    }

    // Knight attacks
    let enemy_knights = position.pieces_of(Piece::Knight, by_color);
    if (knight_attacks(sq) & enemy_knights).is_not_empty() {
        return true;
    }

    // King attacks
    let enemy_king = position.pieces_of(Piece::King, by_color);
    if (king_attacks(sq) & enemy_king).is_not_empty() {
        return true;
    }

    // Bishop/Queen attacks (diagonal)
    let enemy_bishops_queens =
        position.pieces_of(Piece::Bishop, by_color) | position.pieces_of(Piece::Queen, by_color);
    if (bishop_attacks(sq, occupied) & enemy_bishops_queens).is_not_empty() {
        return true;
    }

    // Rook/Queen attacks (orthogonal)
    let enemy_rooks_queens =
        position.pieces_of(Piece::Rook, by_color) | position.pieces_of(Piece::Queen, by_color);
    if (rook_attacks(sq, occupied) & enemy_rooks_queens).is_not_empty() {
        return true;
    }

    false
}

/// Returns true if the king of the given color is in check.
pub fn is_king_attacked(position: &Position, king_color: Color) -> bool {
    let king_bb = position.pieces_of(Piece::King, king_color);
    if let Some(king_idx) = king_bb.lsb() {
        let king_sq = unsafe { Square::from_index_unchecked(king_idx) };
        is_square_attacked(position, king_sq, king_color.opposite())
    } else {
        false // No king (shouldn't happen in valid position)
    }
}

/// Makes a move and returns the new position.
pub fn make_move(position: &Position, m: Move) -> Position {
    let mut new_pos = position.clone();
    let us = position.side_to_move;
    let them = us.opposite();
    let from = m.from();
    let to = m.to();

    // Get the piece being moved
    let (piece, _) = position.piece_at(from).expect("No piece at from square");

    // Remove piece from source
    new_pos.pieces[piece.index()].clear(from);
    new_pos.colors[us.index()].clear(from);

    // Handle captures
    let mut is_capture = false;
    if let Some((captured, _)) = position.piece_at(to) {
        new_pos.pieces[captured.index()].clear(to);
        new_pos.colors[them.index()].clear(to);
        is_capture = true;
    }

    // Handle en passant capture
    if m.flag() == MoveFlag::EnPassant {
        let captured_sq = match us {
            Color::White => unsafe { Square::from_index_unchecked(to.index() - 8) },
            Color::Black => unsafe { Square::from_index_unchecked(to.index() + 8) },
        };
        new_pos.pieces[Piece::Pawn.index()].clear(captured_sq);
        new_pos.colors[them.index()].clear(captured_sq);
        is_capture = true;
    }

    // Determine destination piece (handle promotion)
    let dest_piece = m.flag().promotion_piece().unwrap_or(piece);

    // Place piece at destination
    new_pos.pieces[dest_piece.index()].set(to);
    new_pos.colors[us.index()].set(to);

    // Handle castling - move the rook
    match m.flag() {
        MoveFlag::CastleKingside => {
            let (rook_from, rook_to) = match us {
                Color::White => (Square::H1, Square::F1),
                Color::Black => (Square::H8, Square::F8),
            };
            new_pos.pieces[Piece::Rook.index()].clear(rook_from);
            new_pos.colors[us.index()].clear(rook_from);
            new_pos.pieces[Piece::Rook.index()].set(rook_to);
            new_pos.colors[us.index()].set(rook_to);
        }
        MoveFlag::CastleQueenside => {
            let (rook_from, rook_to) = match us {
                Color::White => (Square::A1, Square::D1),
                Color::Black => (Square::A8, Square::D8),
            };
            new_pos.pieces[Piece::Rook.index()].clear(rook_from);
            new_pos.colors[us.index()].clear(rook_from);
            new_pos.pieces[Piece::Rook.index()].set(rook_to);
            new_pos.colors[us.index()].set(rook_to);
        }
        _ => {}
    }

    // Update castling rights
    // King move removes all castling rights for that color
    if piece == Piece::King {
        new_pos.castling.remove_color(us);
    }
    // Rook move or capture removes castling rights for that rook
    if piece == Piece::Rook {
        match (us, from) {
            (Color::White, sq) if sq == Square::H1 => new_pos.castling.remove_kingside(us),
            (Color::White, sq) if sq == Square::A1 => new_pos.castling.remove_queenside(us),
            (Color::Black, sq) if sq == Square::H8 => new_pos.castling.remove_kingside(us),
            (Color::Black, sq) if sq == Square::A8 => new_pos.castling.remove_queenside(us),
            _ => {}
        }
    }
    // Capture on rook starting square removes opponent's castling rights
    match to {
        sq if sq == Square::H1 => new_pos.castling.remove_kingside(Color::White),
        sq if sq == Square::A1 => new_pos.castling.remove_queenside(Color::White),
        sq if sq == Square::H8 => new_pos.castling.remove_kingside(Color::Black),
        sq if sq == Square::A8 => new_pos.castling.remove_queenside(Color::Black),
        _ => {}
    }

    // Update en passant square
    new_pos.en_passant = if m.flag() == MoveFlag::DoublePush {
        let ep_sq = match us {
            Color::White => unsafe { Square::from_index_unchecked(to.index() - 8) },
            Color::Black => unsafe { Square::from_index_unchecked(to.index() + 8) },
        };
        Some(ep_sq)
    } else {
        None
    };

    // Update clocks
    if piece == Piece::Pawn || is_capture {
        new_pos.halfmove_clock = 0;
    } else {
        new_pos.halfmove_clock += 1;
    }

    if us == Color::Black {
        new_pos.fullmove_number += 1;
    }

    // Switch side to move
    new_pos.side_to_move = them;

    new_pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::File;

    #[test]
    fn movelist_push_and_iterate() {
        let mut list = MoveList::new();
        assert!(list.is_empty());

        let e2 = Square::new(File::E, Rank::R2);
        let e4 = Square::new(File::E, Rank::R4);
        let d2 = Square::new(File::D, Rank::R2);
        let d4 = Square::new(File::D, Rank::R4);

        let m1 = Move::normal(e2, e4);
        let m2 = Move::normal(d2, d4);

        list.push(m1);
        list.push(m2);

        assert_eq!(list.len(), 2);
        assert_eq!(list[0], m1);
        assert_eq!(list[1], m2);
    }

    #[test]
    fn movelist_default() {
        let list = MoveList::default();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn movelist_clear() {
        let mut list = MoveList::new();
        let e2 = Square::new(File::E, Rank::R2);
        let e4 = Square::new(File::E, Rank::R4);
        list.push(Move::normal(e2, e4));
        assert_eq!(list.len(), 1);

        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn movelist_retain() {
        let mut list = MoveList::new();
        let e2 = Square::new(File::E, Rank::R2);
        let e3 = Square::new(File::E, Rank::R3);
        let e4 = Square::new(File::E, Rank::R4);

        list.push(Move::normal(e2, e3));
        list.push(Move::normal(e2, e4));
        list.push(Move::normal(e3, e4));

        // Keep only moves from e2
        list.retain(|m| m.from() == e2);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn generate_moves_startpos() {
        let position = Position::startpos();
        let moves = generate_moves(&position);
        assert_eq!(moves.len(), 20); // 16 pawn moves + 4 knight moves
    }

    #[test]
    fn make_move_pawn_push() {
        let position = Position::startpos();
        let e2 = Square::new(File::E, Rank::R2);
        let e4 = Square::new(File::E, Rank::R4);
        let m = Move::new(e2, e4, MoveFlag::DoublePush);

        let new_pos = make_move(&position, m);
        assert_eq!(new_pos.side_to_move, Color::Black);
        assert!(new_pos.piece_at(e4).is_some());
        assert!(new_pos.piece_at(e2).is_none());
        assert_eq!(new_pos.en_passant, Some(Square::new(File::E, Rank::R3)));
    }

    #[test]
    fn make_move_knight() {
        let position = Position::startpos();
        let g1 = Square::new(File::G, Rank::R1);
        let f3 = Square::new(File::F, Rank::R3);
        let m = Move::normal(g1, f3);

        let new_pos = make_move(&position, m);
        assert_eq!(new_pos.piece_at(f3), Some((Piece::Knight, Color::White)));
        assert!(new_pos.piece_at(g1).is_none());
    }

    #[test]
    fn is_square_attacked_startpos() {
        let position = Position::startpos();

        // e3 is attacked by pawns
        let e3 = Square::new(File::E, Rank::R3);
        assert!(is_square_attacked(&position, e3, Color::White));

        // e4 is not attacked at start
        let e4 = Square::new(File::E, Rank::R4);
        assert!(!is_square_attacked(&position, e4, Color::White));
    }

    #[test]
    fn is_king_attacked_startpos() {
        let position = Position::startpos();
        assert!(!is_king_attacked(&position, Color::White));
        assert!(!is_king_attacked(&position, Color::Black));
    }

    #[test]
    fn castling_kingside() {
        // Position where white can castle kingside
        let position =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let moves = generate_moves(&position);

        let has_castle = moves
            .as_slice()
            .iter()
            .any(|m| m.flag() == MoveFlag::CastleKingside);
        assert!(has_castle);
    }

    #[test]
    fn castling_queenside() {
        let position =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let moves = generate_moves(&position);

        let has_castle = moves
            .as_slice()
            .iter()
            .any(|m| m.flag() == MoveFlag::CastleQueenside);
        assert!(has_castle);
    }

    #[test]
    fn no_castling_through_check() {
        // King would pass through attacked square
        let position =
            Position::from_fen("r3k2r/pppp1ppp/8/4r3/8/8/PPPP1PPP/R3K2R w KQkq - 0 1").unwrap();
        let moves = generate_moves(&position);

        // Neither castle should be available (e-file rook attacks f1/d1)
        let has_kingside = moves
            .as_slice()
            .iter()
            .any(|m| m.flag() == MoveFlag::CastleKingside);
        // f1 is attacked, so no kingside
        assert!(!has_kingside);
    }

    #[test]
    fn en_passant() {
        // Position with en passant available
        let position =
            Position::from_fen("rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 1")
                .unwrap();
        let moves = generate_moves(&position);

        let has_ep = moves
            .as_slice()
            .iter()
            .any(|m| m.flag() == MoveFlag::EnPassant);
        assert!(has_ep);
    }

    #[test]
    fn promotion() {
        // Position with pawn about to promote
        let position = Position::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();
        let moves = generate_moves(&position);

        // Should have 4 promotion moves (Q, R, B, N)
        let promo_count = moves
            .as_slice()
            .iter()
            .filter(|m| m.flag().is_promotion())
            .count();
        assert_eq!(promo_count, 4);
    }
}
