//! Bot API handlers.

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};

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
///
/// # Caching
///
/// Response is cached for 60 seconds (bot data may change with matches).
pub async fn list_bots(State(state): State<AppState>) -> impl IntoResponse {
    let repo = BotRepo::new(state.db.clone());
    match repo.list() {
        Ok(bots) => (
            StatusCode::OK,
            [(header::CACHE_CONTROL, "public, max-age=60")], // 1 minute
            Json(bots),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Get a bot profile by name, including Elo history.
///
/// # Endpoint
///
/// `GET /api/bots/:name`
///
/// # Response
///
/// - `200 OK`: JSON bot profile object with Elo history
/// - `404 Not Found`: Bot with given name doesn't exist
/// - `500 Internal Server Error`: Database error
///
/// # Caching
///
/// Response is cached for 60 seconds (bot data may change with matches).
pub async fn get_bot(State(state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    let repo = BotRepo::new(state.db.clone());
    match repo.get_profile(&name) {
        Ok(Some(profile)) => (
            StatusCode::OK,
            [(header::CACHE_CONTROL, "public, max-age=60")], // 1 minute
            Json(profile),
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::models::{Bot, BotProfile};
    use crate::ws;
    use axum::body::to_bytes;
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

    /// Helper to extract response body as JSON
    async fn extract_json<T: serde::de::DeserializeOwned>(
        response: axum::response::Response,
    ) -> (StatusCode, T) {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: T = serde_json::from_slice(&body).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn test_list_bots_empty() {
        let state = test_state();
        let response = list_bots(State(state)).await.into_response();
        let (status, bots): (_, Vec<Bot>) = extract_json(response).await;
        assert_eq!(status, StatusCode::OK);
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

        let response = list_bots(State(state)).await.into_response();
        let (status, bots): (_, Vec<Bot>) = extract_json(response).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(bots.len(), 2);
        // Should be ordered by Elo descending
        assert_eq!(bots[0].name, "bot1");
        assert_eq!(bots[1].name, "bot2");
    }

    #[tokio::test]
    async fn test_get_bot_profile_found() {
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

        let response = get_bot(State(state), Path("stockfish".to_string()))
            .await
            .into_response();
        let (status, profile): (_, BotProfile) = extract_json(response).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(profile.name, "stockfish");
        assert_eq!(profile.elo_rating, 2000);
        assert!(profile.elo_history.is_empty());
    }

    #[tokio::test]
    async fn test_get_bot_profile_not_found() {
        let state = test_state();
        let response = get_bot(State(state), Path("nonexistent".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_bot_profile_with_elo_history() {
        let state = test_state();

        // Insert test data with Elo history
        {
            let conn = state.db.lock().unwrap();
            conn.execute(
                "INSERT INTO bots (name, elo_rating) VALUES ('stockfish', 1600)",
                [],
            )
            .unwrap();
            // Add some Elo history entries
            conn.execute(
                "INSERT INTO elo_history (bot_name, elo_rating, recorded_at) VALUES ('stockfish', 1500, '2025-01-01T10:00:00')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO elo_history (bot_name, elo_rating, recorded_at) VALUES ('stockfish', 1550, '2025-01-02T10:00:00')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO elo_history (bot_name, elo_rating, recorded_at) VALUES ('stockfish', 1600, '2025-01-03T10:00:00')",
                [],
            )
            .unwrap();
        }

        let response = get_bot(State(state), Path("stockfish".to_string()))
            .await
            .into_response();
        let (status, profile): (_, BotProfile) = extract_json(response).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(profile.name, "stockfish");
        assert_eq!(profile.elo_rating, 1600);
        assert_eq!(profile.elo_history.len(), 3);
        // Verify history is ordered by timestamp ascending
        assert_eq!(profile.elo_history[0].elo, 1500);
        assert_eq!(profile.elo_history[0].timestamp, "2025-01-01T10:00:00");
        assert_eq!(profile.elo_history[1].elo, 1550);
        assert_eq!(profile.elo_history[2].elo, 1600);
    }

    #[tokio::test]
    async fn test_list_bots_cache_header() {
        let state = test_state();
        let response = list_bots(State(state)).await.into_response();

        // Check Cache-Control header is set correctly
        let cache_control = response
            .headers()
            .get(header::CACHE_CONTROL)
            .expect("Cache-Control header should be present");
        assert_eq!(cache_control, "public, max-age=60");
    }

    #[tokio::test]
    async fn test_get_bot_cache_header() {
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

        let response = get_bot(State(state), Path("stockfish".to_string()))
            .await
            .into_response();

        // Check Cache-Control header is set correctly
        let cache_control = response
            .headers()
            .get(header::CACHE_CONTROL)
            .expect("Cache-Control header should be present");
        assert_eq!(cache_control, "public, max-age=60");
    }
}
