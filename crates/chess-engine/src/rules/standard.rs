//! Standard chess rules implementation.

use super::{GameResult, RuleSet};
use crate::movegen::{generate_moves, is_king_attacked, make_move};
use crate::{MoveList, Position};
use chess_core::Move;

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
        // Check 50-move rule
        if position.halfmove_clock >= 100 {
            return Some(GameResult::Draw);
        }

        let moves = self.generate_moves(position);
        if moves.is_empty() {
            if self.is_check(position) {
                // Checkmate - the side to move loses
                return Some(match position.side_to_move {
                    chess_core::Color::White => GameResult::BlackWins,
                    chess_core::Color::Black => GameResult::WhiteWins,
                });
            } else {
                // Stalemate
                return Some(GameResult::Draw);
            }
        }

        None
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
        assert_eq!(StandardChess.game_result(&pos), Some(GameResult::Draw));
    }

    #[test]
    fn fifty_move_rule() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/4K2k w - - 100 1").unwrap();
        assert_eq!(StandardChess.game_result(&pos), Some(GameResult::Draw));
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
