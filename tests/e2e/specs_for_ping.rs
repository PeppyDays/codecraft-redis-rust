use crate::client::RedisClient;
use crate::server::RedisServer;

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
