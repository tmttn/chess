//! Opening statistics API.
//!
//! Provides endpoints to retrieve chess opening statistics from played games.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chess_openings::{builtin::builtin_openings, OpeningDatabase};
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Looks up the ECO code for an opening by name.
///
/// Uses the built-in opening database to find the ECO code for a given opening name.
/// Returns an empty string if the opening is not found or has no ECO code.
fn lookup_eco(db: &OpeningDatabase, name: &str) -> String {
    db.search(name)
        .into_iter()
        .find(|o| o.name == name)
        .and_then(|o| o.eco.clone())
        .unwrap_or_default()
}

/// Statistics for a chess opening.
///
/// Contains aggregated data about how an opening has performed across all games
/// in the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpeningStats {
    /// ECO (Encyclopedia of Chess Openings) code, if available.
    pub eco: String,
    /// Name of the opening.
    pub name: String,
    /// Total number of games played with this opening.
    pub games_played: i32,
    /// Number of games won by white.
    pub white_wins: i32,
    /// Number of games won by black.
    pub black_wins: i32,
    /// Number of drawn games.
    pub draws: i32,
}

/// List all opening statistics.
///
/// Returns statistics for all openings that have been played, ordered by the
/// number of games played (descending).
///
/// # Endpoint
///
/// `GET /api/openings`
///
/// # Response
///
/// - `200 OK`: JSON array of opening statistics
/// - `500 Internal Server Error`: Database error
///
/// # Caching
///
/// Response is cached for 24 hours (opening data changes infrequently).
pub async fn list_openings(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    // Load the opening database for ECO code lookup
    let opening_db = OpeningDatabase::with_openings(builtin_openings());

    let stmt_result = conn.prepare(
        "SELECT
                g.opening_name as name,
                COUNT(*) as games,
                SUM(CASE WHEN g.result = '1-0' THEN 1 ELSE 0 END) as white_wins,
                SUM(CASE WHEN g.result = '0-1' THEN 1 ELSE 0 END) as black_wins,
                SUM(CASE WHEN g.result = '1/2-1/2' THEN 1 ELSE 0 END) as draws
             FROM games g
             WHERE g.opening_name IS NOT NULL
             GROUP BY g.opening_name
             ORDER BY games DESC",
    );

    let mut stmt = match stmt_result {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let query_result = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok((name, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    });

    let openings: Vec<OpeningStats> = match query_result {
        Ok(rows) => rows
            .filter_map(|r| r.ok())
            .map(|(name, games_played, white_wins, black_wins, draws)| {
                let eco = lookup_eco(&opening_db, &name);
                OpeningStats {
                    eco,
                    name,
                    games_played,
                    white_wins,
                    black_wins,
                    draws,
                }
            })
            .collect(),
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "public, max-age=86400")], // 24 hours
        Json(openings),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
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

    #[test]
    fn test_opening_stats_serialization() {
        let stats = OpeningStats {
            eco: "C50".to_string(),
            name: "Italian Game".to_string(),
            games_played: 100,
            white_wins: 40,
            black_wins: 35,
            draws: 25,
        };

        let json = serde_json::to_string(&stats).expect("Failed to serialize");
        assert!(json.contains("\"eco\":\"C50\""));
        assert!(json.contains("\"name\":\"Italian Game\""));
        assert!(json.contains("\"games_played\":100"));
        assert!(json.contains("\"white_wins\":40"));
        assert!(json.contains("\"black_wins\":35"));
        assert!(json.contains("\"draws\":25"));
    }

    #[test]
    fn test_opening_stats_default_eco() {
        let stats = OpeningStats {
            eco: String::new(),
            name: "Unknown Opening".to_string(),
            games_played: 5,
            white_wins: 2,
            black_wins: 2,
            draws: 1,
        };

        assert!(stats.eco.is_empty());
    }

    #[tokio::test]
    async fn test_list_openings_empty() {
        let state = test_state();
        let response = list_openings(State(state)).await.into_response();
        let (status, openings): (_, Vec<OpeningStats>) = extract_json(response).await;
        assert_eq!(status, StatusCode::OK);
        assert!(openings.is_empty());
    }

    #[tokio::test]
    async fn test_list_openings_with_data() {
        let state = test_state();

        // Insert test data
        {
            let conn = state.db.lock().unwrap();

            // Create bots first (required for foreign keys)
            conn.execute("INSERT INTO bots (name) VALUES ('bot1')", [])
                .unwrap();
            conn.execute("INSERT INTO bots (name) VALUES ('bot2')", [])
                .unwrap();

            // Create a match
            conn.execute(
                "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at)
                 VALUES ('match1', 'bot1', 'bot2', 5, '2025-01-21')",
                [],
            )
            .unwrap();

            // Create games with different openings and results
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, opening_name, result, started_at)
                 VALUES ('g1', 'match1', 1, 'Italian Game', '1-0', '2025-01-21')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, opening_name, result, started_at)
                 VALUES ('g2', 'match1', 2, 'Italian Game', '0-1', '2025-01-21')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, opening_name, result, started_at)
                 VALUES ('g3', 'match1', 3, 'Italian Game', '1/2-1/2', '2025-01-21')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, opening_name, result, started_at)
                 VALUES ('g4', 'match1', 4, 'Sicilian Defense', '1-0', '2025-01-21')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, opening_name, result, started_at)
                 VALUES ('g5', 'match1', 5, 'Sicilian Defense', '1-0', '2025-01-21')",
                [],
            )
            .unwrap();
        }

        let response = list_openings(State(state)).await.into_response();
        let (status, openings): (_, Vec<OpeningStats>) = extract_json(response).await;
        assert_eq!(status, StatusCode::OK);

        // Should have 2 openings
        assert_eq!(openings.len(), 2);

        // Italian Game has 3 games, Sicilian has 2, so Italian should be first
        assert_eq!(openings[0].name, "Italian Game");
        assert_eq!(openings[0].games_played, 3);
        assert_eq!(openings[0].white_wins, 1);
        assert_eq!(openings[0].black_wins, 1);
        assert_eq!(openings[0].draws, 1);

        assert_eq!(openings[1].name, "Sicilian Defense");
        assert_eq!(openings[1].games_played, 2);
        assert_eq!(openings[1].white_wins, 2);
        assert_eq!(openings[1].black_wins, 0);
        assert_eq!(openings[1].draws, 0);
    }

    #[tokio::test]
    async fn test_list_openings_excludes_null_names() {
        let state = test_state();

        // Insert test data with a game that has NULL opening_name
        {
            let conn = state.db.lock().unwrap();

            conn.execute("INSERT INTO bots (name) VALUES ('bot1')", [])
                .unwrap();
            conn.execute("INSERT INTO bots (name) VALUES ('bot2')", [])
                .unwrap();

            conn.execute(
                "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at)
                 VALUES ('match1', 'bot1', 'bot2', 2, '2025-01-21')",
                [],
            )
            .unwrap();

            // Game with opening name
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, opening_name, result, started_at)
                 VALUES ('g1', 'match1', 1, 'Italian Game', '1-0', '2025-01-21')",
                [],
            )
            .unwrap();

            // Game without opening name (NULL)
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, result, started_at)
                 VALUES ('g2', 'match1', 2, '0-1', '2025-01-21')",
                [],
            )
            .unwrap();
        }

        let response = list_openings(State(state)).await.into_response();
        let (status, openings): (_, Vec<OpeningStats>) = extract_json(response).await;
        assert_eq!(status, StatusCode::OK);

        // Should only have 1 opening (NULL opening_name is excluded)
        assert_eq!(openings.len(), 1);
        assert_eq!(openings[0].name, "Italian Game");
    }

    #[tokio::test]
    async fn test_list_openings_handles_unfinished_games() {
        let state = test_state();

        // Insert test data with games that have NULL result
        {
            let conn = state.db.lock().unwrap();

            conn.execute("INSERT INTO bots (name) VALUES ('bot1')", [])
                .unwrap();
            conn.execute("INSERT INTO bots (name) VALUES ('bot2')", [])
                .unwrap();

            conn.execute(
                "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at)
                 VALUES ('match1', 'bot1', 'bot2', 2, '2025-01-21')",
                [],
            )
            .unwrap();

            // Finished game
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, opening_name, result, started_at)
                 VALUES ('g1', 'match1', 1, 'Italian Game', '1-0', '2025-01-21')",
                [],
            )
            .unwrap();

            // Unfinished game (NULL result)
            conn.execute(
                "INSERT INTO games (id, match_id, game_number, opening_name, started_at)
                 VALUES ('g2', 'match1', 2, 'Italian Game', '2025-01-21')",
                [],
            )
            .unwrap();
        }

        let response = list_openings(State(state)).await.into_response();
        let (status, openings): (_, Vec<OpeningStats>) = extract_json(response).await;
        assert_eq!(status, StatusCode::OK);

        // Should have 1 opening with 2 games total
        assert_eq!(openings.len(), 1);
        assert_eq!(openings[0].name, "Italian Game");
        assert_eq!(openings[0].games_played, 2);
        // Only 1 finished game counted as white win
        assert_eq!(openings[0].white_wins, 1);
        assert_eq!(openings[0].black_wins, 0);
        assert_eq!(openings[0].draws, 0);
    }

    #[tokio::test]
    async fn test_list_openings_cache_header() {
        let state = test_state();
        let response = list_openings(State(state)).await.into_response();

        // Check Cache-Control header is set correctly
        let cache_control = response
            .headers()
            .get(header::CACHE_CONTROL)
            .expect("Cache-Control header should be present");
        assert_eq!(cache_control, "public, max-age=86400");
    }
}
