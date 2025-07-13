use crate::repository::Repository;
use crate::resp::Value;

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Ping,
    Echo { message: String },
    Set { key: String, value: String },
    Get { key: String },
}

impl From<Value> for Cmd {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(arr) => Self::parse_array_command(arr),
            _ => panic!("Unsupported command"),
        }
    }
}

impl Cmd {
    fn is_ping(cmd: &str) -> bool {
        cmd.eq_ignore_ascii_case("PING")
    }

    fn is_echo(cmd: &str) -> bool {
        cmd.eq_ignore_ascii_case("ECHO")
    }

    fn is_set(cmd: &str) -> bool {
        cmd.eq_ignore_ascii_case("SET")
    }

    fn is_get(cmd: &str) -> bool {
        cmd.eq_ignore_ascii_case("GET")
    }

    fn parse_array_command(arr: Vec<Value>) -> Self {
        match arr.first().unwrap() {
            Value::BulkString(cmd) if Self::is_ping(cmd) => Cmd::Ping,
            Value::BulkString(cmd) if Self::is_echo(cmd) => {
                if let Value::BulkString(message) = arr.get(1).unwrap() {
                    Cmd::Echo {
                        message: message.clone(),
                    }
                } else {
                    panic!("ECHO command expects a bulk string as an argument");
                }
            }
            Value::BulkString(cmd) if Self::is_set(cmd) => {
                if arr.len() == 3 {
                    if let (Value::BulkString(key), Value::BulkString(value)) =
                        (arr.get(1).unwrap(), arr.get(2).unwrap())
                    {
                        Cmd::Set {
                            key: key.clone(),
                            value: value.clone(),
                        }
                    } else {
                        panic!("SET command expects two bulk strings as arguments");
                    }
                } else {
                    panic!("SET command expects exactly two arguments");
                }
            }
            Value::BulkString(cmd) if Self::is_get(cmd) => {
                if arr.len() == 2 {
                    if let Value::BulkString(key) = arr.get(1).unwrap() {
                        Cmd::Get { key: key.clone() }
                    } else {
                        panic!("GET command expects a bulk string as argument");
                    }
                } else {
                    panic!("GET command expects exactly one argument");
                }
            }
            _ => panic!("Unsupported command"),
        }
    }
}

pub async fn execute(repository: &impl Repository, cmd: Cmd) -> Value {
    match cmd {
        Cmd::Ping => Value::SimpleString("PONG".to_string()),
        Cmd::Echo { message } => Value::BulkString(message),
        Cmd::Set { key, value } => {
            repository.set(&key, &value).await;
            Value::SimpleString("OK".to_string())
        }
        Cmd::Get { key } => match repository.get(&key).await {
            Some(value) => Value::BulkString(value),
            None => Value::Null,
        },
    }
}

#[cfg(test)]
mod specs_for_executing_command {
    use fake::Fake;
    use fake::faker::lorem::en::Word;

    use crate::repository::InMemoryRepository;
    use crate::repository::Repository;
    use crate::resp::Value;

    use super::Cmd;
    use super::execute;

    struct DummyRepository;

    #[async_trait::async_trait]
    impl Repository for DummyRepository {
        async fn set(&self, _key: &str, _value: &str) {}
        async fn get(&self, _key: &str) -> Option<String> {
            None
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
        };
        execute(&repository, set_cmd).await;
        let get_cmd = Cmd::Get { key: key.clone() };

        // Act
        let actual = execute(&repository, get_cmd).await;

        // Assert
        let expected = Value::BulkString(value);
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod specs_for_converting_from_value {
    use fake::Fake;
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
    fn sut_parses_set_command_correctly() {
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
}
