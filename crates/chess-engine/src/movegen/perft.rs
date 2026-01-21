//! Perft (performance test) for move generator validation.
//!
//! Perft counts the number of leaf nodes at a given depth, which can be
//! compared against known-correct values to validate the move generator.

use super::{generate_moves, make_move};
use crate::Position;

/// Counts the number of leaf nodes at the given depth.
///
/// This is the standard perft function used to validate move generators.
pub fn perft(position: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_moves(position);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;
    for m in &moves {
        let new_pos = make_move(position, *m);
        nodes += perft(&new_pos, depth - 1);
    }
    nodes
}

/// Perft with divide - shows node count for each move at depth-1.
/// Useful for debugging to identify which moves have incorrect counts.
pub fn perft_divide(position: &Position, depth: u32) -> Vec<(String, u64)> {
    let moves = generate_moves(position);
    let mut results = Vec::with_capacity(moves.len());

    for m in &moves {
        let new_pos = make_move(position, *m);
        let nodes = if depth > 1 {
            perft(&new_pos, depth - 1)
        } else {
            1
        };
        results.push((m.to_uci(), nodes));
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    // Starting position perft values (well-known and verified)
    #[test]
    fn perft_startpos_depth_1() {
        let position = Position::startpos();
        assert_eq!(perft(&position, 1), 20);
    }

    #[test]
    fn perft_startpos_depth_2() {
        let position = Position::startpos();
        assert_eq!(perft(&position, 2), 400);
    }

    #[test]
    fn perft_startpos_depth_3() {
        let position = Position::startpos();
        assert_eq!(perft(&position, 3), 8902);
    }

    #[test]
    fn perft_startpos_depth_4() {
        let position = Position::startpos();
        assert_eq!(perft(&position, 4), 197281);
    }

    // Depth 5 is slower, only run in release mode
    #[test]
    #[ignore]
    fn perft_startpos_depth_5() {
        let position = Position::startpos();
        assert_eq!(perft(&position, 5), 4865609);
    }

    // Kiwipete - a position with lots of special moves
    // r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -
    #[test]
    fn perft_kiwipete_depth_1() {
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        assert_eq!(perft(&position, 1), 48);
    }

    #[test]
    fn perft_kiwipete_depth_2() {
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        assert_eq!(perft(&position, 2), 2039);
    }

    #[test]
    fn perft_kiwipete_depth_3() {
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        assert_eq!(perft(&position, 3), 97862);
    }

    // Position 3: Check evasion, en passant, promotion
    // 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -
    #[test]
    fn perft_position3_depth_1() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        assert_eq!(perft(&position, 1), 14);
    }

    #[test]
    fn perft_position3_depth_2() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        assert_eq!(perft(&position, 2), 191);
    }

    #[test]
    fn perft_position3_depth_3() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        assert_eq!(perft(&position, 3), 2812);
    }

    // Position 4: Lots of promotions and captures
    // r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq -
    #[test]
    fn perft_position4_depth_1() {
        let position =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();
        assert_eq!(perft(&position, 1), 6);
    }

    #[test]
    fn perft_position4_depth_2() {
        let position =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();
        assert_eq!(perft(&position, 2), 264);
    }

    #[test]
    fn perft_position4_depth_3() {
        let position =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();
        assert_eq!(perft(&position, 3), 9467);
    }

    // Position 5: Complex position
    // rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ -
    #[test]
    fn perft_position5_depth_1() {
        let position =
            Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 0 1")
                .unwrap();
        assert_eq!(perft(&position, 1), 44);
    }

    #[test]
    fn perft_position5_depth_2() {
        let position =
            Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 0 1")
                .unwrap();
        assert_eq!(perft(&position, 2), 1486);
    }

    #[test]
    fn perft_position5_depth_3() {
        let position =
            Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 0 1")
                .unwrap();
        assert_eq!(perft(&position, 3), 62379);
    }

    #[test]
    fn perft_divide_works() {
        let position = Position::startpos();
        let results = perft_divide(&position, 1);
        assert_eq!(results.len(), 20);
        // Total should equal perft(1)
        let total: u64 = results.iter().map(|(_, n)| n).sum();
        assert_eq!(total, 20);
    }
}
