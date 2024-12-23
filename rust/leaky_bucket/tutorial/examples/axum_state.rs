use std::{sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use leaky_bucket::RateLimiter;
use tokio::{sync::Mutex, time};

struct AppState {
    volume: Arc<RateLimiter>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            volume: Arc::new(
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
    let state = Arc::new(AppState::new());

    // トークン数の状態をログに出力するバックグラウンドタスク
    // let state_clone = state.clone();
    // tokio::spawn(async move {
    //     loop {
            // let state_clone = state_clone.lock().await;
            // balance()はrefill()された状態を反映しない様子、つまり現時点の残りのサイズを示さない
            // println!("Current tokens: {}", state_clone.volume.balance());
            // drop(state_clone); // ロックを解放
    //         time::sleep(Duration::from_secs(1)).await;
    //     }
    // });

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/volume", post(volumes))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    // for i in 1..=10 {
    //     // トークンを1つ消費
    //     state.volume.acquire_one().await;
    //     println!("Task {} completed.", i);
    // }
}

async fn volumes(State(state): State<Arc<AppState>>) -> Response {
    // let state = state.lock().await;
    if !state.volume.try_acquire(1) {
        return (StatusCode::TOO_MANY_REQUESTS, "No milk available\n").into_response();
    }
    (StatusCode::OK, "Milk withdrawn\n").into_response()
}