use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::sync::Arc;

use tokio::net::TcpListener;

use codecrafters_redis::config::Config;
use codecrafters_redis::repository::InMemoryRepository;
use codecrafters_redis::run;

pub struct RedisServer {
    pub address: SocketAddr,
}

impl RedisServer {
    pub async fn new() -> Self {
        let config = Config::default();
        Self::new_with_config(config).await
    }

    pub async fn new_with_config(config: Config) -> Self {
        Config::initialize(config);

        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let repository = Arc::new(InMemoryRepository::new());
        tokio::spawn(run(listener, repository));
        Self { address }
    }
}
