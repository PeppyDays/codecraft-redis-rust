use codecrafters_redis::config::Config;
use codecrafters_redis::config::Replication;
use codecrafters_redis::config::ReplicationMaster;
use codecrafters_redis::config::ReplicationSlave;

use crate::client::RedisClient;
use crate::server::RedisServer;

#[tokio::test]
async fn sut_responds_replication_role_as_slave_if_replication_is_set() {
    // Arrange
    let config = Config {
        replication: Replication {
            master: ReplicationMaster::default(),
            slave: Some(ReplicationSlave {
                host: "localhost".to_string(),
                port: 6380,
            }),
        },
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

#[tokio::test]
async fn sut_responds_master_related_attributes_if_replication_is_not_set() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.info_replication().await;

    // Assert
    let expected = "$89\r\nrole:master\r\nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb\r\nmaster_repl_offset:0\r\n";
    assert_eq!(actual, expected);
}
