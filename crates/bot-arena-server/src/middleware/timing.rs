//! Request timing middleware.
//!
//! This middleware logs the duration of each HTTP request, helping identify
//! slow API endpoints that may need optimization.

use axum::{body::Body, http::Request, middleware::Next, response::Response};
use std::time::Instant;

/// Middleware that logs request timing.
///
/// Logs slow requests (>100ms) as warnings and normal requests as debug.
/// This helps identify performance bottlenecks in API endpoints.
///
/// # Example
///
/// ```ignore
/// use axum::{Router, middleware};
/// use bot_arena_server::middleware::timing_layer;
///
/// let app = Router::new()
///     .route("/api/example", get(handler))
///     .layer(middleware::from_fn(timing_layer));
/// ```
pub async fn timing_layer(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status().as_u16();

    // Log slow requests (>100ms) as warnings
    if duration.as_millis() > 100 {
        tracing::warn!(
            method = %method,
            path = %uri,
            status = status,
            duration_ms = duration.as_millis(),
            "Slow request"
        );
    } else {
        tracing::debug!(
            method = %method,
            path = %uri,
            status = status,
            duration_ms = duration.as_millis(),
            "Request completed"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "ok"
    }

    async fn slow_handler() -> &'static str {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        "slow"
    }

    #[tokio::test]
    async fn test_timing_middleware_fast_request() {
        let app: Router = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(timing_layer));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_timing_middleware_slow_request() {
        let app: Router = Router::new()
            .route("/slow", get(slow_handler))
            .layer(middleware::from_fn(timing_layer));

        let response = app
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_timing_middleware_preserves_response() {
        let app: Router = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(timing_layer));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"ok");
    }
}
