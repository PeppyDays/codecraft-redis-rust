use crate::command::executor::Command;
use crate::command::executor::CommandExecutor;
use crate::command::executor::CommandExecutorContext;
use crate::command::parser::extract_array;
use crate::command::parser::extract_bulk_string;
use crate::command::parser::validate_array_length;
use crate::command::parser::validate_main_command;
use crate::resp::Value;

#[derive(Debug, Default, PartialEq)]
pub struct Get {
    key: String,
}

impl Command for Get {
    fn parse_from(value: &Value) -> Result<Self, anyhow::Error> {
        let array = extract_array(value)?;
        validate_array_length(array, 2)?;
        validate_main_command(array, "GET")?;
        let key = extract_bulk_string(array, 1)?;
        Ok(Get {
            key: key.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl CommandExecutor for Get {
    async fn execute(&self, context: CommandExecutorContext) -> Value {
        match context.repository.get(&self.key).await {
            Some(value) => Value::BulkString(value),
            None => Value::Null,
        }
    }
}

#[cfg(test)]
mod specs_for_parse_from {
    use fake::Fake;
    use fake::faker::lorem::en::Word;

    use crate::command::executor::Command;
    use crate::resp::Value;

    use super::Get;

    #[test]
    fn sut_parses_get_command_correctly() {
        // Arrange
        let get_key: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString("GET".to_string()),
            Value::BulkString(get_key.to_string()),
        ]);

        // Act
        let actual = Get::parse_from(&value).unwrap();

        // Assert
        let expected = Get {
            key: get_key.to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case("GET")]
    #[case("get")]
    #[case("GeT")]
    fn sut_parses_get_command_with_case_insensitive(#[case] get: &str) {
        // Arrange
        let get_key: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString(get.to_string()),
            Value::BulkString(get_key.to_string()),
        ]);

        // Act
        let actual = Get::parse_from(&value).unwrap();

        // Assert
        let expected = Get {
            key: get_key.to_string(),
        };
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod specs_for_execute {
    use std::sync::Arc;

    use fake::Fake;
    use fake::faker::lorem::en::Word;

    use crate::command::executor::CommandExecutor;
    use crate::command::executor::CommandExecutorContext;
    use crate::repository::InMemoryRepository;
    use crate::repository::Repository;
    use crate::resp::Value;

    use super::Get;

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
    async fn sut_responds_value_when_gets_get_command() {
        // Arrange
        let repository = Arc::new(InMemoryRepository::new());
        let context = CommandExecutorContext::new(repository.clone());
        let key = Word().fake::<String>();
        let value = Word().fake::<String>();

        repository.set(&key, &value, None).await;

        let get_cmd = Get { key: key.clone() };

        // Act
        let actual = get_cmd.execute(context).await;

        // Assert
        let expected = Value::BulkString(value);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_null_when_key_not_found() {
        // Arrange
        let context = CommandExecutorContext::new(Arc::new(DummyRepository));
        let key = Word().fake::<String>();
        let get_cmd = Get { key: key.clone() };

        // Act
        let actual = get_cmd.execute(context).await;

        // Assert
        let expected = Value::Null;
        assert_eq!(actual, expected);
    }
}
