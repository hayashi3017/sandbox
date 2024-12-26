use std::{sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use leaky_bucket::RateLimiter;
use tracing::{debug, level_filters::LevelFilter};

struct AppState {
    tokens: Arc<RateLimiter>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            tokens: Arc::new(
                RateLimiter::builder()
                    .initial(5)
                    .interval(Duration::from_secs(1))
                    .refill(1)
                    .max(5)
                    .build(),
            ),
        }
    }
}

#[tokio::main]
async fn main() {
    use tracing_subscriber::prelude::*;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_level(true)
                .compact(),
        )
        .init();

    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/bucket", post(use_bucket))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn use_bucket(State(state): State<Arc<AppState>>) -> Response {
    debug!("API called.");
    if !state.tokens.try_acquire(1) {
        return (StatusCode::TOO_MANY_REQUESTS, "No token left.\n").into_response();
    }
    (StatusCode::OK, "Token used.\n").into_response()
}
