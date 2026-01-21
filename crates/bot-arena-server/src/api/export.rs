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
    BoardTemplate, GameExportTemplate, GameSummary, MatchExportTemplate,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::ws;
    use http_body_util::BodyExt;

    fn test_state() -> AppState {
        let db = init_db(":memory:").expect("Failed to init test db");
        let ws_broadcast = ws::create_broadcast();
        AppState {
            db,
            ws_broadcast,
            engine_pool: None,
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
}
