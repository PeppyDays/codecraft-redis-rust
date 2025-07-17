use fake::Fake;
use fake::faker::internet::en::Password;
use fake::faker::lorem::en::Word;

use crate::client::RedisClient;
use crate::server::RedisServer;

#[tokio::test]
async fn sut_responds_all_keys_when_client_sends_keys_with_asterisk() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;
    let n = (3..=10).fake::<usize>();
    let keys: Vec<String> = (0..n).map(|_| value()).collect();
    for key in keys.iter() {
        client.set(key, Word().fake(), None).await;
    }

    // Act
    let actual = client.keys("\"*\"").await;

    // Assert
    assert_keys_response(keys.iter().map(|key| key.as_str()).collect(), &actual);
}

#[tokio::test]
async fn sut_responds_the_given_key_when_client_sends_keys_without_asterisk() {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;
    let n = (3..=10).fake::<usize>();
    let keys: Vec<String> = (0..n).map(|_| value()).collect();
    for key in keys.iter() {
        client.set(key, Word().fake(), None).await;
    }
    let first_key = keys.first().unwrap();

    // Act
    let actual = client.keys(format!("\"{first_key}\"").as_str()).await;

    // Assert
    assert_keys_response(vec![first_key], &actual);
}

#[rstest::rstest]
#[case("h*", vec!["hello", "hi", "hps"])]
#[case("a*e", vec!["arine"])]
#[case("*s", vec!["redis", "hps"])]
#[tokio::test]
async fn sut_responds_the_matched_keys_as_asterisk_to_whatever_when_client_sends_keys(
    #[case] pattern: &str,
    #[case] expected_keys: Vec<&str>,
) {
    // Arrange
    let server = RedisServer::new().await;
    let client = RedisClient::new(server.address).await;
    let keys: Vec<&str> = vec!["hello", "arine", "redis", "hi", "hps"];
    for key in keys {
        client.set(key, &value(), None).await;
    }

    // Act
    let actual = client.keys(pattern).await;

    // Assert
    assert_keys_response(expected_keys, &actual);
}

fn value() -> String {
    Password(32..33).fake()
}

fn assert_keys_response(expected_keys: Vec<&str>, actual: &str) {
    let n = expected_keys.len();
    assert!(actual.starts_with(format!("*{n}\r\n").as_str()));
    for key in expected_keys {
        assert!(actual.contains(format!("${}\r\n{}\r\n", key.len(), key).as_str()));
    }
}
