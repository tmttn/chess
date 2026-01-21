//! SVG chess board rendering for HTML exports.
//!
//! This module provides functionality to render chess positions as SVG graphics
//! that can be embedded in HTML exports.

use askama::Template;

/// A piece to render on the board.
///
/// Represents a single chess piece with its position and Unicode symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PieceView {
    /// Row index (0-7, where 0 is the 8th rank).
    pub row: usize,
    /// Column index (0-7, where 0 is the a-file).
    pub col: usize,
    /// Unicode chess piece symbol.
    pub symbol: char,
}

/// SVG chess board template.
///
/// Renders an 8x8 chess board with pieces positioned according to the FEN notation.
#[derive(Template)]
#[template(path = "components/board.html")]
pub struct BoardTemplate {
    /// Pieces to render on the board.
    pub pieces: Vec<PieceView>,
}

impl BoardTemplate {
    /// Create a board from a FEN position string.
    ///
    /// Parses the FEN notation and converts it to piece positions with Unicode symbols.
    ///
    /// # Arguments
    ///
    /// * `fen` - A FEN string (only the piece placement part is used).
    ///
    /// # Examples
    ///
    /// ```
    /// use bot_arena_server::templates::BoardTemplate;
    ///
    /// // Starting position
    /// let board = BoardTemplate::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// assert_eq!(board.pieces.len(), 32);
    /// ```
    #[must_use]
    pub fn from_fen(fen: &str) -> Self {
        let mut pieces = Vec::new();
        let board_part = fen.split_whitespace().next().unwrap_or("");

        for (row, rank) in board_part.split('/').enumerate() {
            let mut col = 0;
            for c in rank.chars() {
                if let Some(skip) = c.to_digit(10) {
                    col += skip as usize;
                } else {
                    let symbol = Self::piece_to_symbol(c);
                    if let Some(symbol) = symbol {
                        pieces.push(PieceView { row, col, symbol });
                    }
                    col += 1;
                }
            }
        }

        Self { pieces }
    }

    /// Convert a FEN piece character to its Unicode chess symbol.
    ///
    /// # Arguments
    ///
    /// * `piece` - A FEN piece character (K, Q, R, B, N, P for white; k, q, r, b, n, p for black).
    ///
    /// # Returns
    ///
    /// The corresponding Unicode chess symbol, or `None` if the character is not a valid piece.
    #[must_use]
    const fn piece_to_symbol(piece: char) -> Option<char> {
        match piece {
            // White pieces
            'K' => Some('\u{2654}'), // White King
            'Q' => Some('\u{2655}'), // White Queen
            'R' => Some('\u{2656}'), // White Rook
            'B' => Some('\u{2657}'), // White Bishop
            'N' => Some('\u{2658}'), // White Knight
            'P' => Some('\u{2659}'), // White Pawn
            // Black pieces
            'k' => Some('\u{265A}'), // Black King
            'q' => Some('\u{265B}'), // Black Queen
            'r' => Some('\u{265C}'), // Black Rook
            'b' => Some('\u{265D}'), // Black Bishop
            'n' => Some('\u{265E}'), // Black Knight
            'p' => Some('\u{265F}'), // Black Pawn
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_from_starting_fen() {
        let board =
            BoardTemplate::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(board.pieces.len(), 32);
    }

    #[test]
    fn test_board_from_empty_fen() {
        let board = BoardTemplate::from_fen("8/8/8/8/8/8/8/8 w - - 0 1");
        assert_eq!(board.pieces.len(), 0);
    }

    #[test]
    fn test_board_renders() {
        let board = BoardTemplate::from_fen("8/8/8/8/8/8/8/8 w - - 0 1");
        let html = board.render().unwrap();
        assert!(html.contains("<svg"));
        assert!(html.contains("viewBox"));
    }

    #[test]
    fn test_starting_position_pieces_correct() {
        let board =
            BoardTemplate::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        // Check that we have the white king at the correct position (row 7, col 4)
        let white_king = board
            .pieces
            .iter()
            .find(|p| p.symbol == '\u{2654}')
            .expect("White king should exist");
        assert_eq!(white_king.row, 7);
        assert_eq!(white_king.col, 4);

        // Check that we have the black king at the correct position (row 0, col 4)
        let black_king = board
            .pieces
            .iter()
            .find(|p| p.symbol == '\u{265A}')
            .expect("Black king should exist");
        assert_eq!(black_king.row, 0);
        assert_eq!(black_king.col, 4);
    }

    #[test]
    fn test_piece_to_symbol() {
        assert_eq!(BoardTemplate::piece_to_symbol('K'), Some('\u{2654}'));
        assert_eq!(BoardTemplate::piece_to_symbol('Q'), Some('\u{2655}'));
        assert_eq!(BoardTemplate::piece_to_symbol('R'), Some('\u{2656}'));
        assert_eq!(BoardTemplate::piece_to_symbol('B'), Some('\u{2657}'));
        assert_eq!(BoardTemplate::piece_to_symbol('N'), Some('\u{2658}'));
        assert_eq!(BoardTemplate::piece_to_symbol('P'), Some('\u{2659}'));
        assert_eq!(BoardTemplate::piece_to_symbol('k'), Some('\u{265A}'));
        assert_eq!(BoardTemplate::piece_to_symbol('q'), Some('\u{265B}'));
        assert_eq!(BoardTemplate::piece_to_symbol('r'), Some('\u{265C}'));
        assert_eq!(BoardTemplate::piece_to_symbol('b'), Some('\u{265D}'));
        assert_eq!(BoardTemplate::piece_to_symbol('n'), Some('\u{265E}'));
        assert_eq!(BoardTemplate::piece_to_symbol('p'), Some('\u{265F}'));
        assert_eq!(BoardTemplate::piece_to_symbol('x'), None);
    }

    #[test]
    fn test_board_renders_with_pieces() {
        let board =
            BoardTemplate::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let html = board.render().unwrap();

        // Should contain SVG structure
        assert!(html.contains("<svg"));
        assert!(html.contains("</svg>"));
        assert!(html.contains("viewBox=\"0 0 400 400\""));

        // Should contain board squares
        assert!(html.contains("<rect"));
        assert!(html.contains("class=\"light\""));
        assert!(html.contains("class=\"dark\""));

        // Should contain piece text elements
        assert!(html.contains("<text"));
        assert!(html.contains("class=\"piece\""));
    }

    #[test]
    fn test_partial_position() {
        // Position with only kings
        let board = BoardTemplate::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
        assert_eq!(board.pieces.len(), 2);

        let white_king = board.pieces.iter().find(|p| p.symbol == '\u{2654}');
        let black_king = board.pieces.iter().find(|p| p.symbol == '\u{265A}');

        assert!(white_king.is_some());
        assert!(black_king.is_some());

        let wk = white_king.unwrap();
        assert_eq!(wk.row, 7);
        assert_eq!(wk.col, 4);

        let bk = black_king.unwrap();
        assert_eq!(bk.row, 0);
        assert_eq!(bk.col, 4);
    }

    #[test]
    fn test_invalid_fen_handles_gracefully() {
        // Empty string
        let board = BoardTemplate::from_fen("");
        assert_eq!(board.pieces.len(), 0);

        // Invalid characters are skipped
        let board = BoardTemplate::from_fen("xxx/xxx/8/8/8/8/8/8 w - - 0 1");
        assert_eq!(board.pieces.len(), 0);
    }

    #[test]
    fn test_piece_view_equality() {
        let p1 = PieceView {
            row: 0,
            col: 0,
            symbol: '\u{2654}',
        };
        let p2 = PieceView {
            row: 0,
            col: 0,
            symbol: '\u{2654}',
        };
        let p3 = PieceView {
            row: 1,
            col: 0,
            symbol: '\u{2654}',
        };

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }
}
