use std::fs::File;
use std::io::Write;

use codecrafters_redis::config::Config;
use codecrafters_redis::config::RdbConfig;
use tempfile::tempdir;

use crate::client::RedisClient;
use crate::server::RedisServer;

#[tokio::test]
async fn sut_starts_without_any_keys_and_values_when_dir_and_dbfilename_are_not_provided() {
    // Arrange
    let config = Config { rdb: None };
    let server = RedisServer::new_with_config(config).await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.keys("*").await;

    // Assert
    let expected = "*0\r\n";
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn sut_starts_without_any_keys_and_values_when_dir_and_dbfilename_are_provided_but_rdb_file_does_not_have_any_entries()
 {
    // Arrange
    let rdb_directory = tempdir().unwrap();
    let mut rdb_file = File::create(rdb_directory.path().join("dump.rdb")).unwrap();
    let _ = rdb_file
        .write(&[0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31])
        .unwrap();

    let config = Config {
        rdb: Some(RdbConfig {
            directory: rdb_directory.path().to_string_lossy().to_string(),
            filename: "dump.rdb".to_string(),
        }),
    };
    let server = RedisServer::new_with_config(config).await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.keys("*").await;

    // Assert
    let expected = "*0\r\n";
    assert_eq!(actual, expected);
}
