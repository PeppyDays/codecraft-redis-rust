use tokio::net::TcpListener;

use codecrafters_redis::run;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    run(listener).await
}
