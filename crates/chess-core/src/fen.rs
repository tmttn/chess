//! FEN (Forsyth-Edwards Notation) parsing and serialization.

use thiserror::Error;

/// Errors that can occur when parsing FEN strings.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FenError {
    #[error("invalid FEN: expected 6 parts, got {0}")]
    InvalidPartCount(usize),

    #[error("invalid piece placement: {0}")]
    InvalidPiecePlacement(String),

    #[error("invalid active color: expected 'w' or 'b', got '{0}'")]
    InvalidActiveColor(String),

    #[error("invalid castling rights: {0}")]
    InvalidCastlingRights(String),

    #[error("invalid en passant square: {0}")]
    InvalidEnPassantSquare(String),

    #[error("invalid halfmove clock: {0}")]
    InvalidHalfmoveClock(String),

    #[error("invalid fullmove number: {0}")]
    InvalidFullmoveNumber(String),
}

/// Parsed FEN data.
///
/// This struct holds the raw parsed FEN components. The engine
/// is responsible for converting this into its internal position
/// representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenParser {
    /// Piece placement string (e.g., "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    pub piece_placement: String,
    /// Active color ('w' or 'b')
    pub active_color: char,
    /// Castling availability (e.g., "KQkq", "-")
    pub castling: String,
    /// En passant target square (e.g., "e3", "-")
    pub en_passant: String,
    /// Halfmove clock (for 50-move rule)
    pub halfmove_clock: u32,
    /// Fullmove number
    pub fullmove_number: u32,
}

impl FenParser {
    /// The standard starting position FEN.
    pub const STARTPOS: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// Parses a FEN string.
    pub fn parse(fen: &str) -> Result<Self, FenError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(FenError::InvalidPartCount(parts.len()));
        }

        // Validate piece placement
        let piece_placement = parts[0];
        Self::validate_piece_placement(piece_placement)?;

        // Validate active color
        let active_color = match parts[1] {
            "w" => 'w',
            "b" => 'b',
            other => return Err(FenError::InvalidActiveColor(other.to_string())),
        };

        // Validate castling rights
        let castling = parts[2];
        Self::validate_castling(castling)?;

        // Validate en passant
        let en_passant = parts[3];
        Self::validate_en_passant(en_passant)?;

        // Parse halfmove clock
        let halfmove_clock = parts[4]
            .parse::<u32>()
            .map_err(|_| FenError::InvalidHalfmoveClock(parts[4].to_string()))?;

        // Parse fullmove number
        let fullmove_number = parts[5]
            .parse::<u32>()
            .map_err(|_| FenError::InvalidFullmoveNumber(parts[5].to_string()))?;

        Ok(FenParser {
            piece_placement: piece_placement.to_string(),
            active_color,
            castling: castling.to_string(),
            en_passant: en_passant.to_string(),
            halfmove_clock,
            fullmove_number,
        })
    }

    fn validate_piece_placement(placement: &str) -> Result<(), FenError> {
        let ranks: Vec<&str> = placement.split('/').collect();
        if ranks.len() != 8 {
            return Err(FenError::InvalidPiecePlacement(format!(
                "expected 8 ranks, got {}",
                ranks.len()
            )));
        }

        for (i, rank) in ranks.iter().enumerate() {
            let mut squares = 0;
            for c in rank.chars() {
                if c.is_ascii_digit() {
                    squares += c.to_digit(10).unwrap();
                } else if "pnbrqkPNBRQK".contains(c) {
                    squares += 1;
                } else {
                    return Err(FenError::InvalidPiecePlacement(format!(
                        "invalid character '{}' in rank {}",
                        c,
                        8 - i
                    )));
                }
            }
            if squares != 8 {
                return Err(FenError::InvalidPiecePlacement(format!(
                    "rank {} has {} squares, expected 8",
                    8 - i,
                    squares
                )));
            }
        }

        Ok(())
    }

    fn validate_castling(castling: &str) -> Result<(), FenError> {
        if castling == "-" {
            return Ok(());
        }

        for c in castling.chars() {
            if !"KQkq".contains(c) {
                return Err(FenError::InvalidCastlingRights(format!(
                    "invalid character '{}'",
                    c
                )));
            }
        }

        Ok(())
    }

    fn validate_en_passant(ep: &str) -> Result<(), FenError> {
        if ep == "-" {
            return Ok(());
        }

        if ep.len() != 2 {
            return Err(FenError::InvalidEnPassantSquare(ep.to_string()));
        }

        let chars: Vec<char> = ep.chars().collect();
        if !('a'..='h').contains(&chars[0]) || !(chars[1] == '3' || chars[1] == '6') {
            return Err(FenError::InvalidEnPassantSquare(ep.to_string()));
        }

        Ok(())
    }

    /// Converts the parsed FEN back to a FEN string.
    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.piece_placement,
            self.active_color,
            self.castling,
            self.en_passant,
            self.halfmove_clock,
            self.fullmove_number
        )
    }
}

impl Default for FenParser {
    fn default() -> Self {
        Self::parse(Self::STARTPOS).expect("STARTPOS is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_startpos() {
        let fen = FenParser::parse(FenParser::STARTPOS).unwrap();
        assert_eq!(fen.active_color, 'w');
        assert_eq!(fen.castling, "KQkq");
        assert_eq!(fen.en_passant, "-");
        assert_eq!(fen.halfmove_clock, 0);
        assert_eq!(fen.fullmove_number, 1);
    }

    #[test]
    fn parse_custom_position() {
        let fen = FenParser::parse(
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
        )
        .unwrap();
        assert_eq!(fen.active_color, 'w');
        assert_eq!(fen.halfmove_clock, 2);
        assert_eq!(fen.fullmove_number, 3);
    }

    #[test]
    fn roundtrip() {
        let original = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let parsed = FenParser::parse(original).unwrap();
        assert_eq!(parsed.to_fen(), original);
    }

    #[test]
    fn invalid_fen() {
        assert!(matches!(
            FenParser::parse("invalid"),
            Err(FenError::InvalidPartCount(_))
        ));

        assert!(matches!(
            FenParser::parse("8/8/8/8/8/8/8/8 x KQkq - 0 1"),
            Err(FenError::InvalidActiveColor(_))
        ));
    }
}
