//! Analysis API endpoints.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Query parameters for analysis request.
#[derive(Debug, Deserialize)]
pub struct AnalysisQuery {
    /// Position in FEN notation.
    pub fen: String,
    /// Search depth (default: 20).
    #[serde(default = "default_depth")]
    pub depth: i32,
}

fn default_depth() -> i32 {
    20
}

/// Analysis response.
#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    /// The analyzed position.
    pub fen: String,
    /// Search depth reached.
    pub depth: i32,
    /// Score in centipawns (positive = white advantage).
    pub score_cp: Option<i32>,
    /// Mate score (positive = white mates in N).
    pub score_mate: Option<i32>,
    /// Best move in UCI notation.
    pub best_move: String,
    /// Principal variation.
    pub pv: Vec<String>,
}

/// GET /api/analysis?fen=...&depth=20
///
/// Analyzes a chess position using Stockfish.
///
/// # Query Parameters
/// * `fen` - Position in FEN notation (required)
/// * `depth` - Search depth (optional, default: 20)
///
/// # Errors
/// * 503 Service Unavailable - Stockfish not configured
/// * 500 Internal Server Error - Analysis failed
pub async fn get_analysis(
    State(state): State<AppState>,
    Query(query): Query<AnalysisQuery>,
) -> Result<Json<AnalysisResponse>, (StatusCode, String)> {
    let pool = state.engine_pool.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Stockfish not configured".to_string(),
        )
    })?;

    let result = pool
        .analyze(&query.fen, query.depth)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AnalysisResponse {
        fen: query.fen,
        depth: result.depth,
        score_cp: result.score_cp,
        score_mate: result.score_mate,
        best_move: result.best_move,
        pv: result.pv,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_depth() {
        assert_eq!(default_depth(), 20);
    }

    #[test]
    fn test_analysis_query_deserialize() {
        let json = r#"{"fen": "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"}"#;
        let query: AnalysisQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.depth, 20); // default
    }

    #[test]
    fn test_analysis_response_serialize() {
        let response = AnalysisResponse {
            fen: "test".to_string(),
            depth: 15,
            score_cp: Some(50),
            score_mate: None,
            best_move: "e2e4".to_string(),
            pv: vec!["e2e4".to_string()],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"depth\":15"));
        assert!(json.contains("\"score_cp\":50"));
    }
}
