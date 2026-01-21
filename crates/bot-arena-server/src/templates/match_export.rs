//! Match export template.
//!
//! This module provides an Askama template for rendering match results as a
//! standalone HTML page that can be downloaded and shared.

use askama::Template;

/// A game summary for the match export.
///
/// Contains the minimal information needed to display a game in the match report.
#[derive(Debug, Clone)]
pub struct GameSummary {
    /// Name of the bot playing white.
    pub white: String,
    /// Name of the bot playing black.
    pub black: String,
    /// Game result (e.g., "1-0", "0-1", "1/2-1/2").
    pub result: String,
    /// Total number of moves in the game.
    pub move_count: i32,
}

/// Match export HTML template.
///
/// Renders a complete match report as a standalone HTML page with styling.
#[derive(Template)]
#[template(path = "export_match.html")]
pub struct MatchExportTemplate {
    /// Name of the bot playing white in the match.
    pub white_bot: String,
    /// Name of the bot playing black in the match.
    pub black_bot: String,
    /// Score achieved by the white bot.
    pub white_score: f64,
    /// Score achieved by the black bot.
    pub black_score: f64,
    /// List of game summaries for the match.
    pub games: Vec<GameSummary>,
    /// Optional creation date for the match report.
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_export_renders() {
        let template = MatchExportTemplate {
            white_bot: "minimax".to_string(),
            black_bot: "random".to_string(),
            white_score: 2.5,
            black_score: 0.5,
            games: vec![GameSummary {
                white: "minimax".to_string(),
                black: "random".to_string(),
                result: "1-0".to_string(),
                move_count: 40,
            }],
            created_at: Some("2025-01-21".to_string()),
        };

        let html = template.render().unwrap();
        assert!(html.contains("minimax"));
        assert!(html.contains("2.5 - 0.5"));
    }

    #[test]
    fn test_match_export_without_date() {
        let template = MatchExportTemplate {
            white_bot: "bot_a".to_string(),
            black_bot: "bot_b".to_string(),
            white_score: 1.0,
            black_score: 1.0,
            games: vec![],
            created_at: None,
        };

        let html = template.render().unwrap();
        assert!(html.contains("bot_a"));
        assert!(html.contains("bot_b"));
        assert!(!html.contains("Date:"));
    }

    #[test]
    fn test_match_export_with_multiple_games() {
        let template = MatchExportTemplate {
            white_bot: "stockfish".to_string(),
            black_bot: "komodo".to_string(),
            white_score: 3.5,
            black_score: 2.5,
            games: vec![
                GameSummary {
                    white: "stockfish".to_string(),
                    black: "komodo".to_string(),
                    result: "1-0".to_string(),
                    move_count: 45,
                },
                GameSummary {
                    white: "komodo".to_string(),
                    black: "stockfish".to_string(),
                    result: "1/2-1/2".to_string(),
                    move_count: 60,
                },
                GameSummary {
                    white: "stockfish".to_string(),
                    black: "komodo".to_string(),
                    result: "1-0".to_string(),
                    move_count: 38,
                },
            ],
            created_at: Some("2025-01-21".to_string()),
        };

        let html = template.render().unwrap();
        assert!(html.contains("stockfish"));
        assert!(html.contains("komodo"));
        assert!(html.contains("3.5 - 2.5"));
        assert!(html.contains("3 games played"));
    }

    #[test]
    fn test_game_summary_result_styling() {
        let template = MatchExportTemplate {
            white_bot: "white".to_string(),
            black_bot: "black".to_string(),
            white_score: 2.0,
            black_score: 1.0,
            games: vec![
                GameSummary {
                    white: "white".to_string(),
                    black: "black".to_string(),
                    result: "1-0".to_string(),
                    move_count: 30,
                },
                GameSummary {
                    white: "black".to_string(),
                    black: "white".to_string(),
                    result: "0-1".to_string(),
                    move_count: 25,
                },
                GameSummary {
                    white: "white".to_string(),
                    black: "black".to_string(),
                    result: "1/2-1/2".to_string(),
                    move_count: 50,
                },
            ],
            created_at: None,
        };

        let html = template.render().unwrap();
        // Check that result styling classes are applied
        assert!(html.contains("result-1-0"));
        assert!(html.contains("result-0-1"));
        assert!(html.contains("result-draw"));
    }

    #[test]
    fn test_template_contains_required_elements() {
        let template = MatchExportTemplate {
            white_bot: "test_white".to_string(),
            black_bot: "test_black".to_string(),
            white_score: 0.0,
            black_score: 0.0,
            games: vec![],
            created_at: None,
        };

        let html = template.render().unwrap();
        // Check HTML structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("Match Report"));
        assert!(html.contains("Games"));
        assert!(html.contains("Generated by Bot Arena"));
    }
}
