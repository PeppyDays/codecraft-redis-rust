use fake::Fake;
use fake::Faker;
use fake::faker::lorem::en::Word;

use crate::client::RedisClient;
use crate::server::RedisServer;

#[tokio::test]
async fn sut_responds_ok_when_client_sets_without_expiration() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;

    let key = Word().fake();
    let value = Word().fake();

    // Act
    let actual = client.set(key, value, None).await;

    // Assert
    let expected = "+OK\r\n";
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn sut_responds_ok_when_client_sets_with_expiration() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;

    let key = Word().fake();
    let value = Word().fake();
    let expired_after: u128 = Faker.fake();

    // Act
    let actual = client.set(key, value, Some(expired_after)).await;

    // Assert
    let expected = "+OK\r\n";
    assert_eq!(actual, expected);
}
