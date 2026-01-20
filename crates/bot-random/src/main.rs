//! Random move bot - plays a random legal move.
//!
//! This is the simplest possible UCI bot, useful as a template
//! for more sophisticated bots.

use chess_engine::{Position, StandardChess};
use chess_engine::rules::RuleSet;
use rand::seq::SliceRandom;
use uci::{stdio_engine, GuiCommand};

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
                engine.send_id("RandomBot", "Chess Devtools").unwrap();
                engine.send_uciok().unwrap();
            }

            GuiCommand::Extensions => {
                // No extensions supported by this simple bot
                engine.send_extensionsok().unwrap();
            }

            GuiCommand::IsReady => {
                engine.send_readyok().unwrap();
            }

            GuiCommand::Position { fen, moves } => {
                // Set up position from FEN or starting position
                position = match fen {
                    Some(f) => Position::from_fen(&f).unwrap_or_else(|_| {
                        StandardChess.initial_position()
                    }),
                    None => StandardChess.initial_position(),
                };

                // Apply moves
                for mv_str in moves {
                    if let Some(mv) = chess_core::Move::from_uci(&mv_str) {
                        // Find matching legal move with correct flags
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

            GuiCommand::Go(_opts) => {
                // Pick a random legal move
                let legal_moves = StandardChess.generate_moves(&position);
                let moves = legal_moves.as_slice();

                if moves.is_empty() {
                    // No legal moves - game over
                    engine.send_bestmove("0000").unwrap();
                } else {
                    let mv = moves.choose(&mut rand::thread_rng()).unwrap();
                    engine.send_bestmove(&mv.to_uci()).unwrap();
                }
            }

            GuiCommand::Stop => {
                // Nothing to stop for instant moves
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
