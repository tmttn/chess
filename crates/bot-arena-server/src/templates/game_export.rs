//! Game export template.
//!
//! This module provides an Askama template for rendering individual chess games
//! as standalone HTML pages with a visual board and move list.

use askama::Template;

/// Game export HTML template.
///
/// Renders a single chess game as a standalone HTML page with:
/// - Game information (players, result, opening)
/// - Visual chess board showing the final position
/// - Full move list in standard notation
#[derive(Template)]
#[template(path = "export_game.html")]
pub struct GameExportTemplate {
    /// Name of the bot playing white.
    pub white: String,
    /// Name of the bot playing black.
    pub black: String,
    /// Game result (e.g., "1-0", "0-1", "1/2-1/2").
    pub result: String,
    /// Optional opening name.
    pub opening: Option<String>,
    /// Pre-rendered SVG board from BoardTemplate.
    pub board: String,
    /// Move pairs for display (white_move, optional black_move).
    pub move_pairs: Vec<(String, Option<String>)>,
}

impl GameExportTemplate {
    /// Convert a flat list of moves into pairs (white_move, black_move).
    ///
    /// Takes a sequential list of moves and groups them into pairs for display
    /// in a move list format where each row shows the move number, white's move,
    /// and black's move.
    ///
    /// # Arguments
    ///
    /// * `moves` - A vector of move strings in sequential order.
    ///
    /// # Returns
    ///
    /// A vector of tuples where each tuple contains:
    /// - White's move (always present)
    /// - Black's move (optional, `None` for the last move if the game ends on white's turn)
    ///
    /// # Examples
    ///
    /// ```
    /// use bot_arena_server::templates::GameExportTemplate;
    ///
    /// let moves = vec!["e4".into(), "e5".into(), "Nf3".into(), "Nc6".into()];
    /// let pairs = GameExportTemplate::pair_moves(moves);
    /// assert_eq!(pairs.len(), 2);
    /// assert_eq!(pairs[0], ("e4".into(), Some("e5".into())));
    /// assert_eq!(pairs[1], ("Nf3".into(), Some("Nc6".into())));
    /// ```
    #[must_use]
    pub fn pair_moves(moves: Vec<String>) -> Vec<(String, Option<String>)> {
        moves
            .chunks(2)
            .map(|chunk| {
                let white = chunk.first().cloned().unwrap_or_default();
                let black = chunk.get(1).cloned();
                (white, black)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pair_moves_even() {
        let moves = vec!["e4".into(), "e5".into(), "Nf3".into(), "Nc6".into()];
        let pairs = GameExportTemplate::pair_moves(moves);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("e4".into(), Some("e5".into())));
        assert_eq!(pairs[1], ("Nf3".into(), Some("Nc6".into())));
    }

    #[test]
    fn test_pair_moves_odd() {
        let moves = vec!["e4".into(), "e5".into(), "Nf3".into()];
        let pairs = GameExportTemplate::pair_moves(moves);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("e4".into(), Some("e5".into())));
        assert_eq!(pairs[1], ("Nf3".into(), None));
    }

    #[test]
    fn test_pair_moves_empty() {
        let moves: Vec<String> = vec![];
        let pairs = GameExportTemplate::pair_moves(moves);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_pair_moves_single() {
        let moves = vec!["e4".into()];
        let pairs = GameExportTemplate::pair_moves(moves);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("e4".into(), None));
    }

    #[test]
    fn test_game_export_renders() {
        let template = GameExportTemplate {
            white: "minimax".into(),
            black: "random".into(),
            result: "1-0".into(),
            opening: Some("Italian Game".into()),
            board: "<svg></svg>".into(),
            move_pairs: vec![("e4".into(), Some("e5".into()))],
        };
        let html = template.render().unwrap();
        assert!(html.contains("minimax"));
        assert!(html.contains("random"));
        assert!(html.contains("Italian Game"));
        assert!(html.contains("1-0"));
        assert!(html.contains("<svg></svg>"));
    }

    #[test]
    fn test_game_export_without_opening() {
        let template = GameExportTemplate {
            white: "bot_a".into(),
            black: "bot_b".into(),
            result: "0-1".into(),
            opening: None,
            board: "<svg></svg>".into(),
            move_pairs: vec![],
        };
        let html = template.render().unwrap();
        assert!(html.contains("bot_a"));
        assert!(html.contains("bot_b"));
        assert!(!html.contains("Opening:"));
    }

    #[test]
    fn test_game_export_with_multiple_moves() {
        let template = GameExportTemplate {
            white: "stockfish".into(),
            black: "komodo".into(),
            result: "1/2-1/2".into(),
            opening: Some("Sicilian Defense".into()),
            board: "<svg></svg>".into(),
            move_pairs: vec![
                ("e4".into(), Some("c5".into())),
                ("Nf3".into(), Some("d6".into())),
                ("d4".into(), Some("cxd4".into())),
            ],
        };
        let html = template.render().unwrap();
        assert!(html.contains("stockfish"));
        assert!(html.contains("komodo"));
        assert!(html.contains("Sicilian Defense"));
        assert!(html.contains("e4"));
        assert!(html.contains("c5"));
        assert!(html.contains("Nf3"));
    }

    #[test]
    fn test_template_contains_required_elements() {
        let template = GameExportTemplate {
            white: "test_white".into(),
            black: "test_black".into(),
            result: "*".into(),
            opening: None,
            board: "<svg></svg>".into(),
            move_pairs: vec![],
        };
        let html = template.render().unwrap();
        // Check HTML structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("Game Viewer"));
        assert!(html.contains("Generated by Bot Arena"));
    }

    #[test]
    fn test_game_export_escapes_html_in_names() {
        let template = GameExportTemplate {
            white: "bot<script>".into(),
            black: "bot&evil".into(),
            result: "1-0".into(),
            opening: None,
            board: "<svg></svg>".into(),
            move_pairs: vec![],
        };
        let html = template.render().unwrap();
        // Askama should escape HTML special characters
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;") || html.contains("bot&lt;script&gt;"));
    }
}
