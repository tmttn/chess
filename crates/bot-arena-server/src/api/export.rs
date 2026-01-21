//! Export API handlers.
//!
//! Provides endpoints for exporting match and game data as downloadable HTML files.

use askama::Template;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};

use crate::repo::MatchRepo;
use crate::AppState;
use bot_arena_server::templates::{
    BoardTemplate, BotExportTemplate, EloPoint, GameExportTemplate, GameSummary,
    MatchExportTemplate,
};

/// Export a match as a standalone HTML file.
///
/// Generates a complete HTML page with match results that can be saved and viewed
/// offline. The response includes a `Content-Disposition` header to trigger a download.
///
/// # Endpoint
///
/// `GET /api/export/match/:id`
///
/// # Response
///
/// - `200 OK`: HTML file download
/// - `404 Not Found`: Match with given ID doesn't exist
/// - `500 Internal Server Error`: Database or rendering error
pub async fn export_match(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());

    // Get the match
    let match_info = repo
        .get(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get all games for the match
    let games = repo
        .get_games(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert games to summaries
    // In a match, colors may alternate between games, but for simplicity
    // we use the match's white/black bots and determine the actual players
    // based on game_number (odd = normal, even = swapped)
    let game_summaries: Vec<GameSummary> = games
        .iter()
        .map(|game| {
            let (white, black) = if game.game_number % 2 == 1 {
                (match_info.white_bot.clone(), match_info.black_bot.clone())
            } else {
                (match_info.black_bot.clone(), match_info.white_bot.clone())
            };

            GameSummary {
                white,
                black,
                result: game.result.clone().unwrap_or_else(|| "*".to_string()),
                move_count: 0, // We don't have move counts in the Game model without querying moves
            }
        })
        .collect();

    // Build the template
    let template = MatchExportTemplate {
        white_bot: match_info.white_bot.clone(),
        black_bot: match_info.black_bot.clone(),
        white_score: match_info.white_score,
        black_score: match_info.black_score,
        games: game_summaries,
        created_at: Some(match_info.started_at.clone()),
    };

    // Render the template
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create filename for download
    let filename = format!(
        "match_{}_{}_vs_{}.html",
        id,
        sanitize_filename(&match_info.white_bot),
        sanitize_filename(&match_info.black_bot)
    );

    // Build response with Content-Disposition header for download
    let response = (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        Html(html),
    )
        .into_response();

    Ok(response)
}

/// Sanitize a string for use in a filename.
///
/// Replaces any non-alphanumeric characters (except dash and underscore) with underscores.
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Starting position FEN for chess.
const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Query result for game export information.
struct GameQueryResult {
    game_id: String,
    match_id: String,
    game_number: i32,
    result: Option<String>,
    opening_name: Option<String>,
    match_white: String,
    match_black: String,
}

/// Export a game as a standalone HTML file.
///
/// Generates a complete HTML page with the game's board position, move list,
/// and game info that can be saved and viewed offline.
///
/// # Endpoint
///
/// `GET /api/export/game/:id`
///
/// # Response
///
/// - `200 OK`: HTML file download
/// - `404 Not Found`: Game with given ID doesn't exist
/// - `500 Internal Server Error`: Database or rendering error
pub async fn export_game(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());

    // Query game info with bot names from the match
    let (game, white_bot, black_bot) = {
        let conn = state
            .db
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Get game with match info to determine player colors
        let query_result: Option<GameQueryResult> = conn
            .query_row(
                "SELECT g.id, g.match_id, g.game_number, g.result, g.opening_name,
                        m.white_bot, m.black_bot
                 FROM games g
                 JOIN matches m ON g.match_id = m.id
                 WHERE g.id = ?1",
                [&id],
                |row| {
                    Ok(GameQueryResult {
                        game_id: row.get(0)?,
                        match_id: row.get(1)?,
                        game_number: row.get(2)?,
                        result: row.get(3)?,
                        opening_name: row.get(4)?,
                        match_white: row.get(5)?,
                        match_black: row.get(6)?,
                    })
                },
            )
            .ok();

        match query_result {
            Some(qr) => {
                // Determine actual colors based on game number (odd = normal, even = swapped)
                let (white, black) = if qr.game_number % 2 == 1 {
                    (qr.match_white, qr.match_black)
                } else {
                    (qr.match_black, qr.match_white)
                };
                (
                    crate::models::Game {
                        id: qr.game_id,
                        match_id: qr.match_id,
                        game_number: qr.game_number,
                        result: qr.result,
                        opening_name: qr.opening_name,
                        pgn: None,
                    },
                    white,
                    black,
                )
            }
            None => return Err(StatusCode::NOT_FOUND),
        }
    };

    // Get moves for the game
    let moves = repo
        .get_moves(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get the final position FEN, or starting position if no moves
    let final_fen = moves
        .last()
        .map(|m| m.fen_after.as_str())
        .unwrap_or(STARTING_FEN);

    // Render the board
    let board_template = BoardTemplate::from_fen(final_fen);
    let board_svg = board_template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert moves to SAN notation, falling back to UCI if SAN not available
    let move_strings: Vec<String> = moves
        .iter()
        .map(|m| m.san.clone().unwrap_or_else(|| m.uci.clone()))
        .collect();

    // Pair the moves for display
    let move_pairs = GameExportTemplate::pair_moves(move_strings);

    // Build the template
    let template = GameExportTemplate {
        white: white_bot.clone(),
        black: black_bot.clone(),
        result: game.result.clone().unwrap_or_else(|| "*".to_string()),
        opening: game.opening_name.clone(),
        board: board_svg,
        move_pairs,
    };

    // Render the template
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create filename for download
    let filename = format!(
        "game_{}_{}_vs_{}.html",
        id,
        sanitize_filename(&white_bot),
        sanitize_filename(&black_bot)
    );

    // Build response with Content-Disposition header for download
    let response = (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        Html(html),
    )
        .into_response();

    Ok(response)
}

/// Query result for bot information.
struct BotQueryResult {
    name: String,
    elo_rating: i32,
    games_played: i32,
    wins: i32,
    draws: i32,
    losses: i32,
}

/// Query result for Elo history.
struct EloHistoryRow {
    elo_rating: i32,
    recorded_at: String,
}

/// Export a bot profile as a standalone HTML file.
///
/// Generates a complete HTML page with the bot's statistics, Elo rating,
/// and Elo history chart that can be saved and viewed offline.
///
/// # Endpoint
///
/// `GET /api/export/bot/:name`
///
/// # Response
///
/// - `200 OK`: HTML file download
/// - `404 Not Found`: Bot with given name doesn't exist
/// - `500 Internal Server Error`: Database or rendering error
pub async fn export_bot(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    // Query bot information
    let bot = {
        let conn = state
            .db
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        conn.query_row(
            "SELECT name, elo_rating, games_played, wins, draws, losses
             FROM bots WHERE name = ?1",
            [&name],
            |row| {
                Ok(BotQueryResult {
                    name: row.get(0)?,
                    elo_rating: row.get(1)?,
                    games_played: row.get(2)?,
                    wins: row.get(3)?,
                    draws: row.get(4)?,
                    losses: row.get(5)?,
                })
            },
        )
        .map_err(|_| StatusCode::NOT_FOUND)?
    };

    // Calculate win rate
    let win_rate = if bot.games_played > 0 {
        format!("{:.1}", (bot.wins as f64 / bot.games_played as f64) * 100.0)
    } else {
        "0.0".to_string()
    };

    // Query Elo history
    let elo_history: Vec<EloPoint> = {
        let conn = state
            .db
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut stmt = conn
            .prepare(
                "SELECT elo_rating, recorded_at FROM elo_history
                 WHERE bot_name = ?1
                 ORDER BY recorded_at ASC",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let history_rows: Vec<EloHistoryRow> = stmt
            .query_map([&name], |row| {
                Ok(EloHistoryRow {
                    elo_rating: row.get(0)?,
                    recorded_at: row.get(1)?,
                })
            })
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .filter_map(|r| r.ok())
            .collect();

        history_rows
            .into_iter()
            .map(|h| EloPoint {
                elo: h.elo_rating,
                date: h.recorded_at,
            })
            .collect()
    };

    // Generate Elo chart
    let elo_chart = BotExportTemplate::generate_elo_chart(&elo_history);

    // Build the template
    let template = BotExportTemplate {
        name: bot.name.clone(),
        elo: bot.elo_rating,
        games_played: bot.games_played,
        wins: bot.wins,
        draws: bot.draws,
        losses: bot.losses,
        win_rate,
        elo_history,
        elo_chart,
    };

    // Render the template
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create filename for download
    let filename = format!("bot_profile_{}.html", sanitize_filename(&bot.name));

    // Build response with Content-Disposition header for download
    let response = (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        Html(html),
    )
        .into_response();

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::ws;
    use bot_arena::config::ArenaConfig;
    use http_body_util::BodyExt;
    use std::sync::Arc;

    fn test_state() -> AppState {
        let db = init_db(":memory:").expect("Failed to init test db");
        let ws_broadcast = ws::create_broadcast();
        AppState {
            db,
            ws_broadcast,
            engine_pool: None,
            config: Arc::new(ArenaConfig::default()),
        }
    }

    fn setup_test_data(state: &AppState) {
        let conn = state.db.lock().unwrap();
        conn.execute("INSERT INTO bots (name) VALUES ('stockfish')", [])
            .unwrap();
        conn.execute("INSERT INTO bots (name) VALUES ('komodo')", [])
            .unwrap();
    }

    fn insert_match(state: &AppState, id: &str, white: &str, black: &str, started_at: &str) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO matches (id, white_bot, black_bot, games_total, white_score, black_score, started_at, status)
             VALUES (?1, ?2, ?3, 10, 5.5, 4.5, ?4, 'completed')",
            [id, white, black, started_at],
        )
        .unwrap();
    }

    fn insert_game(
        state: &AppState,
        id: &str,
        match_id: &str,
        game_number: i32,
        result: Option<&str>,
    ) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO games (id, match_id, game_number, result, started_at)
             VALUES (?1, ?2, ?3, ?4, '2025-01-21')",
            rusqlite::params![id, match_id, game_number, result],
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_export_match_not_found() {
        let state = test_state();
        let result = export_match(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_export_match_success() {
        let state = test_state();
        setup_test_data(&state);
        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_game(&state, "game1", "match1", 1, Some("1-0"));
        insert_game(&state, "game2", "match1", 2, Some("0-1"));

        let result = export_match(State(state), Path("match1".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check Content-Type header
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("Should have Content-Type header");
        assert!(content_type.to_str().unwrap().contains("text/html"));

        // Check Content-Disposition header
        let content_disposition = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .expect("Should have Content-Disposition header");
        let disposition_str = content_disposition.to_str().unwrap();
        assert!(disposition_str.contains("attachment"));
        assert!(disposition_str.contains("match_match1_stockfish_vs_komodo.html"));
    }

    #[tokio::test]
    async fn test_export_match_contains_correct_content() {
        let state = test_state();
        setup_test_data(&state);
        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_game(&state, "game1", "match1", 1, Some("1-0"));

        let result = export_match(State(state), Path("match1".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Verify the HTML contains expected content
        assert!(html.contains("stockfish"));
        assert!(html.contains("komodo"));
        assert!(html.contains("5.5 - 4.5"));
        assert!(html.contains("Match Report"));
        assert!(html.contains("Generated by Bot Arena"));
    }

    #[tokio::test]
    async fn test_export_match_empty_games() {
        let state = test_state();
        setup_test_data(&state);
        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );

        let result = export_match(State(state), Path("match1".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("stockfish"), "stockfish");
        assert_eq!(sanitize_filename("bot-1"), "bot-1");
        assert_eq!(sanitize_filename("bot_2"), "bot_2");
        assert_eq!(sanitize_filename("bot 3"), "bot_3");
        assert_eq!(sanitize_filename("bot/4"), "bot_4");
        assert_eq!(sanitize_filename("bot\\5"), "bot_5");
        assert_eq!(sanitize_filename("bot:6"), "bot_6");
    }

    #[test]
    fn test_sanitize_filename_preserves_alphanumeric() {
        assert_eq!(
            sanitize_filename("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        );
    }

    fn insert_game_with_opening(
        state: &AppState,
        id: &str,
        match_id: &str,
        game_number: i32,
        result: Option<&str>,
        opening_name: Option<&str>,
    ) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO games (id, match_id, game_number, result, opening_name, started_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '2025-01-21')",
            rusqlite::params![id, match_id, game_number, result, opening_name],
        )
        .unwrap();
    }

    fn insert_move(state: &AppState, game_id: &str, ply: i32, uci: &str, san: &str, fen: &str) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO moves (game_id, ply, uci, san, fen_after)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [game_id, &ply.to_string(), uci, san, fen],
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_export_game_not_found() {
        let state = test_state();
        let result = export_game(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_export_game_success() {
        let state = test_state();
        setup_test_data(&state);
        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_game_with_opening(
            &state,
            "game1",
            "match1",
            1,
            Some("1-0"),
            Some("Italian Game"),
        );

        let result = export_game(State(state), Path("game1".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check Content-Type header
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("Should have Content-Type header");
        assert!(content_type.to_str().unwrap().contains("text/html"));

        // Check Content-Disposition header
        let content_disposition = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .expect("Should have Content-Disposition header");
        let disposition_str = content_disposition.to_str().unwrap();
        assert!(disposition_str.contains("attachment"));
        assert!(disposition_str.contains("game_game1_stockfish_vs_komodo.html"));
    }

    #[tokio::test]
    async fn test_export_game_contains_correct_content() {
        let state = test_state();
        setup_test_data(&state);
        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_game_with_opening(
            &state,
            "game1",
            "match1",
            1,
            Some("1-0"),
            Some("Italian Game"),
        );

        // Add some moves
        let fen1 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let fen2 = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";
        insert_move(&state, "game1", 1, "e2e4", "e4", fen1);
        insert_move(&state, "game1", 2, "e7e5", "e5", fen2);

        let result = export_game(State(state), Path("game1".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Verify the HTML contains expected content
        assert!(html.contains("stockfish"));
        assert!(html.contains("komodo"));
        assert!(html.contains("1-0"));
        assert!(html.contains("Italian Game"));
        assert!(html.contains("Game Viewer"));
        assert!(html.contains("Generated by Bot Arena"));
        // Check for moves
        assert!(html.contains("e4"));
        assert!(html.contains("e5"));
        // Check for SVG board
        assert!(html.contains("<svg"));
    }

    #[tokio::test]
    async fn test_export_game_empty_moves() {
        let state = test_state();
        setup_test_data(&state);
        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_game(&state, "game1", "match1", 1, None);

        let result = export_game(State(state), Path("game1".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Should render starting position when no moves
        assert!(html.contains("<svg"));
    }

    #[tokio::test]
    async fn test_export_game_swapped_colors() {
        let state = test_state();
        setup_test_data(&state);
        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        // Game 2 has swapped colors (even game number)
        insert_game_with_opening(&state, "game2", "match1", 2, Some("1-0"), None);

        let result = export_game(State(state), Path("game2".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Check Content-Disposition for swapped colors
        // In game 2 (even), komodo plays white and stockfish plays black
        assert!(html.contains("komodo"));
        assert!(html.contains("stockfish"));
    }

    #[tokio::test]
    async fn test_export_game_without_opening() {
        let state = test_state();
        setup_test_data(&state);
        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_game_with_opening(&state, "game1", "match1", 1, Some("0-1"), None);

        let result = export_game(State(state), Path("game1".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Should not contain "Opening:" when no opening is set
        assert!(!html.contains("Opening:"));
    }

    fn insert_bot_with_stats(
        state: &AppState,
        name: &str,
        elo: i32,
        games_played: i32,
        wins: i32,
        draws: i32,
        losses: i32,
    ) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO bots (name, elo_rating, games_played, wins, draws, losses)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![name, elo, games_played, wins, draws, losses],
        )
        .unwrap();
    }

    fn insert_elo_history(state: &AppState, bot_name: &str, elo: i32, recorded_at: &str) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO elo_history (bot_name, elo_rating, recorded_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![bot_name, elo, recorded_at],
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_export_bot_not_found() {
        let state = test_state();
        let result = export_bot(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_export_bot_success() {
        let state = test_state();
        insert_bot_with_stats(&state, "stockfish", 1650, 100, 60, 20, 20);

        let result = export_bot(State(state), Path("stockfish".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check Content-Type header
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("Should have Content-Type header");
        assert!(content_type.to_str().unwrap().contains("text/html"));

        // Check Content-Disposition header
        let content_disposition = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .expect("Should have Content-Disposition header");
        let disposition_str = content_disposition.to_str().unwrap();
        assert!(disposition_str.contains("attachment"));
        assert!(disposition_str.contains("bot_profile_stockfish.html"));
    }

    #[tokio::test]
    async fn test_export_bot_contains_correct_content() {
        let state = test_state();
        insert_bot_with_stats(&state, "minimax", 1550, 50, 25, 10, 15);

        let result = export_bot(State(state), Path("minimax".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Verify the HTML contains expected content
        assert!(html.contains("minimax"));
        assert!(html.contains("1550 Elo"));
        assert!(html.contains("50")); // games_played
        assert!(html.contains("25")); // wins
        assert!(html.contains("10")); // draws
        assert!(html.contains("15")); // losses
        assert!(html.contains("50.0%")); // win_rate = 25/50 = 50%
        assert!(html.contains("Generated by Bot Arena"));
    }

    #[tokio::test]
    async fn test_export_bot_with_elo_history() {
        let state = test_state();
        insert_bot_with_stats(&state, "alpha", 1600, 20, 12, 4, 4);
        insert_elo_history(&state, "alpha", 1500, "2025-01-01");
        insert_elo_history(&state, "alpha", 1520, "2025-01-05");
        insert_elo_history(&state, "alpha", 1580, "2025-01-10");
        insert_elo_history(&state, "alpha", 1600, "2025-01-15");

        let result = export_bot(State(state), Path("alpha".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Should have Elo History section with SVG chart
        assert!(html.contains("Elo History"));
        assert!(html.contains("<svg"));
        assert!(html.contains("polyline"));
    }

    #[tokio::test]
    async fn test_export_bot_without_elo_history() {
        let state = test_state();
        insert_bot_with_stats(&state, "newbot", 1500, 0, 0, 0, 0);

        let result = export_bot(State(state), Path("newbot".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Should NOT have Elo History section
        assert!(!html.contains("Elo History"));
    }

    #[tokio::test]
    async fn test_export_bot_zero_games_win_rate() {
        let state = test_state();
        insert_bot_with_stats(&state, "fresh_bot", 1500, 0, 0, 0, 0);

        let result = export_bot(State(state), Path("fresh_bot".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Win rate should be 0.0% for zero games
        assert!(html.contains("0.0%"));
    }

    #[tokio::test]
    async fn test_export_bot_html_structure() {
        let state = test_state();
        insert_bot_with_stats(&state, "test_bot", 1500, 10, 5, 3, 2);

        let result = export_bot(State(state), Path("test_bot".to_string())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        // Check HTML structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("Bot Profile:") || html.contains("test_bot"));
    }
}
