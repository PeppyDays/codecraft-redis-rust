use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::sync::Arc;

use fake::Fake;
use fake::faker::lorem::en::Word;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use codecrafters_redis::repository::InMemoryRepository;
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
async fn sut_responds_pongs_when_client_pings() {
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

#[tokio::test]
async fn sut_responds_the_same_message_when_client_echos() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;
    let message = Word().fake();

    // Act
    let actual = client.echo(message).await;

    // Assert
    let expected = format!("${}\r\n{}\r\n", message.len(), message);
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn sut_responds_ok_when_client_sets() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;
    let key = Word().fake();
    let value = Word().fake();

    // Act
    let actual = client.set(key, value).await;

    // Assert
    let expected = "+OK\r\n";
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn sut_responds_value_when_client_sets_and_gets() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;
    let key = Word().fake();
    let value = Word().fake();
    client.set(key, value).await;

    // Act
    let actual = client.get(key).await;

    // Assert
    let expected = format!("${}\r\n{}\r\n", value.len(), value);
    assert_eq!(actual, expected);
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
        let repository = Arc::new(InMemoryRepository::new());
        tokio::spawn(run(listener, repository));
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
        self.response().await
    }

    async fn echo(&self, message: &str) -> String {
        let str = format!("*2\r\n$4\r\nECHO\r\n${}\r\n{}\r\n", message.len(), message);
        let buf = str.as_bytes();
        self.stream.lock().await.write_all(buf).await.unwrap();
        self.response().await
    }

    async fn set(&self, key: &str, value: &str) -> String {
        let str = format!(
            "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
            key.len(),
            key,
            value.len(),
            value
        );
        let buf = str.as_bytes();
        self.stream.lock().await.write_all(buf).await.unwrap();
        self.response().await
    }

    async fn get(&self, key: &str) -> String {
        let str = format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", key.len(), key);
        let buf = str.as_bytes();
        self.stream.lock().await.write_all(buf).await.unwrap();
        self.response().await
    }

    async fn response(&self) -> String {
        let mut buf = [0; 512];
        let bytes_read = self.stream.lock().await.read(&mut buf).await.unwrap();
        String::from_utf8_lossy(&buf[..bytes_read]).to_string()
    }
}
