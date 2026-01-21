//! Minimax bot with alpha-beta pruning.
//!
//! A basic chess bot that uses minimax search with alpha-beta pruning
//! and a simple material + position evaluation function.

use chess_core::{Color, Move, Piece};
use chess_engine::rules::RuleSet;
use chess_engine::{is_king_attacked, Position, StandardChess};
use std::io::{BufReader, Stdin, Stdout};
use std::time::{Duration, Instant};
use uci::{stdio_engine, GuiCommand, InfoBuilder, UciEngine};

type StdioEngine = UciEngine<BufReader<Stdin>, Stdout>;

/// Piece values in centipawns
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

/// Piece-square tables for positional evaluation (from white's perspective).
/// Values are in centipawns, added to piece base value.
const PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_PST: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_PST: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];

const QUEEN_PST: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const KING_MIDDLEGAME_PST: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

/// Search state
struct Searcher {
    nodes: u64,
    start_time: Instant,
    max_time: Duration,
    stopped: bool,
}

impl Searcher {
    fn new(max_time: Duration) -> Self {
        Searcher {
            nodes: 0,
            start_time: Instant::now(),
            max_time,
            stopped: false,
        }
    }

    fn check_time(&mut self) {
        if self.nodes.is_multiple_of(4096) && self.start_time.elapsed() > self.max_time {
            self.stopped = true;
        }
    }
}

/// Evaluate the position from the side to move's perspective
fn evaluate(position: &Position) -> i32 {
    let mut score = 0i32;

    // Material and positional evaluation
    for color in [Color::White, Color::Black] {
        let sign = if color == Color::White { 1 } else { -1 };

        // Pawns
        for sq in position.pieces_of(Piece::Pawn, color) {
            let idx = if color == Color::White {
                sq.index() as usize
            } else {
                63 - sq.index() as usize
            };
            score += sign * (PAWN_VALUE + PAWN_PST[idx]);
        }

        // Knights
        for sq in position.pieces_of(Piece::Knight, color) {
            let idx = if color == Color::White {
                sq.index() as usize
            } else {
                63 - sq.index() as usize
            };
            score += sign * (KNIGHT_VALUE + KNIGHT_PST[idx]);
        }

        // Bishops
        for sq in position.pieces_of(Piece::Bishop, color) {
            let idx = if color == Color::White {
                sq.index() as usize
            } else {
                63 - sq.index() as usize
            };
            score += sign * (BISHOP_VALUE + BISHOP_PST[idx]);
        }

        // Rooks
        for sq in position.pieces_of(Piece::Rook, color) {
            let idx = if color == Color::White {
                sq.index() as usize
            } else {
                63 - sq.index() as usize
            };
            score += sign * (ROOK_VALUE + ROOK_PST[idx]);
        }

        // Queens
        for sq in position.pieces_of(Piece::Queen, color) {
            let idx = if color == Color::White {
                sq.index() as usize
            } else {
                63 - sq.index() as usize
            };
            score += sign * (QUEEN_VALUE + QUEEN_PST[idx]);
        }

        // King (middlegame table)
        for sq in position.pieces_of(Piece::King, color) {
            let idx = if color == Color::White {
                sq.index() as usize
            } else {
                63 - sq.index() as usize
            };
            score += sign * KING_MIDDLEGAME_PST[idx];
        }
    }

    // Return score from side to move's perspective
    if position.side_to_move == Color::White {
        score
    } else {
        -score
    }
}

/// Alpha-beta search
fn alpha_beta(
    searcher: &mut Searcher,
    position: &Position,
    depth: u8,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    searcher.nodes += 1;
    searcher.check_time();

    if searcher.stopped {
        return 0;
    }

    // Terminal node
    if depth == 0 {
        return evaluate(position);
    }

    let moves = StandardChess.generate_moves(position);

    // Check for checkmate or stalemate
    if moves.is_empty() {
        if is_king_attacked(position, position.side_to_move) {
            // Checkmate - return large negative score (we lost)
            return -100_000 + (100 - depth as i32); // Prefer faster mates
        } else {
            // Stalemate
            return 0;
        }
    }

    for mv in moves.as_slice() {
        let new_pos = StandardChess.make_move(position, *mv);
        let score = -alpha_beta(searcher, &new_pos, depth - 1, -beta, -alpha);

        if searcher.stopped {
            return 0;
        }

        if score >= beta {
            return beta; // Beta cutoff
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

/// Find the best move using iterative deepening
fn search(position: &Position, max_time: Duration, engine: &mut StdioEngine) -> Option<Move> {
    let mut searcher = Searcher::new(max_time);
    let mut best_move: Option<Move> = None;

    let moves = StandardChess.generate_moves(position);
    if moves.is_empty() {
        return None;
    }

    // Iterative deepening
    for depth in 1..=64u8 {
        let iter_start = Instant::now();
        let mut current_best: Option<Move> = None;
        let mut current_score = i32::MIN;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;

        for mv in moves.as_slice() {
            let new_pos = StandardChess.make_move(position, *mv);
            let score = -alpha_beta(&mut searcher, &new_pos, depth - 1, -beta, -alpha);

            if searcher.stopped {
                break;
            }

            if score > current_score {
                current_score = score;
                current_best = Some(*mv);
                if score > alpha {
                    alpha = score;
                }
            }
        }

        if searcher.stopped {
            break;
        }

        // Update best move if we completed this depth
        if let Some(mv) = current_best {
            best_move = Some(mv);
            let best_score = current_score;

            // Send search info
            let info = InfoBuilder::new()
                .depth(depth as u32)
                .score_cp(best_score)
                .nodes(searcher.nodes)
                .time(searcher.start_time.elapsed().as_millis() as u64)
                .pv(vec![mv.to_uci()])
                .build();

            engine.send_info(info).ok();
        }

        // Check if we should stop
        let elapsed = iter_start.elapsed();
        if elapsed.as_millis() > 0 && searcher.start_time.elapsed() > max_time / 2 {
            break; // Unlikely to complete next depth in time
        }
    }

    best_move
}

fn main() {
    let mut engine = stdio_engine();
    let mut position = StandardChess.initial_position();

    loop {
        let cmd = match engine.read_command() {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Error reading command: {}", e);
                continue;
            }
        };

        match cmd {
            GuiCommand::Uci => {
                engine.send_id("MinimaxBot", "Chess Devtools").unwrap();
                engine.send_uciok().unwrap();
            }

            GuiCommand::Extensions => {
                // No extensions supported yet
                engine.send_extensionsok().unwrap();
            }

            GuiCommand::IsReady => {
                engine.send_readyok().unwrap();
            }

            GuiCommand::Position { fen, moves } => {
                // Set up position from FEN or starting position
                position = match fen {
                    Some(f) => {
                        Position::from_fen(&f).unwrap_or_else(|_| StandardChess.initial_position())
                    }
                    None => StandardChess.initial_position(),
                };

                // Apply moves
                for mv_str in moves {
                    if let Some(mv) = Move::from_uci(&mv_str) {
                        let legal_moves = StandardChess.generate_moves(&position);
                        if let Some(&legal_mv) = legal_moves.as_slice().iter().find(|m| {
                            m.from() == mv.from()
                                && m.to() == mv.to()
                                && m.flag().promotion_piece() == mv.flag().promotion_piece()
                        }) {
                            position = StandardChess.make_move(&position, legal_mv);
                        }
                    }
                }
            }

            GuiCommand::Go(opts) => {
                // Determine search time
                let max_time = if let Some(mt) = opts.movetime {
                    Duration::from_millis(mt)
                } else {
                    // Use time controls if available
                    let our_time = match position.side_to_move {
                        Color::White => opts.wtime,
                        Color::Black => opts.btime,
                    };

                    if let Some(time_ms) = our_time {
                        // Use about 2.5% of remaining time
                        Duration::from_millis(time_ms / 40)
                    } else {
                        // Default to 1 second
                        Duration::from_secs(1)
                    }
                };

                // Search for best move
                if let Some(mv) = search(&position, max_time, &mut engine) {
                    engine.send_bestmove(&mv.to_uci()).unwrap();
                } else {
                    // No legal moves - game over
                    engine.send_bestmove("0000").unwrap();
                }
            }

            GuiCommand::Stop => {
                // Nothing to stop (we don't support pondering)
            }

            GuiCommand::Quit => {
                break;
            }

            GuiCommand::Unknown(_) => {
                // Ignore unknown commands
            }
        }
    }
}
