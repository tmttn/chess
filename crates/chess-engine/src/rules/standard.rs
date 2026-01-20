//! Standard chess rules implementation.

use super::{DrawReason, GameResult, RuleSet};
use crate::movegen::{generate_moves, is_king_attacked, make_move};
use crate::{Bitboard, MoveList, Position};
use chess_core::{Color, Move, Piece};

/// Standard chess rules (FIDE).
///
/// This is the default rule set implementing standard chess rules:
/// - Standard piece movement
/// - Castling (kingside and queenside)
/// - En passant
/// - Pawn promotion
/// - Check, checkmate, and stalemate detection
/// - 50-move rule
#[derive(Debug, Clone, Copy, Default)]
pub struct StandardChess;

impl RuleSet for StandardChess {
    fn initial_position(&self) -> Position {
        Position::startpos()
    }

    fn generate_moves(&self, position: &Position) -> MoveList {
        generate_moves(position)
    }

    fn is_legal(&self, position: &Position, m: Move) -> bool {
        self.generate_moves(position).as_slice().contains(&m)
    }

    fn make_move(&self, position: &Position, m: Move) -> Position {
        make_move(position, m)
    }

    fn is_check(&self, position: &Position) -> bool {
        is_king_attacked(position, position.side_to_move)
    }

    fn game_result(&self, position: &Position) -> Option<GameResult> {
        // Check 75-move rule (automatic draw)
        if position.halfmove_clock >= 150 {
            return Some(GameResult::Draw(DrawReason::SeventyFiveMoveRule));
        }

        // Check insufficient material
        if self.is_insufficient_material(position) {
            return Some(GameResult::Draw(DrawReason::InsufficientMaterial));
        }

        let moves = self.generate_moves(position);
        if moves.is_empty() {
            if self.is_check(position) {
                // Checkmate - the side to move loses
                return Some(match position.side_to_move {
                    Color::White => GameResult::BlackWins,
                    Color::Black => GameResult::WhiteWins,
                });
            } else {
                // Stalemate
                return Some(GameResult::Draw(DrawReason::Stalemate));
            }
        }

        None
    }

    fn is_insufficient_material(&self, position: &Position) -> bool {
        // Count all pieces for each side
        let white_pawns = position.pieces_of(Piece::Pawn, Color::White).count();
        let black_pawns = position.pieces_of(Piece::Pawn, Color::Black).count();
        let white_knights = position.pieces_of(Piece::Knight, Color::White).count();
        let black_knights = position.pieces_of(Piece::Knight, Color::Black).count();
        let white_bishops = position.pieces_of(Piece::Bishop, Color::White);
        let black_bishops = position.pieces_of(Piece::Bishop, Color::Black);
        let white_rooks = position.pieces_of(Piece::Rook, Color::White).count();
        let black_rooks = position.pieces_of(Piece::Rook, Color::Black).count();
        let white_queens = position.pieces_of(Piece::Queen, Color::White).count();
        let black_queens = position.pieces_of(Piece::Queen, Color::Black).count();

        // If any pawns, rooks, or queens exist, not insufficient
        if white_pawns > 0
            || black_pawns > 0
            || white_rooks > 0
            || black_rooks > 0
            || white_queens > 0
            || black_queens > 0
        {
            return false;
        }

        let white_bishop_count = white_bishops.count();
        let black_bishop_count = black_bishops.count();

        // K vs K
        if white_knights == 0
            && black_knights == 0
            && white_bishop_count == 0
            && black_bishop_count == 0
        {
            return true;
        }

        // K+N vs K or K vs K+N
        if white_bishop_count == 0 && black_bishop_count == 0 {
            if (white_knights == 1 && black_knights == 0)
                || (white_knights == 0 && black_knights == 1)
            {
                return true;
            }
        }

        // K+B vs K or K vs K+B
        if white_knights == 0 && black_knights == 0 {
            if (white_bishop_count == 1 && black_bishop_count == 0)
                || (white_bishop_count == 0 && black_bishop_count == 1)
            {
                return true;
            }

            // K+B vs K+B with bishops on same color
            if white_bishop_count == 1 && black_bishop_count == 1 {
                let white_on_light = (white_bishops & Bitboard::LIGHT_SQUARES).is_not_empty();
                let black_on_light = (black_bishops & Bitboard::LIGHT_SQUARES).is_not_empty();
                if white_on_light == black_on_light {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_position() {
        let pos = StandardChess.initial_position();
        assert_eq!(pos.to_fen(), chess_core::FenParser::STARTPOS);
    }

    #[test]
    fn starting_moves() {
        let pos = StandardChess.initial_position();
        let moves = StandardChess.generate_moves(&pos);
        assert_eq!(moves.len(), 20); // 16 pawn moves + 4 knight moves
    }

    #[test]
    fn not_in_check_startpos() {
        let pos = StandardChess.initial_position();
        assert!(!StandardChess.is_check(&pos));
    }

    #[test]
    fn no_game_result_startpos() {
        let pos = StandardChess.initial_position();
        assert!(StandardChess.game_result(&pos).is_none());
    }

    #[test]
    fn checkmate_fools_mate() {
        // Fool's mate position - black has checkmated white
        let pos = Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        assert!(StandardChess.is_check(&pos));
        assert_eq!(
            StandardChess.game_result(&pos),
            Some(GameResult::BlackWins)
        );
    }

    #[test]
    fn stalemate() {
        // Classic stalemate position - black king trapped in corner
        // Black king on h8, white queen on f7, white king on g6
        // Queen and king control all escape squares but don't give check
        let pos = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(!StandardChess.is_check(&pos));
        let moves = StandardChess.generate_moves(&pos);
        assert!(moves.is_empty());
        assert_eq!(
            StandardChess.game_result(&pos),
            Some(GameResult::Draw(DrawReason::Stalemate))
        );
    }

    #[test]
    fn seventy_five_move_rule() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/4K2k w - - 150 1").unwrap();
        assert_eq!(
            StandardChess.game_result(&pos),
            Some(GameResult::Draw(DrawReason::SeventyFiveMoveRule))
        );
    }

    #[test]
    fn insufficient_material_k_vs_k() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/4K2k w - - 0 1").unwrap();
        assert!(StandardChess.is_insufficient_material(&pos));
        assert_eq!(
            StandardChess.game_result(&pos),
            Some(GameResult::Draw(DrawReason::InsufficientMaterial))
        );
    }

    #[test]
    fn insufficient_material_k_n_vs_k() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/4KN1k w - - 0 1").unwrap();
        assert!(StandardChess.is_insufficient_material(&pos));
    }

    #[test]
    fn insufficient_material_k_b_vs_k() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/4KB1k w - - 0 1").unwrap();
        assert!(StandardChess.is_insufficient_material(&pos));
    }

    #[test]
    fn insufficient_material_k_b_vs_k_b_same_color() {
        // Both bishops on light squares (f1=light, a2=light)
        let pos = Position::from_fen("8/8/8/8/8/8/b7/4KB1k w - - 0 1").unwrap();
        assert!(StandardChess.is_insufficient_material(&pos));

        // Both bishops on dark squares (c1=dark, b2=dark)
        let pos = Position::from_fen("8/8/8/8/8/8/1b6/2B1K2k w - - 0 1").unwrap();
        assert!(StandardChess.is_insufficient_material(&pos));
    }

    #[test]
    fn sufficient_material_k_b_vs_k_b_opposite_color() {
        // Bishops on opposite colors (f1=light, b2=dark) - can checkmate
        let pos = Position::from_fen("8/8/8/8/8/8/1b6/4KB1k w - - 0 1").unwrap();
        assert!(!StandardChess.is_insufficient_material(&pos));
    }

    #[test]
    fn sufficient_material_with_pawn() {
        let pos = Position::from_fen("8/8/8/8/8/8/4P3/4K2k w - - 0 1").unwrap();
        assert!(!StandardChess.is_insufficient_material(&pos));
    }

    #[test]
    fn sufficient_material_with_rook() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/4KR1k w - - 0 1").unwrap();
        assert!(!StandardChess.is_insufficient_material(&pos));
    }

    #[test]
    fn sufficient_material_k_n_n_vs_k() {
        // Two knights can technically checkmate (with opponent cooperation)
        let pos = Position::from_fen("8/8/8/8/8/8/8/3NKN1k w - - 0 1").unwrap();
        assert!(!StandardChess.is_insufficient_material(&pos));
    }

    #[test]
    fn is_legal_move() {
        let pos = StandardChess.initial_position();
        let e2 = chess_core::Square::new(chess_core::File::E, chess_core::Rank::R2);
        let e4 = chess_core::Square::new(chess_core::File::E, chess_core::Rank::R4);
        let legal_move = Move::new(e2, e4, chess_core::MoveFlag::DoublePush);
        assert!(StandardChess.is_legal(&pos, legal_move));

        // Illegal move - e2 to e5
        let e5 = chess_core::Square::new(chess_core::File::E, chess_core::Rank::R5);
        let illegal_move = Move::normal(e2, e5);
        assert!(!StandardChess.is_legal(&pos, illegal_move));
    }

    #[test]
    fn make_move_updates_position() {
        let pos = StandardChess.initial_position();
        let e2 = chess_core::Square::new(chess_core::File::E, chess_core::Rank::R2);
        let e4 = chess_core::Square::new(chess_core::File::E, chess_core::Rank::R4);
        let m = Move::new(e2, e4, chess_core::MoveFlag::DoublePush);

        let new_pos = StandardChess.make_move(&pos, m);
        assert_eq!(new_pos.side_to_move, chess_core::Color::Black);
        assert!(new_pos.piece_at(e4).is_some());
        assert!(new_pos.piece_at(e2).is_none());
    }
}
