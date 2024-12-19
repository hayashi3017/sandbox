use leaky_bucket::RateLimiter;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    let bucket = RateLimiter::builder()
        .max(5)
        .initial(0)
        .refill(1)
        .interval(Duration::from_secs(1))
        .build();

    for i in 1..=10 {
        // トークンを1つ消費
        bucket.acquire_one().await;
        println!("Task {} completed.", i);
    }
}
