//! Match API handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::{Game, Match, Move};
use crate::repo::{MatchFilter, MatchRepo};
use crate::AppState;

/// Query parameters for listing matches.
#[derive(Debug, Deserialize)]
pub struct ListMatchesQuery {
    /// Filter by bot name (matches where bot is white or black).
    pub bot: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<i32>,
    /// Number of results to skip.
    pub offset: Option<i32>,
}

/// List matches with optional filtering.
///
/// # Endpoint
///
/// `GET /api/matches`
///
/// # Query Parameters
///
/// - `bot`: Filter by bot name (optional)
/// - `limit`: Maximum results (default: 20)
/// - `offset`: Skip results (default: 0)
///
/// # Response
///
/// - `200 OK`: JSON array of match objects
/// - `500 Internal Server Error`: Database error
pub async fn list_matches(
    State(state): State<AppState>,
    Query(query): Query<ListMatchesQuery>,
) -> Result<Json<Vec<Match>>, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());
    let filter = MatchFilter {
        bot: query.bot,
        limit: query.limit.unwrap_or(20),
        offset: query.offset.unwrap_or(0),
    };

    repo.list(filter)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Match with full game details.
#[derive(Debug, Clone, Serialize)]
pub struct MatchDetail {
    /// The match information.
    #[serde(flatten)]
    pub match_info: Match,
    /// All games in this match.
    pub games: Vec<Game>,
}

/// Get match detail with all games.
///
/// # Endpoint
///
/// `GET /api/matches/:id` (detail endpoint)
///
/// # Response
///
/// - `200 OK`: JSON match detail object with games
/// - `404 Not Found`: Match with given ID doesn't exist
/// - `500 Internal Server Error`: Database error
pub async fn get_match_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MatchDetail>, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());

    let match_info = repo
        .get(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let games = repo
        .get_games(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MatchDetail { match_info, games }))
}

/// Get all moves for a game.
///
/// # Endpoint
///
/// `GET /api/games/:id/moves`
///
/// # Response
///
/// - `200 OK`: JSON array of move objects
/// - `500 Internal Server Error`: Database error
pub async fn get_game_moves(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> Result<Json<Vec<Move>>, StatusCode> {
    let repo = MatchRepo::new(state.db.clone());
    repo.get_moves(&game_id)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::ws;

    fn test_state() -> AppState {
        let db = init_db(":memory:").expect("Failed to init test db");
        let ws_broadcast = ws::create_broadcast();
        AppState { db, ws_broadcast }
    }

    fn setup_test_data(state: &AppState) {
        let conn = state.db.lock().unwrap();

        // Insert test bots
        conn.execute("INSERT INTO bots (name) VALUES ('stockfish')", [])
            .unwrap();
        conn.execute("INSERT INTO bots (name) VALUES ('komodo')", [])
            .unwrap();
        conn.execute("INSERT INTO bots (name) VALUES ('leela')", [])
            .unwrap();
    }

    fn insert_match(state: &AppState, id: &str, white: &str, black: &str, started_at: &str) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at, status)
             VALUES (?1, ?2, ?3, 10, ?4, 'pending')",
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

    fn insert_move(state: &AppState, game_id: &str, ply: i32, uci: &str, fen: &str) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO moves (game_id, ply, uci, fen_after)
             VALUES (?1, ?2, ?3, ?4)",
            [game_id, &ply.to_string(), uci, fen],
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_list_matches_empty() {
        let state = test_state();
        let query = ListMatchesQuery {
            bot: None,
            limit: None,
            offset: None,
        };
        let result = list_matches(State(state), Query(query)).await;
        assert!(result.is_ok());
        let Json(matches) = result.unwrap();
        assert!(matches.is_empty());
    }

    #[tokio::test]
    async fn test_list_matches_with_data() {
        let state = test_state();
        setup_test_data(&state);

        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_match(
            &state,
            "match2",
            "stockfish",
            "leela",
            "2025-01-21T11:00:00",
        );

        let query = ListMatchesQuery {
            bot: None,
            limit: None,
            offset: None,
        };
        let result = list_matches(State(state), Query(query)).await;
        assert!(result.is_ok());
        let Json(matches) = result.unwrap();
        assert_eq!(matches.len(), 2);
        // Most recent first
        assert_eq!(matches[0].id, "match2");
        assert_eq!(matches[1].id, "match1");
    }

    #[tokio::test]
    async fn test_list_matches_with_bot_filter() {
        let state = test_state();
        setup_test_data(&state);

        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_match(&state, "match2", "komodo", "leela", "2025-01-21T11:00:00");

        let query = ListMatchesQuery {
            bot: Some("stockfish".to_string()),
            limit: None,
            offset: None,
        };
        let result = list_matches(State(state), Query(query)).await;
        assert!(result.is_ok());
        let Json(matches) = result.unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "match1");
    }

    #[tokio::test]
    async fn test_list_matches_with_limit_and_offset() {
        let state = test_state();
        setup_test_data(&state);

        insert_match(
            &state,
            "match1",
            "stockfish",
            "komodo",
            "2025-01-21T10:00:00",
        );
        insert_match(
            &state,
            "match2",
            "stockfish",
            "leela",
            "2025-01-21T11:00:00",
        );
        insert_match(&state, "match3", "komodo", "leela", "2025-01-21T12:00:00");

        let query = ListMatchesQuery {
            bot: None,
            limit: Some(1),
            offset: Some(1),
        };
        let result = list_matches(State(state), Query(query)).await;
        assert!(result.is_ok());
        let Json(matches) = result.unwrap();
        assert_eq!(matches.len(), 1);
        // Skipped match3, got match2
        assert_eq!(matches[0].id, "match2");
    }

    #[tokio::test]
    async fn test_get_match_detail_with_games() {
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

        let result = get_match_detail(State(state), Path("match1".to_string())).await;
        assert!(result.is_ok());
        let Json(detail) = result.unwrap();
        assert_eq!(detail.match_info.id, "match1");
        assert_eq!(detail.games.len(), 2);
        assert_eq!(detail.games[0].game_number, 1);
        assert_eq!(detail.games[1].game_number, 2);
    }

    #[tokio::test]
    async fn test_get_match_detail_not_found() {
        let state = test_state();
        let result = get_match_detail(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_game_moves() {
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

        let fen1 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let fen2 = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";

        insert_move(&state, "game1", 1, "e2e4", fen1);
        insert_move(&state, "game1", 2, "e7e5", fen2);

        let result = get_game_moves(State(state), Path("game1".to_string())).await;
        assert!(result.is_ok());
        let Json(moves) = result.unwrap();
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].ply, 1);
        assert_eq!(moves[0].uci, "e2e4");
        assert_eq!(moves[1].ply, 2);
        assert_eq!(moves[1].uci, "e7e5");
    }

    #[tokio::test]
    async fn test_get_game_moves_empty() {
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

        let result = get_game_moves(State(state), Path("game1".to_string())).await;
        assert!(result.is_ok());
        let Json(moves) = result.unwrap();
        assert!(moves.is_empty());
    }
}
