//! Match API handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::{Game, Match, Move};
use crate::repo::{BotRepo, MatchFilter, MatchRepo};
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

/// Request body for creating a new match.
#[derive(Debug, Deserialize)]
pub struct CreateMatchRequest {
    /// Name of the bot playing white.
    pub white_bot: String,
    /// Name of the bot playing black.
    pub black_bot: String,
    /// Total number of games in the match.
    pub games: i32,
    /// Move time in milliseconds (optional, defaults to 1000).
    pub movetime_ms: Option<i32>,
    /// Opening ID to use (optional).
    pub opening_id: Option<String>,
}

/// Create a new match.
///
/// # Endpoint
///
/// `POST /api/matches`
///
/// # Request Body
///
/// JSON object with:
/// - `white_bot`: Name of the bot playing white
/// - `black_bot`: Name of the bot playing black
/// - `games`: Total number of games
/// - `movetime_ms`: Move time in milliseconds (optional, defaults to 1000)
/// - `opening_id`: Opening ID to use (optional)
///
/// # Response
///
/// - `200 OK`: JSON match object with the created match
/// - `500 Internal Server Error`: Database error
pub async fn create_match(
    State(state): State<AppState>,
    Json(req): Json<CreateMatchRequest>,
) -> Result<Json<Match>, StatusCode> {
    // Validate request
    if req.white_bot.is_empty() || req.black_bot.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if req.white_bot == req.black_bot {
        return Err(StatusCode::BAD_REQUEST);
    }
    if req.games <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Some(movetime) = req.movetime_ms {
        if movetime <= 0 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let match_repo = MatchRepo::new(state.db.clone());
    let bot_repo = BotRepo::new(state.db.clone());

    // Ensure bots exist (creates them if they don't)
    bot_repo
        .ensure(&req.white_bot)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    bot_repo
        .ensure(&req.black_bot)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let id = match_repo
        .create(
            &req.white_bot,
            &req.black_bot,
            req.games,
            req.movetime_ms.unwrap_or(1000),
            req.opening_id.as_deref(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let match_info = match_repo
        .get(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(match_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::ws;
    use bot_arena::config::ArenaConfig;
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

    #[tokio::test]
    async fn test_create_match_basic() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "bot_alpha".to_string(),
            black_bot: "bot_beta".to_string(),
            games: 10,
            movetime_ms: None,
            opening_id: None,
        };

        let result = create_match(State(state.clone()), Json(req)).await;
        assert!(result.is_ok());

        let Json(created_match) = result.unwrap();
        assert_eq!(created_match.white_bot, "bot_alpha");
        assert_eq!(created_match.black_bot, "bot_beta");
        assert_eq!(created_match.games_total, 10);
        assert_eq!(created_match.movetime_ms, 1000); // Default value
        assert_eq!(created_match.status, "pending");
        assert!(created_match.opening_id.is_none());

        // Verify bots were created
        let conn = state.db.lock().unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM bots", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_create_match_with_options() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "stockfish".to_string(),
            black_bot: "komodo".to_string(),
            games: 20,
            movetime_ms: Some(2000),
            opening_id: Some("sicilian".to_string()),
        };

        let result = create_match(State(state), Json(req)).await;
        assert!(result.is_ok());

        let Json(created_match) = result.unwrap();
        assert_eq!(created_match.white_bot, "stockfish");
        assert_eq!(created_match.black_bot, "komodo");
        assert_eq!(created_match.games_total, 20);
        assert_eq!(created_match.movetime_ms, 2000);
        assert_eq!(created_match.opening_id, Some("sicilian".to_string()));
    }

    #[tokio::test]
    async fn test_create_match_with_existing_bots() {
        let state = test_state();
        setup_test_data(&state); // Creates stockfish, komodo, leela

        let req = CreateMatchRequest {
            white_bot: "stockfish".to_string(),
            black_bot: "komodo".to_string(),
            games: 5,
            movetime_ms: None,
            opening_id: None,
        };

        let result = create_match(State(state.clone()), Json(req)).await;
        assert!(result.is_ok());

        // Verify no duplicate bots were created
        let conn = state.db.lock().unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM bots", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3); // Still only 3 bots
    }

    #[tokio::test]
    async fn test_create_match_empty_bot_name() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "".to_string(),
            black_bot: "bot2".to_string(),
            games: 10,
            movetime_ms: None,
            opening_id: None,
        };

        let result = create_match(State(state), Json(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_match_empty_black_bot_name() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "bot1".to_string(),
            black_bot: "".to_string(),
            games: 10,
            movetime_ms: None,
            opening_id: None,
        };

        let result = create_match(State(state), Json(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_match_same_bot() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "bot1".to_string(),
            black_bot: "bot1".to_string(),
            games: 10,
            movetime_ms: None,
            opening_id: None,
        };

        let result = create_match(State(state), Json(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_match_invalid_games_zero() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "bot1".to_string(),
            black_bot: "bot2".to_string(),
            games: 0,
            movetime_ms: None,
            opening_id: None,
        };

        let result = create_match(State(state), Json(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_match_invalid_games_negative() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "bot1".to_string(),
            black_bot: "bot2".to_string(),
            games: -5,
            movetime_ms: None,
            opening_id: None,
        };

        let result = create_match(State(state), Json(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_match_invalid_movetime_zero() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "bot1".to_string(),
            black_bot: "bot2".to_string(),
            games: 10,
            movetime_ms: Some(0),
            opening_id: None,
        };

        let result = create_match(State(state), Json(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_match_invalid_movetime_negative() {
        let state = test_state();

        let req = CreateMatchRequest {
            white_bot: "bot1".to_string(),
            black_bot: "bot2".to_string(),
            games: 10,
            movetime_ms: Some(-100),
            opening_id: None,
        };

        let result = create_match(State(state), Json(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }
}
