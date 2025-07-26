use codecrafters_redis::config::Config;
use codecrafters_redis::config::Replication;

use crate::client::RedisClient;
use crate::server::RedisServer;

#[tokio::test]
async fn sut_responds_replication_role_as_master_if_replication_is_not_set() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.info_replication().await;

    // Assert
    let expected = "$11\r\nrole:master\r\n";
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn sut_responds_replication_role_as_slave_if_replication_is_set() {
    // Arrange
    let config = Config {
        replication: Some(Replication {
            server_host: "localhost".to_string(),
            server_port: 6380,
        }),
        ..Default::default()
    };
    let server = RedisServer::new_with_config(config).await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.info_replication().await;

    // Assert
    let expected = "$10\r\nrole:slave\r\n";
    assert_eq!(actual, expected);
}
