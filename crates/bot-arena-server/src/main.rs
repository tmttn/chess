//! Bot Arena Server
//!
//! A minimal Axum-based web server that will serve:
//! - REST API endpoints for bot/match data
//! - WebSocket for live match updates
//! - Static files for the SvelteKit frontend

mod api;
mod db;
mod elo;
mod models;
mod repo;
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
    let state = AppState { db, ws_broadcast };

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
        .route("/api/bots", get(api::bots::list_bots))
        .route("/api/bots/:name", get(api::bots::get_bot))
        .route(
            "/api/matches",
            get(api::matches::list_matches).post(api::matches::create_match),
        )
        .route("/api/matches/:id", get(api::matches::get_match_detail))
        .route("/api/games/:id/moves", get(api::matches::get_game_moves))
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
