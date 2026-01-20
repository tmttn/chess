//! Chess position representation.

use chess_core::{Color, FenError, FenParser, Piece, Square};

use crate::Bitboard;

/// Castling rights flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const NONE: CastlingRights = CastlingRights(0);
    pub const WHITE_KINGSIDE: u8 = 0b0001;
    pub const WHITE_QUEENSIDE: u8 = 0b0010;
    pub const BLACK_KINGSIDE: u8 = 0b0100;
    pub const BLACK_QUEENSIDE: u8 = 0b1000;
    pub const ALL: CastlingRights = CastlingRights(0b1111);

    /// Creates new castling rights from flags.
    #[inline]
    pub const fn new(flags: u8) -> Self {
        CastlingRights(flags & 0b1111)
    }

    /// Returns true if the given side can castle kingside.
    #[inline]
    pub const fn can_castle_kingside(self, color: Color) -> bool {
        let flag = match color {
            Color::White => Self::WHITE_KINGSIDE,
            Color::Black => Self::BLACK_KINGSIDE,
        };
        (self.0 & flag) != 0
    }

    /// Returns true if the given side can castle queenside.
    #[inline]
    pub const fn can_castle_queenside(self, color: Color) -> bool {
        let flag = match color {
            Color::White => Self::WHITE_QUEENSIDE,
            Color::Black => Self::BLACK_QUEENSIDE,
        };
        (self.0 & flag) != 0
    }

    /// Removes castling rights for a color.
    #[inline]
    pub fn remove_color(&mut self, color: Color) {
        let mask = match color {
            Color::White => !(Self::WHITE_KINGSIDE | Self::WHITE_QUEENSIDE),
            Color::Black => !(Self::BLACK_KINGSIDE | Self::BLACK_QUEENSIDE),
        };
        self.0 &= mask;
    }

    /// Removes kingside castling for a color.
    #[inline]
    pub fn remove_kingside(&mut self, color: Color) {
        let mask = match color {
            Color::White => !Self::WHITE_KINGSIDE,
            Color::Black => !Self::BLACK_KINGSIDE,
        };
        self.0 &= mask;
    }

    /// Removes queenside castling for a color.
    #[inline]
    pub fn remove_queenside(&mut self, color: Color) {
        let mask = match color {
            Color::White => !Self::WHITE_QUEENSIDE,
            Color::Black => !Self::BLACK_QUEENSIDE,
        };
        self.0 &= mask;
    }

    /// Returns the raw flags.
    #[inline]
    pub const fn raw(self) -> u8 {
        self.0
    }
}

/// Complete chess position state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    /// Bitboards for each piece type, indexed by Piece enum.
    pub pieces: [Bitboard; 6],

    /// Bitboards for each color's pieces.
    pub colors: [Bitboard; 2],

    /// The side to move.
    pub side_to_move: Color,

    /// Castling rights.
    pub castling: CastlingRights,

    /// En passant target square (if any).
    pub en_passant: Option<Square>,

    /// Halfmove clock for 50-move rule.
    pub halfmove_clock: u32,

    /// Fullmove number (starts at 1, increments after Black's move).
    pub fullmove_number: u32,
}

impl Position {
    /// Creates an empty position.
    pub fn empty() -> Self {
        Position {
            pieces: [Bitboard::EMPTY; 6],
            colors: [Bitboard::EMPTY; 2],
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Creates the standard starting position.
    pub fn startpos() -> Self {
        Self::from_fen(FenParser::STARTPOS).expect("STARTPOS is valid")
    }

    /// Creates a position from a FEN string.
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let parsed = FenParser::parse(fen)?;
        let mut position = Position::empty();

        // Parse piece placement
        let ranks: Vec<&str> = parsed.piece_placement.split('/').collect();
        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_idx; // FEN starts from rank 8
            let mut file = 0usize;

            for c in rank_str.chars() {
                if let Some(digit) = c.to_digit(10) {
                    file += digit as usize;
                } else if let Some((piece, color)) = Piece::from_fen_char(c) {
                    let sq = unsafe { Square::from_index_unchecked((rank * 8 + file) as u8) };
                    position.pieces[piece.index()].set(sq);
                    position.colors[color.index()].set(sq);
                    file += 1;
                }
            }
        }

        // Active color
        position.side_to_move = match parsed.active_color {
            'w' => Color::White,
            'b' => Color::Black,
            _ => unreachable!("FEN parser validated this"),
        };

        // Castling rights
        let mut castling = 0u8;
        for c in parsed.castling.chars() {
            match c {
                'K' => castling |= CastlingRights::WHITE_KINGSIDE,
                'Q' => castling |= CastlingRights::WHITE_QUEENSIDE,
                'k' => castling |= CastlingRights::BLACK_KINGSIDE,
                'q' => castling |= CastlingRights::BLACK_QUEENSIDE,
                '-' => {}
                _ => {}
            }
        }
        position.castling = CastlingRights::new(castling);

        // En passant
        position.en_passant = if parsed.en_passant == "-" {
            None
        } else {
            Square::from_algebraic(&parsed.en_passant)
        };

        position.halfmove_clock = parsed.halfmove_clock;
        position.fullmove_number = parsed.fullmove_number;

        Ok(position)
    }

    /// Converts the position to a FEN string.
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Piece placement
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let sq = unsafe { Square::from_index_unchecked(rank * 8 + file) };
                if let Some((piece, color)) = self.piece_at(sq) {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push(piece.to_fen_char(color));
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // Active color
        fen.push(' ');
        fen.push(match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling
        fen.push(' ');
        if self.castling.raw() == 0 {
            fen.push('-');
        } else {
            if self.castling.can_castle_kingside(Color::White) {
                fen.push('K');
            }
            if self.castling.can_castle_queenside(Color::White) {
                fen.push('Q');
            }
            if self.castling.can_castle_kingside(Color::Black) {
                fen.push('k');
            }
            if self.castling.can_castle_queenside(Color::Black) {
                fen.push('q');
            }
        }

        // En passant
        fen.push(' ');
        match self.en_passant {
            Some(sq) => fen.push_str(&sq.to_algebraic()),
            None => fen.push('-'),
        }

        // Halfmove clock and fullmove number
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    /// Returns the piece and color at the given square, if any.
    pub fn piece_at(&self, sq: Square) -> Option<(Piece, Color)> {
        let bb = Bitboard::from_square(sq);

        // Check if any piece is on this square
        let color = if (self.colors[Color::White.index()] & bb).is_not_empty() {
            Color::White
        } else if (self.colors[Color::Black.index()] & bb).is_not_empty() {
            Color::Black
        } else {
            return None;
        };

        // Find which piece type
        for piece in Piece::ALL {
            if (self.pieces[piece.index()] & bb).is_not_empty() {
                return Some((piece, color));
            }
        }

        None
    }

    /// Returns a bitboard of all occupied squares.
    #[inline]
    pub fn occupied(&self) -> Bitboard {
        self.colors[0] | self.colors[1]
    }

    /// Returns a bitboard of all empty squares.
    #[inline]
    pub fn empty_squares(&self) -> Bitboard {
        !self.occupied()
    }

    /// Returns a bitboard of pieces of the given type and color.
    #[inline]
    pub fn pieces_of(&self, piece: Piece, color: Color) -> Bitboard {
        self.pieces[piece.index()] & self.colors[color.index()]
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::startpos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_fen_roundtrip() {
        let pos = Position::startpos();
        assert_eq!(pos.to_fen(), FenParser::STARTPOS);
    }

    #[test]
    fn custom_fen_roundtrip() {
        let fen = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
        let pos = Position::from_fen(fen).unwrap();
        assert_eq!(pos.to_fen(), fen);
    }

    #[test]
    fn piece_at() {
        let pos = Position::startpos();
        assert_eq!(pos.piece_at(Square::E1), Some((Piece::King, Color::White)));
        assert_eq!(pos.piece_at(Square::E8), Some((Piece::King, Color::Black)));
        assert_eq!(
            pos.piece_at(Square::new(chess_core::File::E, chess_core::Rank::R4)),
            None
        );
    }

    #[test]
    fn castling_rights() {
        let mut rights = CastlingRights::ALL;
        assert!(rights.can_castle_kingside(Color::White));
        assert!(rights.can_castle_queenside(Color::Black));

        rights.remove_kingside(Color::White);
        assert!(!rights.can_castle_kingside(Color::White));
        assert!(rights.can_castle_queenside(Color::White));
    }

    #[test]
    fn castling_rights_remove_queenside() {
        let mut rights = CastlingRights::ALL;
        rights.remove_queenside(Color::Black);
        assert!(rights.can_castle_kingside(Color::Black));
        assert!(!rights.can_castle_queenside(Color::Black));
    }

    #[test]
    fn castling_rights_remove_color() {
        let mut rights = CastlingRights::ALL;
        rights.remove_color(Color::White);
        assert!(!rights.can_castle_kingside(Color::White));
        assert!(!rights.can_castle_queenside(Color::White));
        assert!(rights.can_castle_kingside(Color::Black));
        assert!(rights.can_castle_queenside(Color::Black));
    }

    #[test]
    fn castling_rights_none() {
        let rights = CastlingRights::NONE;
        assert!(!rights.can_castle_kingside(Color::White));
        assert!(!rights.can_castle_queenside(Color::White));
        assert!(!rights.can_castle_kingside(Color::Black));
        assert!(!rights.can_castle_queenside(Color::Black));
        assert_eq!(rights.raw(), 0);
    }

    #[test]
    fn position_empty() {
        let pos = Position::empty();
        assert_eq!(pos.side_to_move, Color::White);
        assert_eq!(pos.castling.raw(), 0);
        assert_eq!(pos.en_passant, None);
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_number, 1);
        assert!(pos.occupied().is_empty());
    }

    #[test]
    fn position_default() {
        let pos = Position::default();
        assert_eq!(pos.to_fen(), FenParser::STARTPOS);
    }

    #[test]
    fn position_occupied_empty() {
        let pos = Position::startpos();
        // Starting position has 32 pieces
        assert_eq!(pos.occupied().count(), 32);
        assert_eq!(pos.empty_squares().count(), 32);
    }

    #[test]
    fn position_pieces_of() {
        let pos = Position::startpos();
        // White pawns on rank 2
        assert_eq!(pos.pieces_of(Piece::Pawn, Color::White).count(), 8);
        // Black pawns on rank 7
        assert_eq!(pos.pieces_of(Piece::Pawn, Color::Black).count(), 8);
        // One king each
        assert_eq!(pos.pieces_of(Piece::King, Color::White).count(), 1);
        assert_eq!(pos.pieces_of(Piece::King, Color::Black).count(), 1);
    }

    #[test]
    fn position_with_en_passant() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let pos = Position::from_fen(fen).unwrap();
        assert!(pos.en_passant.is_some());
        assert_eq!(pos.en_passant.unwrap().to_algebraic(), "e3");
        assert_eq!(pos.to_fen(), fen);
    }

    #[test]
    fn position_no_castling() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        assert!(!pos.castling.can_castle_kingside(Color::White));
        assert!(!pos.castling.can_castle_queenside(Color::White));
        assert_eq!(pos.to_fen(), fen);
    }

    #[test]
    fn position_black_to_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        assert_eq!(pos.side_to_move, Color::Black);
    }
}
