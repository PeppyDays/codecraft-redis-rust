use std::time::Duration;

use fake::Fake;
use fake::faker::lorem::en::Word;
use tokio::time::sleep;

use crate::client::RedisClient;
use crate::server::RedisServer;

#[tokio::test]
async fn sut_responds_value_when_client_sets_and_gets_before_expiration() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;
    let key = Word().fake();
    let value = Word().fake();
    let expires_after: u128 = 1000; // 1000 ms
    client.set(key, value, Some(expires_after)).await;

    // Act
    let actual = client.get(key).await;

    // Assert
    let expected = format!("${}\r\n{}\r\n", value.len(), value);
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn sut_responds_null_when_client_sets_and_gets_after_expiration() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;
    let key = Word().fake();
    let value = Word().fake();
    let expires_after: u128 = 50; // 50 ms
    client.set(key, value, Some(expires_after)).await;

    // Act
    sleep(Duration::from_millis(60)).await;
    let actual = client.get(key).await;

    // Assert
    let expected = "$-1\r\n";
    assert_eq!(actual, expected);
}
