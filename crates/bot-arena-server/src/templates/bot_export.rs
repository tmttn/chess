//! Bot profile export template.
//!
//! This module provides an Askama template for rendering bot profiles as
//! standalone HTML pages with statistics and Elo history charts.

use askama::Template;

/// Elo history data point.
///
/// Represents a single point in time for a bot's Elo rating history.
#[derive(Debug, Clone)]
pub struct EloPoint {
    /// The Elo rating at this point in time.
    pub elo: i32,
    /// The date when this rating was recorded (ISO format).
    pub date: String,
}

/// Bot profile export HTML template.
///
/// Renders a bot's profile as a standalone HTML page with:
/// - Bot name and current Elo rating
/// - Win/loss/draw statistics
/// - Elo history chart (if history is available)
#[derive(Template)]
#[template(path = "export_bot.html")]
pub struct BotExportTemplate {
    /// Name of the bot.
    pub name: String,
    /// Current Elo rating.
    pub elo: i32,
    /// Total number of games played.
    pub games_played: i32,
    /// Number of wins.
    pub wins: i32,
    /// Number of draws.
    pub draws: i32,
    /// Number of losses.
    pub losses: i32,
    /// Win rate as a string (e.g., "70.0").
    pub win_rate: String,
    /// Historical Elo data points for the chart.
    pub elo_history: Vec<EloPoint>,
    /// Pre-rendered SVG chart of Elo history.
    pub elo_chart: String,
}

impl BotExportTemplate {
    /// Generate a simple SVG line chart for Elo history.
    ///
    /// Creates an inline SVG visualization showing the bot's Elo rating
    /// over time as a line chart.
    ///
    /// # Arguments
    ///
    /// * `history` - A slice of `EloPoint` values to visualize.
    ///
    /// # Returns
    ///
    /// An SVG string that can be embedded directly in HTML, or an empty
    /// string if the history is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bot_arena_server::templates::bot_export::{BotExportTemplate, EloPoint};
    ///
    /// let history = vec![
    ///     EloPoint { elo: 1500, date: "2025-01-01".into() },
    ///     EloPoint { elo: 1520, date: "2025-01-02".into() },
    /// ];
    /// let chart = BotExportTemplate::generate_elo_chart(&history);
    /// assert!(chart.contains("<svg"));
    /// ```
    #[must_use]
    pub fn generate_elo_chart(history: &[EloPoint]) -> String {
        if history.is_empty() {
            return String::new();
        }

        let min_elo = history.iter().map(|p| p.elo).min().unwrap_or(1000) - 50;
        let max_elo = history.iter().map(|p| p.elo).max().unwrap_or(2000) + 50;
        let range = (max_elo - min_elo).max(1) as f64;

        let width = 600.0;
        let height = 200.0;
        let padding = 40.0;
        let inner_width = width - 2.0 * padding;
        let inner_height = height - 2.0 * padding;

        let points: Vec<String> = history
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let x = padding + (i as f64 / (history.len() - 1).max(1) as f64) * inner_width;
                let y = height - padding - ((p.elo - min_elo) as f64 / range) * inner_height;
                format!("{x:.1},{y:.1}")
            })
            .collect();

        format!(
            "<svg viewBox=\"0 0 {width} {height}\" xmlns=\"http://www.w3.org/2000/svg\">\
                <rect fill=\"#0f3460\" width=\"{width}\" height=\"{height}\"/>\
                <polyline fill=\"none\" stroke=\"#e94560\" stroke-width=\"2\" points=\"{points}\"/>\
                <text x=\"{padding}\" y=\"15\" fill=\"#888\" font-size=\"12\">{max_elo}</text>\
                <text x=\"{padding}\" y=\"{bottom}\" fill=\"#888\" font-size=\"12\">{min_elo}</text>\
            </svg>",
            width = width,
            height = height,
            points = points.join(" "),
            padding = padding,
            max_elo = max_elo,
            min_elo = min_elo,
            bottom = height - 5.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_elo_chart_empty() {
        let chart = BotExportTemplate::generate_elo_chart(&[]);
        assert!(chart.is_empty());
    }

    #[test]
    fn test_generate_elo_chart_single_point() {
        let history = vec![EloPoint {
            elo: 1500,
            date: "2025-01-01".into(),
        }];
        let chart = BotExportTemplate::generate_elo_chart(&history);
        assert!(chart.contains("<svg"));
        assert!(chart.contains("polyline"));
    }

    #[test]
    fn test_generate_elo_chart_multiple_points() {
        let history = vec![
            EloPoint {
                elo: 1500,
                date: "2025-01-01".into(),
            },
            EloPoint {
                elo: 1520,
                date: "2025-01-02".into(),
            },
            EloPoint {
                elo: 1480,
                date: "2025-01-03".into(),
            },
            EloPoint {
                elo: 1550,
                date: "2025-01-04".into(),
            },
        ];
        let chart = BotExportTemplate::generate_elo_chart(&history);
        assert!(chart.contains("<svg"));
        assert!(chart.contains("polyline"));
        assert!(chart.contains("viewBox"));
        // Check min/max labels are present
        assert!(chart.contains("1430")); // min - 50 = 1480 - 50
        assert!(chart.contains("1600")); // max + 50 = 1550 + 50
    }

    #[test]
    fn test_generate_elo_chart_contains_points() {
        let history = vec![
            EloPoint {
                elo: 1500,
                date: "2025-01-01".into(),
            },
            EloPoint {
                elo: 1600,
                date: "2025-01-02".into(),
            },
        ];
        let chart = BotExportTemplate::generate_elo_chart(&history);
        assert!(chart.contains("points="));
    }

    #[test]
    fn test_bot_export_renders() {
        let template = BotExportTemplate {
            name: "minimax".into(),
            elo: 1650,
            games_played: 100,
            wins: 60,
            draws: 20,
            losses: 20,
            win_rate: "70.0".into(),
            elo_history: vec![],
            elo_chart: String::new(),
        };
        let html = template.render().unwrap();
        assert!(html.contains("minimax"));
        assert!(html.contains("1650 Elo"));
        assert!(html.contains("100")); // games_played
        assert!(html.contains("70.0%")); // win_rate
    }

    #[test]
    fn test_bot_export_with_elo_history() {
        let history = vec![
            EloPoint {
                elo: 1500,
                date: "2025-01-01".into(),
            },
            EloPoint {
                elo: 1520,
                date: "2025-01-02".into(),
            },
        ];
        let chart = BotExportTemplate::generate_elo_chart(&history);
        let template = BotExportTemplate {
            name: "stockfish".into(),
            elo: 1520,
            games_played: 10,
            wins: 8,
            draws: 1,
            losses: 1,
            win_rate: "85.0".into(),
            elo_history: history,
            elo_chart: chart,
        };
        let html = template.render().unwrap();
        assert!(html.contains("stockfish"));
        assert!(html.contains("1520 Elo"));
        assert!(html.contains("Elo History"));
        assert!(html.contains("<svg"));
    }

    #[test]
    fn test_bot_export_without_history_hides_chart() {
        let template = BotExportTemplate {
            name: "random".into(),
            elo: 1200,
            games_played: 5,
            wins: 1,
            draws: 1,
            losses: 3,
            win_rate: "30.0".into(),
            elo_history: vec![],
            elo_chart: String::new(),
        };
        let html = template.render().unwrap();
        assert!(html.contains("random"));
        assert!(!html.contains("Elo History"));
    }

    #[test]
    fn test_template_contains_required_elements() {
        let template = BotExportTemplate {
            name: "test_bot".into(),
            elo: 1500,
            games_played: 0,
            wins: 0,
            draws: 0,
            losses: 0,
            win_rate: "0.0".into(),
            elo_history: vec![],
            elo_chart: String::new(),
        };
        let html = template.render().unwrap();
        // Check HTML structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("Bot Profile:") || html.contains("test_bot"));
        assert!(html.contains("Generated by Bot Arena"));
    }

    #[test]
    fn test_bot_export_escapes_html_in_name() {
        let template = BotExportTemplate {
            name: "bot<script>alert(1)</script>".into(),
            elo: 1500,
            games_played: 0,
            wins: 0,
            draws: 0,
            losses: 0,
            win_rate: "0.0".into(),
            elo_history: vec![],
            elo_chart: String::new(),
        };
        let html = template.render().unwrap();
        // Askama should escape HTML special characters
        assert!(!html.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn test_elo_chart_handles_equal_elos() {
        // Edge case: all Elo values are the same
        let history = vec![
            EloPoint {
                elo: 1500,
                date: "2025-01-01".into(),
            },
            EloPoint {
                elo: 1500,
                date: "2025-01-02".into(),
            },
            EloPoint {
                elo: 1500,
                date: "2025-01-03".into(),
            },
        ];
        let chart = BotExportTemplate::generate_elo_chart(&history);
        // Should not panic or produce NaN
        assert!(chart.contains("<svg"));
        assert!(!chart.contains("NaN"));
    }
}
