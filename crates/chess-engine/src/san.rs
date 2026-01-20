//! Standard Algebraic Notation (SAN) parsing and generation.
//!
//! SAN is the standard way to record chess moves in human-readable form.
//! Examples: "e4", "Nf3", "Bxc6", "O-O", "e8=Q", "Nbd2", "R1e1"

use crate::movegen::{generate_moves, is_king_attacked, make_move};
use crate::Position;
use chess_core::{File, Move, MoveFlag, Piece, Rank, Square};
use std::fmt;

/// Error type for SAN parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SanError {
    /// The SAN string is empty.
    Empty,
    /// The SAN string has invalid format.
    InvalidFormat(String),
    /// No legal move matches the SAN.
    NoMatchingMove(String),
    /// Multiple legal moves match the SAN (ambiguous).
    AmbiguousMove(String),
}

impl fmt::Display for SanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SanError::Empty => write!(f, "empty SAN string"),
            SanError::InvalidFormat(s) => write!(f, "invalid SAN format: {}", s),
            SanError::NoMatchingMove(s) => write!(f, "no legal move matches: {}", s),
            SanError::AmbiguousMove(s) => write!(f, "ambiguous move: {}", s),
        }
    }
}

impl std::error::Error for SanError {}

/// Converts a move to SAN notation given the current position.
///
/// The position must be the state BEFORE the move is made.
/// The move must be legal in the given position.
pub fn move_to_san(position: &Position, m: Move) -> String {
    let mut san = String::new();

    // Handle castling
    if m.flag() == MoveFlag::CastleKingside {
        san.push_str("O-O");
        return add_check_suffix(position, m, san);
    }
    if m.flag() == MoveFlag::CastleQueenside {
        san.push_str("O-O-O");
        return add_check_suffix(position, m, san);
    }

    let from = m.from();
    let to = m.to();
    let (piece, _color) = position.piece_at(from).expect("move has no piece at from square");

    // Piece letter (except for pawns)
    if piece != Piece::Pawn {
        san.push(piece_to_san_char(piece));
    }

    // Disambiguation for non-pawn pieces
    if piece != Piece::Pawn {
        let disambig = get_disambiguation(position, m, piece);
        san.push_str(&disambig);
    }

    // Capture indicator
    let is_capture = position.piece_at(to).is_some() || m.flag() == MoveFlag::EnPassant;
    if is_capture {
        if piece == Piece::Pawn {
            // Pawn captures include the source file
            san.push(file_to_char(from.file()));
        }
        san.push('x');
    }

    // Destination square
    san.push(file_to_char(to.file()));
    san.push(rank_to_char(to.rank()));

    // Promotion
    if m.flag().is_promotion() {
        san.push('=');
        san.push(piece_to_san_char(
            m.flag().promotion_piece().expect("promotion flag without piece"),
        ));
    }

    add_check_suffix(position, m, san)
}

/// Parses a SAN string and returns the corresponding legal move.
pub fn san_to_move(position: &Position, san: &str) -> Result<Move, SanError> {
    let san = san.trim();
    if san.is_empty() {
        return Err(SanError::Empty);
    }

    // Remove check/checkmate suffix for parsing
    let san = san.trim_end_matches('#').trim_end_matches('+');

    // Handle castling
    if san == "O-O" || san == "0-0" {
        return find_castling_move(position, true);
    }
    if san == "O-O-O" || san == "0-0-0" {
        return find_castling_move(position, false);
    }

    // Parse the SAN components
    let parsed = parse_san_components(san)?;

    // Find matching legal move
    find_matching_move(position, &parsed)
}

/// Parsed components of a SAN string.
#[derive(Debug)]
struct ParsedSan {
    piece: Piece,
    from_file: Option<File>,
    from_rank: Option<Rank>,
    to_square: Square,
    promotion: Option<Piece>,
    is_capture: bool,
}

fn parse_san_components(san: &str) -> Result<ParsedSan, SanError> {
    let chars: Vec<char> = san.chars().collect();
    if chars.is_empty() {
        return Err(SanError::Empty);
    }

    let mut idx = 0;

    // Determine piece type
    let piece = if chars[0].is_uppercase() && chars[0] != 'O' {
        let p = san_char_to_piece(chars[0]).ok_or_else(|| {
            SanError::InvalidFormat(format!("invalid piece character: {}", chars[0]))
        })?;
        idx += 1;
        p
    } else {
        Piece::Pawn
    };

    // Parse remaining characters to find disambiguation, capture, and destination
    let remaining: String = chars[idx..].iter().collect();

    // Remove 'x' if present and note capture
    let (remaining, is_capture) = if remaining.contains('x') {
        (remaining.replace('x', ""), true)
    } else {
        (remaining, false)
    };

    // Check for promotion
    let (remaining, promotion) = if remaining.contains('=') {
        let parts: Vec<&str> = remaining.split('=').collect();
        if parts.len() != 2 || parts[1].len() != 1 {
            return Err(SanError::InvalidFormat(format!(
                "invalid promotion: {}",
                san
            )));
        }
        let promo_piece = san_char_to_piece(parts[1].chars().next().unwrap()).ok_or_else(|| {
            SanError::InvalidFormat(format!("invalid promotion piece: {}", parts[1]))
        })?;
        (parts[0].to_string(), Some(promo_piece))
    } else {
        (remaining, None)
    };

    let chars: Vec<char> = remaining.chars().collect();

    // The last two characters should be the destination square
    if chars.len() < 2 {
        return Err(SanError::InvalidFormat(format!("too short: {}", san)));
    }

    let to_file = char_to_file(chars[chars.len() - 2])
        .ok_or_else(|| SanError::InvalidFormat(format!("invalid file: {}", san)))?;
    let to_rank = char_to_rank(chars[chars.len() - 1])
        .ok_or_else(|| SanError::InvalidFormat(format!("invalid rank: {}", san)))?;
    let to_square = Square::new(to_file, to_rank);

    // Any characters before the destination are disambiguation
    let disambig: String = chars[..chars.len() - 2].iter().collect();
    let (from_file, from_rank) = parse_disambiguation(&disambig)?;

    Ok(ParsedSan {
        piece,
        from_file,
        from_rank,
        to_square,
        promotion,
        is_capture,
    })
}

fn parse_disambiguation(s: &str) -> Result<(Option<File>, Option<Rank>), SanError> {
    let chars: Vec<char> = s.chars().collect();

    match chars.len() {
        0 => Ok((None, None)),
        1 => {
            if let Some(f) = char_to_file(chars[0]) {
                Ok((Some(f), None))
            } else if let Some(r) = char_to_rank(chars[0]) {
                Ok((None, Some(r)))
            } else {
                Err(SanError::InvalidFormat(format!(
                    "invalid disambiguation: {}",
                    s
                )))
            }
        }
        2 => {
            let file = char_to_file(chars[0]).ok_or_else(|| {
                SanError::InvalidFormat(format!("invalid disambiguation file: {}", s))
            })?;
            let rank = char_to_rank(chars[1]).ok_or_else(|| {
                SanError::InvalidFormat(format!("invalid disambiguation rank: {}", s))
            })?;
            Ok((Some(file), Some(rank)))
        }
        _ => Err(SanError::InvalidFormat(format!(
            "disambiguation too long: {}",
            s
        ))),
    }
}

fn find_castling_move(position: &Position, kingside: bool) -> Result<Move, SanError> {
    let moves = generate_moves(position);
    let flag = if kingside {
        MoveFlag::CastleKingside
    } else {
        MoveFlag::CastleQueenside
    };

    for m in moves.as_slice() {
        if m.flag() == flag {
            return Ok(*m);
        }
    }

    let name = if kingside { "O-O" } else { "O-O-O" };
    Err(SanError::NoMatchingMove(name.to_string()))
}

fn find_matching_move(position: &Position, parsed: &ParsedSan) -> Result<Move, SanError> {
    let moves = generate_moves(position);
    let mut matching: Vec<Move> = Vec::new();

    for m in moves.as_slice() {
        // Check destination
        if m.to() != parsed.to_square {
            continue;
        }

        // Check piece type
        if let Some((piece, _)) = position.piece_at(m.from()) {
            if piece != parsed.piece {
                continue;
            }
        } else {
            continue;
        }

        // Check disambiguation
        if let Some(file) = parsed.from_file {
            if m.from().file() != file {
                continue;
            }
        }
        if let Some(rank) = parsed.from_rank {
            if m.from().rank() != rank {
                continue;
            }
        }

        // Check promotion
        if let Some(promo) = parsed.promotion {
            if !m.flag().is_promotion() {
                continue;
            }
            if m.flag().promotion_piece() != Some(promo) {
                continue;
            }
        } else if m.flag().is_promotion() {
            // If no promotion specified but move is promotion, skip
            // (unless it's a queen promotion which is often implicit)
            continue;
        }

        matching.push(*m);
    }

    match matching.len() {
        0 => Err(SanError::NoMatchingMove(format!("{:?}", parsed))),
        1 => Ok(matching[0]),
        _ => {
            // If multiple matches and it's a pawn move, try to be smarter
            // In case of multiple promotions, this shouldn't happen with proper SAN
            Err(SanError::AmbiguousMove(format!(
                "multiple moves match: {:?}",
                parsed
            )))
        }
    }
}

fn get_disambiguation(position: &Position, m: Move, piece: Piece) -> String {
    let moves = generate_moves(position);
    let to = m.to();
    let from = m.from();

    // Find all moves of the same piece type to the same square
    let mut same_dest: Vec<Move> = Vec::new();
    for other in moves.as_slice() {
        if other.to() != to {
            continue;
        }
        if let Some((p, _)) = position.piece_at(other.from()) {
            if p == piece {
                same_dest.push(*other);
            }
        }
    }

    // If only one move, no disambiguation needed
    if same_dest.len() <= 1 {
        return String::new();
    }

    // Check if file alone is sufficient
    let same_file: Vec<&Move> = same_dest
        .iter()
        .filter(|o| o.from().file() == from.file())
        .collect();
    if same_file.len() == 1 {
        return file_to_char(from.file()).to_string();
    }

    // Check if rank alone is sufficient
    let same_rank: Vec<&Move> = same_dest
        .iter()
        .filter(|o| o.from().rank() == from.rank())
        .collect();
    if same_rank.len() == 1 {
        return rank_to_char(from.rank()).to_string();
    }

    // Need both file and rank
    format!("{}{}", file_to_char(from.file()), rank_to_char(from.rank()))
}

fn add_check_suffix(position: &Position, m: Move, mut san: String) -> String {
    let new_pos = make_move(position, m);
    if is_king_attacked(&new_pos, new_pos.side_to_move) {
        // Check if it's checkmate
        let moves = generate_moves(&new_pos);
        if moves.is_empty() {
            san.push('#');
        } else {
            san.push('+');
        }
    }
    san
}

fn piece_to_san_char(piece: Piece) -> char {
    match piece {
        Piece::Pawn => 'P', // Not typically used in SAN
        Piece::Knight => 'N',
        Piece::Bishop => 'B',
        Piece::Rook => 'R',
        Piece::Queen => 'Q',
        Piece::King => 'K',
    }
}

fn san_char_to_piece(c: char) -> Option<Piece> {
    match c {
        'N' => Some(Piece::Knight),
        'B' => Some(Piece::Bishop),
        'R' => Some(Piece::Rook),
        'Q' => Some(Piece::Queen),
        'K' => Some(Piece::King),
        'P' => Some(Piece::Pawn),
        _ => None,
    }
}

fn file_to_char(file: File) -> char {
    (b'a' + file.index() as u8) as char
}

fn rank_to_char(rank: Rank) -> char {
    (b'1' + rank.index() as u8) as char
}

fn char_to_file(c: char) -> Option<File> {
    match c {
        'a' => Some(File::A),
        'b' => Some(File::B),
        'c' => Some(File::C),
        'd' => Some(File::D),
        'e' => Some(File::E),
        'f' => Some(File::F),
        'g' => Some(File::G),
        'h' => Some(File::H),
        _ => None,
    }
}

fn char_to_rank(c: char) -> Option<Rank> {
    match c {
        '1' => Some(Rank::R1),
        '2' => Some(Rank::R2),
        '3' => Some(Rank::R3),
        '4' => Some(Rank::R4),
        '5' => Some(Rank::R5),
        '6' => Some(Rank::R6),
        '7' => Some(Rank::R7),
        '8' => Some(Rank::R8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn san_pawn_push() {
        let pos = Position::startpos();
        let e2 = Square::new(File::E, Rank::R2);
        let e4 = Square::new(File::E, Rank::R4);
        let m = Move::new(e2, e4, MoveFlag::DoublePush);
        assert_eq!(move_to_san(&pos, m), "e4");
    }

    #[test]
    fn san_knight_move() {
        let pos = Position::startpos();
        let g1 = Square::new(File::G, Rank::R1);
        let f3 = Square::new(File::F, Rank::R3);
        let m = Move::normal(g1, f3);
        assert_eq!(move_to_san(&pos, m), "Nf3");
    }

    #[test]
    fn san_pawn_capture() {
        // Position where e4 pawn can capture d5 pawn
        let pos = Position::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2")
            .unwrap();
        let e4 = Square::new(File::E, Rank::R4);
        let d5 = Square::new(File::D, Rank::R5);
        let m = Move::normal(e4, d5);
        assert_eq!(move_to_san(&pos, m), "exd5");
    }

    #[test]
    fn san_castling_kingside() {
        let pos =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let e1 = Square::new(File::E, Rank::R1);
        let g1 = Square::new(File::G, Rank::R1);
        let m = Move::new(e1, g1, MoveFlag::CastleKingside);
        assert_eq!(move_to_san(&pos, m), "O-O");
    }

    #[test]
    fn san_castling_queenside() {
        let pos =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let e1 = Square::new(File::E, Rank::R1);
        let c1 = Square::new(File::C, Rank::R1);
        let m = Move::new(e1, c1, MoveFlag::CastleQueenside);
        assert_eq!(move_to_san(&pos, m), "O-O-O");
    }

    #[test]
    fn san_promotion() {
        // Position where promotion doesn't give check
        let pos = Position::from_fen("8/P7/8/8/8/8/8/4K1k1 w - - 0 1").unwrap();
        let a7 = Square::new(File::A, Rank::R7);
        let a8 = Square::new(File::A, Rank::R8);
        let m = Move::new(a7, a8, MoveFlag::PromoteQueen);
        assert_eq!(move_to_san(&pos, m), "a8=Q");
    }

    #[test]
    fn san_knight_disambiguation_file() {
        // Two knights on same rank can move to d2, need file disambiguation
        // Knights on b1 and f1, both can move to d2
        let pos = Position::from_fen("8/8/8/8/8/8/8/1N1K1N1k w - - 0 1").unwrap();
        let b1 = Square::new(File::B, Rank::R1);
        let d2 = Square::new(File::D, Rank::R2);
        let m = Move::normal(b1, d2);
        assert_eq!(move_to_san(&pos, m), "Nbd2");
    }

    #[test]
    fn san_check() {
        // Queen gives check
        let pos = Position::from_fen("8/8/8/8/8/8/8/4K1Qk w - - 0 1").unwrap();
        let g1 = Square::new(File::G, Rank::R1);
        let h2 = Square::new(File::H, Rank::R2);
        let m = Move::normal(g1, h2);
        assert_eq!(move_to_san(&pos, m), "Qh2+");
    }

    #[test]
    fn san_checkmate() {
        // Back rank mate
        let pos = Position::from_fen("6k1/5ppp/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();
        let a1 = Square::new(File::A, Rank::R1);
        let a8 = Square::new(File::A, Rank::R8);
        let m = Move::normal(a1, a8);
        assert_eq!(move_to_san(&pos, m), "Ra8#");
    }

    #[test]
    fn parse_san_pawn_push() {
        let pos = Position::startpos();
        let m = san_to_move(&pos, "e4").unwrap();
        assert_eq!(m.to(), Square::new(File::E, Rank::R4));
    }

    #[test]
    fn parse_san_knight_move() {
        let pos = Position::startpos();
        let m = san_to_move(&pos, "Nf3").unwrap();
        assert_eq!(m.from(), Square::new(File::G, Rank::R1));
        assert_eq!(m.to(), Square::new(File::F, Rank::R3));
    }

    #[test]
    fn parse_san_castling() {
        let pos =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let m = san_to_move(&pos, "O-O").unwrap();
        assert_eq!(m.flag(), MoveFlag::CastleKingside);

        let m = san_to_move(&pos, "O-O-O").unwrap();
        assert_eq!(m.flag(), MoveFlag::CastleQueenside);
    }

    #[test]
    fn parse_san_with_check_suffix() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/4K1Qk w - - 0 1").unwrap();
        let m = san_to_move(&pos, "Qh2+").unwrap();
        assert_eq!(m.to(), Square::new(File::H, Rank::R2));
    }

    #[test]
    fn parse_san_invalid() {
        let pos = Position::startpos();
        assert!(san_to_move(&pos, "").is_err());
        assert!(san_to_move(&pos, "Ke4").is_err()); // King can't move to e4
        assert!(san_to_move(&pos, "xyz").is_err());
    }

    #[test]
    fn san_roundtrip() {
        let pos = Position::startpos();
        let moves = generate_moves(&pos);
        for m in moves.as_slice() {
            let san = move_to_san(&pos, *m);
            let parsed = san_to_move(&pos, &san).unwrap();
            assert_eq!(*m, parsed, "roundtrip failed for {}", san);
        }
    }
}
