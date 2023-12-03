async fn say_world() {
    println!("world");
}

// async/awaitを使うと、Rustはコンパイル時に非同期的な処理にする
#[tokio::main]
async fn main() {
    // Calling `say_world()` does not execute the body of `say_world()`.
    let op = say_world();

    // This println! comes first
    println!("hello");

    // Calling `.await` on `op` starts executing `say_world`.
    op.await;
}