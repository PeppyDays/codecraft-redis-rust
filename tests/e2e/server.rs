use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::sync::Arc;

use tokio::net::TcpListener;

use codecrafters_redis::repository::InMemoryRepository;
use codecrafters_redis::run;

pub struct RedisServer {
    pub address: SocketAddr,
}

impl RedisServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let repository = Arc::new(InMemoryRepository::new());
        tokio::spawn(run(listener, repository));
        Self { address }
    }
}
