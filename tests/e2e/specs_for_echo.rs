use fake::Fake;
use fake::faker::lorem::en::Word;

use crate::client::RedisClient;
use crate::server::RedisServer;

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
