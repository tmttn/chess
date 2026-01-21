//! Statistics API endpoints.

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::AppState;

/// Record of head-to-head performance between two bots.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HeadToHeadRecord {
    /// Name of the bot playing white.
    pub white_bot: String,
    /// Name of the bot playing black.
    pub black_bot: String,
    /// Number of games won by white.
    pub white_wins: i32,
    /// Number of games won by black.
    pub black_wins: i32,
    /// Number of drawn games.
    pub draws: i32,
    /// Total number of games played.
    pub games: i32,
}

/// Matrix of head-to-head records between all bots.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HeadToHeadMatrix {
    /// List of all bot names, ordered by Elo rating.
    pub bots: Vec<String>,
    /// Head-to-head records for each pairing.
    pub records: Vec<HeadToHeadRecord>,
}

/// Get head-to-head statistics between all bots.
///
/// # Endpoint
///
/// `GET /api/stats/head-to-head`
///
/// # Response
///
/// - `200 OK`: JSON object containing bot list and head-to-head records
/// - `500 Internal Server Error`: Database error
pub async fn head_to_head(
    State(state): State<AppState>,
) -> Result<Json<HeadToHeadMatrix>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();

    // Get all bot names ordered by Elo rating
    let mut stmt = conn
        .prepare("SELECT name FROM bots ORDER BY elo_rating DESC")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let bots: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    // Get head-to-head records from completed matches
    let mut stmt = conn
        .prepare(
            "SELECT
                m.white_bot,
                m.black_bot,
                SUM(CASE WHEN g.result = '1-0' THEN 1 ELSE 0 END) as white_wins,
                SUM(CASE WHEN g.result = '0-1' THEN 1 ELSE 0 END) as black_wins,
                SUM(CASE WHEN g.result = '1/2-1/2' THEN 1 ELSE 0 END) as draws,
                COUNT(*) as games
             FROM matches m
             JOIN games g ON g.match_id = m.id
             WHERE m.status = 'completed'
             GROUP BY m.white_bot, m.black_bot",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let records: Vec<HeadToHeadRecord> = stmt
        .query_map([], |row| {
            Ok(HeadToHeadRecord {
                white_bot: row.get(0)?,
                black_bot: row.get(1)?,
                white_wins: row.get(2)?,
                black_wins: row.get(3)?,
                draws: row.get(4)?,
                games: row.get(5)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(HeadToHeadMatrix { bots, records }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::ws;

    fn test_state() -> AppState {
        let db = init_db(":memory:").expect("Failed to init test db");
        let ws_broadcast = ws::create_broadcast();
        AppState {
            db,
            ws_broadcast,
            engine_pool: None,
        }
    }

    fn setup_bots(state: &AppState) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO bots (name, elo_rating) VALUES ('stockfish', 2000)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO bots (name, elo_rating) VALUES ('komodo', 1800)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO bots (name, elo_rating) VALUES ('leela', 1600)",
            [],
        )
        .unwrap();
    }

    fn insert_match(state: &AppState, id: &str, white: &str, black: &str, status: &str) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO matches (id, white_bot, black_bot, games_total, started_at, status)
             VALUES (?1, ?2, ?3, 10, '2025-01-21', ?4)",
            [id, white, black, status],
        )
        .unwrap();
    }

    fn insert_game(state: &AppState, id: &str, match_id: &str, game_number: i32, result: &str) {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO games (id, match_id, game_number, result, started_at)
             VALUES (?1, ?2, ?3, ?4, '2025-01-21')",
            rusqlite::params![id, match_id, game_number, result],
        )
        .unwrap();
    }

    #[test]
    fn test_head_to_head_record_serialization() {
        let record = HeadToHeadRecord {
            white_bot: "stockfish".to_string(),
            black_bot: "komodo".to_string(),
            white_wins: 5,
            black_wins: 3,
            draws: 2,
            games: 10,
        };

        let json = serde_json::to_string(&record).expect("Failed to serialize");
        assert!(json.contains("\"white_bot\":\"stockfish\""));
        assert!(json.contains("\"black_bot\":\"komodo\""));
        assert!(json.contains("\"white_wins\":5"));
        assert!(json.contains("\"black_wins\":3"));
        assert!(json.contains("\"draws\":2"));
        assert!(json.contains("\"games\":10"));
    }

    #[test]
    fn test_head_to_head_matrix_serialization() {
        let matrix = HeadToHeadMatrix {
            bots: vec!["stockfish".to_string(), "komodo".to_string()],
            records: vec![HeadToHeadRecord {
                white_bot: "stockfish".to_string(),
                black_bot: "komodo".to_string(),
                white_wins: 5,
                black_wins: 3,
                draws: 2,
                games: 10,
            }],
        };

        let json = serde_json::to_string(&matrix).expect("Failed to serialize");
        assert!(json.contains("\"bots\":[\"stockfish\",\"komodo\"]"));
        assert!(json.contains("\"records\""));
    }

    #[tokio::test]
    async fn test_head_to_head_empty_database() {
        let state = test_state();
        let result = head_to_head(State(state)).await;
        assert!(result.is_ok());

        let Json(matrix) = result.unwrap();
        assert!(matrix.bots.is_empty());
        assert!(matrix.records.is_empty());
    }

    #[tokio::test]
    async fn test_head_to_head_bots_ordered_by_elo() {
        let state = test_state();
        setup_bots(&state);

        let result = head_to_head(State(state)).await;
        assert!(result.is_ok());

        let Json(matrix) = result.unwrap();
        assert_eq!(matrix.bots.len(), 3);
        // Ordered by Elo rating descending
        assert_eq!(matrix.bots[0], "stockfish");
        assert_eq!(matrix.bots[1], "komodo");
        assert_eq!(matrix.bots[2], "leela");
    }

    #[tokio::test]
    async fn test_head_to_head_no_completed_matches() {
        let state = test_state();
        setup_bots(&state);

        // Insert a pending match
        insert_match(&state, "match1", "stockfish", "komodo", "pending");

        let result = head_to_head(State(state)).await;
        assert!(result.is_ok());

        let Json(matrix) = result.unwrap();
        assert_eq!(matrix.bots.len(), 3);
        // No records since match is not completed
        assert!(matrix.records.is_empty());
    }

    #[tokio::test]
    async fn test_head_to_head_with_completed_games() {
        let state = test_state();
        setup_bots(&state);

        // Insert a completed match
        insert_match(&state, "match1", "stockfish", "komodo", "completed");

        // Insert games: 2 white wins, 1 black win, 1 draw
        insert_game(&state, "game1", "match1", 1, "1-0");
        insert_game(&state, "game2", "match1", 2, "1-0");
        insert_game(&state, "game3", "match1", 3, "0-1");
        insert_game(&state, "game4", "match1", 4, "1/2-1/2");

        let result = head_to_head(State(state)).await;
        assert!(result.is_ok());

        let Json(matrix) = result.unwrap();
        assert_eq!(matrix.records.len(), 1);

        let record = &matrix.records[0];
        assert_eq!(record.white_bot, "stockfish");
        assert_eq!(record.black_bot, "komodo");
        assert_eq!(record.white_wins, 2);
        assert_eq!(record.black_wins, 1);
        assert_eq!(record.draws, 1);
        assert_eq!(record.games, 4);
    }

    #[tokio::test]
    async fn test_head_to_head_multiple_pairings() {
        let state = test_state();
        setup_bots(&state);

        // Match 1: stockfish vs komodo
        insert_match(&state, "match1", "stockfish", "komodo", "completed");
        insert_game(&state, "game1", "match1", 1, "1-0");
        insert_game(&state, "game2", "match1", 2, "0-1");

        // Match 2: stockfish vs leela
        insert_match(&state, "match2", "stockfish", "leela", "completed");
        insert_game(&state, "game3", "match2", 1, "1-0");
        insert_game(&state, "game4", "match2", 2, "1-0");

        // Match 3: komodo vs leela
        insert_match(&state, "match3", "komodo", "leela", "completed");
        insert_game(&state, "game5", "match3", 1, "1/2-1/2");
        insert_game(&state, "game6", "match3", 2, "1/2-1/2");

        let result = head_to_head(State(state)).await;
        assert!(result.is_ok());

        let Json(matrix) = result.unwrap();
        assert_eq!(matrix.records.len(), 3);

        // Find stockfish vs leela record
        let sf_leela = matrix
            .records
            .iter()
            .find(|r| r.white_bot == "stockfish" && r.black_bot == "leela")
            .expect("Should find stockfish vs leela record");
        assert_eq!(sf_leela.white_wins, 2);
        assert_eq!(sf_leela.black_wins, 0);
        assert_eq!(sf_leela.draws, 0);
        assert_eq!(sf_leela.games, 2);

        // Find komodo vs leela record
        let ko_leela = matrix
            .records
            .iter()
            .find(|r| r.white_bot == "komodo" && r.black_bot == "leela")
            .expect("Should find komodo vs leela record");
        assert_eq!(ko_leela.white_wins, 0);
        assert_eq!(ko_leela.black_wins, 0);
        assert_eq!(ko_leela.draws, 2);
        assert_eq!(ko_leela.games, 2);
    }

    #[tokio::test]
    async fn test_head_to_head_aggregates_multiple_matches() {
        let state = test_state();
        setup_bots(&state);

        // Two matches between same bots
        insert_match(&state, "match1", "stockfish", "komodo", "completed");
        insert_game(&state, "game1", "match1", 1, "1-0");
        insert_game(&state, "game2", "match1", 2, "1-0");

        insert_match(&state, "match2", "stockfish", "komodo", "completed");
        insert_game(&state, "game3", "match2", 1, "0-1");
        insert_game(&state, "game4", "match2", 2, "1/2-1/2");

        let result = head_to_head(State(state)).await;
        assert!(result.is_ok());

        let Json(matrix) = result.unwrap();
        // Should have one aggregated record since white_bot/black_bot is the same
        assert_eq!(matrix.records.len(), 1);

        let record = &matrix.records[0];
        assert_eq!(record.white_wins, 2);
        assert_eq!(record.black_wins, 1);
        assert_eq!(record.draws, 1);
        assert_eq!(record.games, 4);
    }

    #[tokio::test]
    async fn test_head_to_head_ignores_running_matches() {
        let state = test_state();
        setup_bots(&state);

        // Running match should be ignored
        insert_match(&state, "match1", "stockfish", "komodo", "running");
        insert_game(&state, "game1", "match1", 1, "1-0");

        // Completed match should be included
        insert_match(&state, "match2", "stockfish", "leela", "completed");
        insert_game(&state, "game2", "match2", 1, "1-0");

        let result = head_to_head(State(state)).await;
        assert!(result.is_ok());

        let Json(matrix) = result.unwrap();
        assert_eq!(matrix.records.len(), 1);
        assert_eq!(matrix.records[0].black_bot, "leela");
    }
}
