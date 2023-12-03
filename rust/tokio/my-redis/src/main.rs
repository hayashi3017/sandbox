// https://tokio.rs/tokio/tutorial/hello-tokio#breaking-it-down

use mini_redis::{client, Result};

/// ランタイムを必要とする非同期処理、その非同期処理を同期処理にするマクロ
/// 以下のようにする
/// fn main() {
/// let mut rt = tokio::runtime::Runtime::new().unwrap();
/// rt.block_on(async {
///     println!("hello");
/// })
/// }
#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;

    // Get key "hello"
    let result = client.get("hello").await?;

    println!("got value from the server; result={:?}", result);

    Ok(())
}