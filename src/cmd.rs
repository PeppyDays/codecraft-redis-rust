use crate::config::Config;
use crate::repository::Repository;
use crate::resp::Value;

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Ping,
    Echo {
        message: String,
    },
    Set {
        key: String,
        value: String,
        expires_after: Option<u128>,
    },
    Get {
        key: String,
    },
    Keys {
        pattern: String,
    },
    ConfigGet {
        key: String,
    },
}

impl From<Value> for Cmd {
    fn from(value: Value) -> Self {
        let Value::Array(arr) = value else {
            panic!("Unsupported command");
        };

        let cmd_name = Self::extract_bulk_string(&arr, 0);
        match cmd_name.to_uppercase().as_str() {
            "PING" => Cmd::Ping,
            "ECHO" => Self::parse_echo_command(&arr),
            "SET" => Self::parse_set_command(&arr),
            "GET" => Self::parse_get_command(&arr),
            "KEYS" => Self::parse_keys_command(&arr),
            "CONFIG" => Self::parge_config(&arr),
            _ => panic!("Unsupported command"),
        }
    }
}

impl Cmd {
    fn extract_bulk_string(arr: &[Value], index: usize) -> &str {
        let Value::BulkString(s) = arr.get(index).unwrap() else {
            panic!();
        };
        s
    }

    fn parse_echo_command(arr: &[Value]) -> Self {
        let message = Self::extract_bulk_string(arr, 1);
        Cmd::Echo {
            message: message.to_string(),
        }
    }

    fn parse_get_command(arr: &[Value]) -> Self {
        let key = Self::extract_bulk_string(arr, 1);
        Cmd::Get {
            key: key.to_string(),
        }
    }

    fn parse_set_command(arr: &[Value]) -> Self {
        let key = Self::extract_bulk_string(arr, 1);
        let value = Self::extract_bulk_string(arr, 2);
        let expires_after = Self::parse_set_expiration(arr);
        Cmd::Set {
            key: key.to_string(),
            value: value.to_string(),
            expires_after,
        }
    }

    fn parse_keys_command(arr: &[Value]) -> Self {
        let pattern = Self::extract_bulk_string(arr, 1).trim_matches('"');
        Cmd::Keys {
            pattern: pattern.to_string(),
        }
    }

    fn parse_set_expiration(arr: &[Value]) -> Option<u128> {
        if arr.len() == 3 {
            return None;
        }

        let sub_cmd_name = Self::extract_bulk_string(arr, 3);
        if sub_cmd_name.to_uppercase() != "PX" {
            panic!("SET command with expiration expects 'PX' as the third argument");
        }
        let expiration_str = Self::extract_bulk_string(arr, 4);
        let expiration: u128 = expiration_str.parse().expect("Invalid expiration time");
        Some(expiration)
    }

    fn parge_config(arr: &[Value]) -> Cmd {
        if arr.len() < 3 || Self::extract_bulk_string(arr, 1).to_uppercase() != "GET" {
            panic!("Unsupported CONFIG command");
        }
        let key = Self::extract_bulk_string(arr, 2);
        Cmd::ConfigGet {
            key: key.to_string(),
        }
    }
}

pub async fn execute(repository: &impl Repository, cmd: Cmd) -> Value {
    match cmd {
        Cmd::Ping => Value::SimpleString("PONG".to_string()),
        Cmd::Echo { message } => Value::BulkString(message),
        Cmd::Set {
            key,
            value,
            expires_after,
        } => {
            repository.set(&key, &value, expires_after).await;
            Value::SimpleString("OK".to_string())
        }
        Cmd::Get { key } => match repository.get(&key).await {
            Some(value) => Value::BulkString(value),
            None => Value::Null,
        },
        Cmd::Keys { pattern } => {
            let keys = repository.get_all_keys().await;
            Value::Array(
                keys.into_iter()
                    .filter(|key| match_asterisk_pattern(&pattern, key))
                    .map(Value::BulkString)
                    .collect(),
            )
        }
        Cmd::ConfigGet { key } => {
            let config = Config::global();
            let v = config.get(&key);
            if v.is_none() {
                return Value::Null;
            }
            let v = v.unwrap();
            Value::Array(vec![
                Value::BulkString(key.to_string()),
                Value::BulkString(v.to_string()),
            ])
        }
    }
}

fn match_asterisk_pattern(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix('*') {
        text.starts_with(prefix)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        text.ends_with(suffix)
    } else if let Some(pos) = pattern.find('*') {
        let prefix = &pattern[..pos];
        let suffix = &pattern[pos + 1..];
        text.starts_with(prefix) && text.ends_with(suffix)
    } else {
        pattern == text
    }
}

#[cfg(test)]
mod specs_for_executing_command {
    use std::time::Duration;

    use fake::Fake;
    use fake::faker::internet::en::Password;
    use fake::faker::lorem::en::Word;
    use tokio::time::sleep;

    use crate::repository::InMemoryRepository;
    use crate::repository::Repository;
    use crate::resp::Value;

    use super::Cmd;
    use super::execute;

    struct DummyRepository;

    #[async_trait::async_trait]
    impl Repository for DummyRepository {
        async fn set(&self, _key: &str, _value: &str, _expires_after: Option<u128>) {}
        async fn get(&self, _key: &str) -> Option<String> {
            None
        }
        async fn get_all_keys(&self) -> Vec<String> {
            vec![]
        }
    }

    #[tokio::test]
    async fn sut_responds_pong_when_gets_ping_command() {
        // Arrange
        let dummy_repository = DummyRepository;
        let cmd = Cmd::Ping;

        // Act
        let actual = execute(&dummy_repository, cmd).await;

        // Assert
        let expected = Value::SimpleString("PONG".to_string());
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_echo_when_gets_echo_command() {
        // Arrange
        let dummy_repository = DummyRepository;
        let message = Word().fake::<String>();
        let cmd = Cmd::Echo {
            message: message.clone(),
        };

        // Act
        let actual = execute(&dummy_repository, cmd).await;

        // Assert
        let expected = Value::BulkString(message);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_ok_when_gets_set_command() {
        // Arrange
        let dummy_repository = DummyRepository;
        let key = Word().fake::<String>();
        let value = Word().fake::<String>();
        let cmd = Cmd::Set {
            key: key.clone(),
            value: value.clone(),
            expires_after: None,
        };

        // Act
        let actual = execute(&dummy_repository, cmd).await;

        // Assert
        let expected = Value::SimpleString("OK".to_string());
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_value_when_gets_get_command() {
        // Arrange
        let repository = InMemoryRepository::new();
        let key = Word().fake::<String>();
        let value = Word().fake::<String>();
        let set_cmd = Cmd::Set {
            key: key.clone(),
            value: value.clone(),
            expires_after: None,
        };
        execute(&repository, set_cmd).await;
        let get_cmd = Cmd::Get { key: key.clone() };

        // Act
        let actual = execute(&repository, get_cmd).await;

        // Assert
        let expected = Value::BulkString(value);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_null_when_gets_get_command_but_value_is_expired() {
        // Arrange
        let repository = InMemoryRepository::new();
        let key = Word().fake::<String>();
        let value = Word().fake::<String>();
        let expires_after: u128 = 50;
        let set_cmd = Cmd::Set {
            key: key.clone(),
            value: value.clone(),
            expires_after: Some(expires_after),
        };
        execute(&repository, set_cmd).await;
        let get_cmd = Cmd::Get { key: key.clone() };

        // Act
        sleep(Duration::from_millis(60)).await;
        let actual = execute(&repository, get_cmd).await;

        // Assert
        let expected = Value::Null;
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_all_keys_when_keys_command_pattern_is_asterisk() {
        // Arrange
        let repository = InMemoryRepository::new();
        let n = (3..=10).fake::<usize>();
        let keys: Vec<String> = (0..n).map(|_| Password(32..33).fake()).collect();
        for key in keys.iter() {
            let cmd = Cmd::Set {
                key: key.clone(),
                value: Password(32..33).fake(),
                expires_after: None,
            };
            execute(&repository, cmd).await;
        }
        let cmd = Cmd::Keys {
            pattern: "*".to_string(),
        };

        // Act
        let actual = execute(&repository, cmd).await;

        // Assert
        let expected = Value::Array(keys.into_iter().map(Value::BulkString).collect());
        assert_eq!(sort_value_array(&actual), sort_value_array(&expected));
    }

    #[tokio::test]
    async fn sut_responds_the_given_key_when_keys_command_pattern_is_exactly_the_key() {
        // Arrange
        let repository = InMemoryRepository::new();
        let n = (3..=10).fake::<usize>();
        let keys: Vec<String> = (0..n).map(|_| Password(32..33).fake()).collect();
        for key in keys.iter() {
            let cmd = Cmd::Set {
                key: key.clone(),
                value: Word().fake(),
                expires_after: None,
            };
            execute(&repository, cmd).await;
        }
        let first_key = keys.first().unwrap();
        let cmd = Cmd::Keys {
            pattern: first_key.to_string(),
        };

        // Act
        let actual = execute(&repository, cmd).await;

        // Assert
        let expected = Value::Array(vec![Value::BulkString(first_key.to_string())]);
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[tokio::test]
    #[case("h*", vec!["healingpaper"])]
    async fn sut_responds_the_matched_keys_as_asterisk_to_whatever(
        #[case] pattern: &str,
        #[case] expected_keys: Vec<&str>,
    ) {
        // Arrange
        let repository = InMemoryRepository::new();
        let keys: Vec<&str> = vec!["healingpaper", "arine"];
        for key in keys.iter() {
            let cmd = Cmd::Set {
                key: key.to_string(),
                value: Password(32..33).fake(),
                expires_after: None,
            };
            execute(&repository, cmd).await;
        }
        let cmd = Cmd::Keys {
            pattern: pattern.to_string(),
        };

        // Act
        let actual = execute(&repository, cmd).await;

        // Assert
        let expected = Value::Array(vec![Value::BulkString("healingpaper".to_string())]);
        assert_eq!(actual, expected);
    }

    fn sort_value_array(value: &Value) -> Value {
        match value {
            Value::Array(arr) => {
                let mut sorted_arr = arr.clone();
                sorted_arr.sort();
                Value::Array(sorted_arr)
            }
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod specs_for_converting_from_value {
    use fake::Fake;
    use fake::Faker;
    use fake::faker::lorem::en::Word;

    use crate::resp::Value;

    use super::Cmd;

    #[rstest::rstest]
    #[case("PING")]
    #[case("ping")]
    #[case("PiNg")]
    fn sut_parses_ping_command_with_case_insensitive(#[case] ping: &str) {
        // Arrange
        let value = Value::Array(vec![Value::BulkString(ping.to_string())]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::Ping;
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_echo_command_correctly() {
        // Arrange
        let message: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString("ECHO".to_string()),
            Value::BulkString(message.to_string()),
        ]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::Echo {
            message: message.to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case("ECHO")]
    #[case("echo")]
    #[case("EcHo")]
    fn sut_parses_echo_command_with_case_insensitive(#[case] echo: &str) {
        // Arrange
        let message: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString(echo.to_string()),
            Value::BulkString(message.to_string()),
        ]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::Echo {
            message: message.to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_set_command_without_expiration_correctly() {
        // Arrange
        let set_key: &str = Word().fake();
        let set_value: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString("SET".to_string()),
            Value::BulkString(set_key.to_string()),
            Value::BulkString(set_value.to_string()),
        ]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::Set {
            key: set_key.to_string(),
            value: set_value.to_string(),
            expires_after: None,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_set_command_with_expiration_correctly() {
        // Arrange
        let set_key: &str = Word().fake();
        let set_value: &str = Word().fake();
        let set_expires_after: u128 = Faker.fake();
        let value = Value::Array(vec![
            Value::BulkString("SET".to_string()),
            Value::BulkString(set_key.to_string()),
            Value::BulkString(set_value.to_string()),
            Value::BulkString("PX".to_string()),
            Value::BulkString(set_expires_after.to_string()),
        ]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::Set {
            key: set_key.to_string(),
            value: set_value.to_string(),
            expires_after: Some(set_expires_after),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_get_command_correctly() {
        // Arrange
        let get_key: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString("GET".to_string()),
            Value::BulkString(get_key.to_string()),
        ]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::Get {
            key: get_key.to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_config_get_command_correctly() {
        // Arrange
        let config_key: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString("CONFIG".to_string()),
            Value::BulkString("GET".to_string()),
            Value::BulkString(config_key.to_string()),
        ]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::ConfigGet {
            key: config_key.to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_keys_command_correctly() {
        // Arrange
        let pattern: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString("KEYS".to_string()),
            Value::BulkString(pattern.to_string()),
        ]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::Keys {
            pattern: pattern.to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_keys_command_and_trim_double_quotes_if_exists_in_pattern() {
        // Arrange
        let pattern: String = Word().fake();
        let surrounded_pattern = format!("\"{pattern}\"");
        let value = Value::Array(vec![
            Value::BulkString("KEYS".to_string()),
            Value::BulkString(surrounded_pattern.to_string()),
        ]);

        // Act
        let actual = Cmd::from(value);

        // Assert
        let expected = Cmd::Keys {
            pattern: pattern.to_string(),
        };
        assert_eq!(actual, expected);
    }
}
