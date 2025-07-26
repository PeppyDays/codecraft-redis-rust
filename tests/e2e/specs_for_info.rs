use crate::client::RedisClient;
use crate::server::RedisServer;

#[tokio::test]
async fn sut_responds_replication_role_when_client_requests_info_replication() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.info_replication().await;

    // Assert
    let expected = "$11\r\nrole:master\r\n";
    assert_eq!(actual, expected);
}
