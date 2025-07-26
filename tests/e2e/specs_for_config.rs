use codecrafters_redis::config::Config;
use codecrafters_redis::config::RdbConfig;
use fake::Fake;
use fake::faker::filesystem::en::DirPath;
use fake::faker::filesystem::en::FileName;

use crate::client::RedisClient;
use crate::server::RedisServer;

#[tokio::test]
async fn sut_responds_dir_in_config_when_clients_sends_config_get_of_dir() {
    // Arrange
    let directory: String = DirPath().fake();
    let filename: String = FileName().fake();
    let config = Config {
        rdb: Some(RdbConfig {
            directory: directory.clone(),
            filename: filename.clone(),
        }),
        ..Default::default()
    };
    let server = RedisServer::new_with_config(config).await;
    let client = RedisClient::new(server.address).await;

    // Act
    let actual = client.config_get("dir").await;

    // Assert
    let expected = format!(
        "*2\r\n$3\r\ndir\r\n${}\r\n{}\r\n",
        directory.len(),
        directory,
    );
    assert_eq!(actual, expected);
}
