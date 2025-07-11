use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use codecrafters_redis::run;

#[tokio::test]
async fn sut_responds_pong_when_gets_ping() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.ping().await;

    // Assert
    assert_eq!(actual, "+PONG\r\n");
}

#[tokio::test]
async fn sut_responds_pongs_whenever_gets_pings() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;

    // Act & Assert
    for _ in 0..10 {
        let actual = client.ping().await;
        assert_eq!(actual, "+PONG\r\n");
    }
}

#[tokio::test]
async fn sut_responds_pongs_when_clients_ping() {
    // Arrange
    let server = RedisServer::new().await;
    let client_1 = RedisClient::new(server.address).await;
    let client_2 = RedisClient::new(server.address).await;

    // Act
    let (actual_1, actual_2) = tokio::join!(client_1.ping(), client_2.ping());

    // Assert
    assert_eq!(actual_1, "+PONG\r\n");
    assert_eq!(actual_2, "+PONG\r\n");
}

struct RedisServer {
    address: SocketAddr,
}

impl RedisServer {
    async fn new() -> Self {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(run(listener));
        Self { address }
    }
}

struct RedisClient {
    stream: Mutex<TcpStream>,
}

impl RedisClient {
    async fn new(address: SocketAddr) -> Self {
        let stream = TcpStream::connect(address).await.unwrap();
        Self {
            stream: Mutex::new(stream),
        }
    }

    async fn ping(&self) -> String {
        self.stream
            .lock()
            .await
            .write_all(b"*1\r\n$4\r\nPING\r\n")
            .await
            .unwrap();
        self.get_response().await
    }

    async fn get_response(&self) -> String {
        let mut buffer = [0; 1024];
        let bytes_read = self.stream.lock().await.read(&mut buffer).await.unwrap();
        String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
    }
}
