use std::sync::Arc;

use tokio::net::TcpListener;

use codecrafters_redis::repository::InMemoryRepository;
use codecrafters_redis::run;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let repository = Arc::new(InMemoryRepository::new());

    run(listener, repository).await
}
