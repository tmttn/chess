//! Bot Arena Server
//!
//! A minimal Axum-based web server that will serve:
//! - REST API endpoints for bot/match data
//! - WebSocket for live match updates
//! - Static files for the SvelteKit frontend

use axum::{routing::get, Router};
use std::net::SocketAddr;

/// Health check endpoint.
///
/// Returns "ok" to indicate the server is running.
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/health", get(health));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
