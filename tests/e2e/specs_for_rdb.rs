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
    let config = Config::default();
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
    let _ = rdb_file.write(header()).unwrap();
    let _ = rdb_file.write(metadata()).unwrap();
    let _ = rdb_file.write(footer()).unwrap();

    let config = Config {
        rdb: Some(RdbConfig {
            directory: rdb_directory.path().to_string_lossy().to_string(),
            filename: "dump.rdb".to_string(),
        }),
        ..Config::default()
    };
    let server = RedisServer::new_with_config(config).await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.keys("*").await;

    // Assert
    let expected = "*0\r\n";
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn sut_loads_entries_in_rdb_file_correctly() {
    // Arrange
    let rdb_directory = tempdir().unwrap();
    let mut rdb_file = File::create(rdb_directory.path().join("dump.rdb")).unwrap();
    let _ = rdb_file.write(header()).unwrap();
    let _ = rdb_file.write(metadata()).unwrap();
    let _ = rdb_file.write(entries()).unwrap();
    let _ = rdb_file.write(footer()).unwrap();

    let config = Config {
        rdb: Some(RdbConfig {
            directory: rdb_directory.path().to_string_lossy().to_string(),
            filename: "dump.rdb".to_string(),
        }),
        ..Config::default()
    };
    let server = RedisServer::new_with_config(config).await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.keys("*").await;

    // Assert
    let expected = "*1\r\n$6\r\nfoobar\r\n";
    assert_eq!(actual, expected);
}

fn header() -> &'static [u8] {
    // REDIS0011
    &[0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31]
}

fn metadata() -> &'static [u8] {
    &[
        0xFA, 0x09, 0x72, 0x65, 0x64, 0x69, 0x73, 0x2D, 0x76, 0x65, 0x72, 0x06, 0x36, 0x2E, 0x30,
        0x2E, 0x31, 0x36,
    ]
}

fn entries() -> &'static [u8] {
    &[
        0xFE, 0x00, 0xFB, 0x01, 0x00, 0x00, 0x06, 0x66, 0x6F, 0x6F, 0x62, 0x61, 0x72, 0x06, 0x62,
        0x61, 0x7A, 0x71, 0x75, 0x78, 0xFC, 0x15, 0x72, 0xE7, 0x07, 0x8F, 0x01, 0x00, 0x00, 0x00,
        0x03, 0x66, 0x6F, 0x6F, 0x03, 0x62, 0x61, 0x72, 0xFD, 0x52, 0xED, 0x2A, 0x66, 0x00, 0x03,
        0x62, 0x61, 0x7A, 0x03, 0x71, 0x75, 0x78,
    ]
}

fn footer() -> &'static [u8] {
    &[0xFF, 0x89, 0x3B, 0xB7, 0x4E, 0xF8, 0x0F, 0x77, 0x19]
}
