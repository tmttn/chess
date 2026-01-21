//! Bot API handlers.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::Bot;
use crate::repo::BotRepo;
use crate::AppState;

/// List all bots, ordered by Elo rating (descending).
///
/// # Endpoint
///
/// `GET /api/bots`
///
/// # Response
///
/// - `200 OK`: JSON array of bot objects
/// - `500 Internal Server Error`: Database error
pub async fn list_bots(State(state): State<AppState>) -> Result<Json<Vec<Bot>>, StatusCode> {
    let repo = BotRepo::new(state.db.clone());
    repo.list()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get a single bot by name.
///
/// # Endpoint
///
/// `GET /api/bots/:name`
///
/// # Response
///
/// - `200 OK`: JSON bot object
/// - `404 Not Found`: Bot with given name doesn't exist
/// - `500 Internal Server Error`: Database error
pub async fn get_bot(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Bot>, StatusCode> {
    let repo = BotRepo::new(state.db.clone());
    repo.get(&name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
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

    #[tokio::test]
    async fn test_list_bots_empty() {
        let state = test_state();
        let result = list_bots(State(state)).await;
        assert!(result.is_ok());
        let Json(bots) = result.unwrap();
        assert!(bots.is_empty());
    }

    #[tokio::test]
    async fn test_list_bots_with_data() {
        let state = test_state();

        // Insert test data
        {
            let conn = state.db.lock().unwrap();
            conn.execute(
                "INSERT INTO bots (name, elo_rating) VALUES ('bot1', 1600)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO bots (name, elo_rating) VALUES ('bot2', 1400)",
                [],
            )
            .unwrap();
        }

        let result = list_bots(State(state)).await;
        assert!(result.is_ok());
        let Json(bots) = result.unwrap();
        assert_eq!(bots.len(), 2);
        // Should be ordered by Elo descending
        assert_eq!(bots[0].name, "bot1");
        assert_eq!(bots[1].name, "bot2");
    }

    #[tokio::test]
    async fn test_get_bot_found() {
        let state = test_state();

        // Insert test data
        {
            let conn = state.db.lock().unwrap();
            conn.execute(
                "INSERT INTO bots (name, elo_rating) VALUES ('stockfish', 2000)",
                [],
            )
            .unwrap();
        }

        let result = get_bot(State(state), Path("stockfish".to_string())).await;
        assert!(result.is_ok());
        let Json(bot) = result.unwrap();
        assert_eq!(bot.name, "stockfish");
        assert_eq!(bot.elo_rating, 2000);
    }

    #[tokio::test]
    async fn test_get_bot_not_found() {
        let state = test_state();
        let result = get_bot(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }
}
