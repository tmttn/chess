//! Bot Arena Server
//!
//! A minimal Axum-based web server that will serve:
//! - REST API endpoints for bot/match data
//! - WebSocket for live match updates
//! - Static files for the SvelteKit frontend

mod analysis;
mod api;
mod db;
mod elo;
mod models;
mod repo;
mod watcher;
mod ws;

use axum::routing::get;
use axum::Router;
use db::DbPool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool.
    pub db: DbPool,
    /// WebSocket broadcast channel for live match updates.
    pub ws_broadcast: ws::WsBroadcast,
    /// Stockfish engine pool for position analysis.
    pub engine_pool: Option<std::sync::Arc<analysis::EnginePool>>,
}

/// Health check endpoint.
///
/// Returns "ok" to indicate the server is running.
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Create data directory if needed
    std::fs::create_dir_all("data").expect("Failed to create data directory");

    let db = db::init_db("data/arena.db").expect("Failed to initialize database");
    let ws_broadcast = ws::create_broadcast();

    // Create engine pool if STOCKFISH_PATH is set
    let engine_pool = std::env::var("STOCKFISH_PATH").ok().map(|path| {
        tracing::info!("Stockfish analysis enabled: {}", path);
        std::sync::Arc::new(analysis::EnginePool::new(path, 2)) // Pool size of 2
    });

    let state = AppState {
        db,
        ws_broadcast,
        engine_pool,
    };

    // Spawn move watcher for live updates
    let db_for_watcher = state.db.clone();
    let broadcast_for_watcher = state.ws_broadcast.clone();
    tokio::spawn(async move {
        watcher::watch_moves(db_for_watcher, broadcast_for_watcher).await;
    });

    // CORS layer for cross-origin requests
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // WebSocket route with broadcast state
    let ws_router = Router::new()
        .route("/ws", get(ws::ws_handler))
        .with_state(state.ws_broadcast.clone());

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/analysis", get(api::analysis::get_analysis))
        .route("/api/bots", get(api::bots::list_bots))
        .route("/api/bots/:name", get(api::bots::get_bot))
        .route(
            "/api/matches",
            get(api::matches::list_matches).post(api::matches::create_match),
        )
        .route("/api/matches/:id", get(api::matches::get_match_detail))
        .route("/api/games/:id/moves", get(api::matches::get_game_moves))
        .route("/api/export/match/:id", get(api::export::export_match))
        .route("/api/export/game/:id", get(api::export::export_game))
        .route("/api/export/bot/:name", get(api::export::export_bot))
        .route("/api/openings", get(api::openings::list_openings))
        .route("/api/stats/head-to-head", get(api::stats::head_to_head))
        .with_state(state)
        .merge(ws_router)
        .layer(cors)
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app).await.expect("Server error");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_returns_ok() {
        let result = health().await;
        assert_eq!(result, "ok");
    }
}
