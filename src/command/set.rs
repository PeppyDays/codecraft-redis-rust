use crate::command::executor::Command;
use crate::command::executor::CommandExecutor;
use crate::command::executor::CommandExecutorContext;
use crate::command::parser::extract_array;
use crate::command::parser::extract_bulk_string;
use crate::command::parser::validate_main_command;
use crate::command::parser::validate_min_array_length;
use crate::resp::Value;

#[derive(Debug, Default, PartialEq)]
pub struct Set {
    key: String,
    value: String,
    expires_after: Option<u128>,
}

impl Command for Set {
    fn parse_from(value: &Value) -> Result<Self, anyhow::Error> {
        let array = extract_array(value)?;
        validate_min_array_length(array, 3)?;
        validate_main_command(array, "SET")?;
        let key = extract_bulk_string(array, 1)?;
        let value = extract_bulk_string(array, 2)?;

        let expires_after = if array.len() >= 5 {
            let option_key = extract_bulk_string(array, 3)?;
            if option_key.to_uppercase() == "PX" {
                let option_value = extract_bulk_string(array, 4)?;
                Some(option_value.parse()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Set {
            key: key.to_string(),
            value: value.to_string(),
            expires_after,
        })
    }
}

#[async_trait::async_trait]
impl CommandExecutor for Set {
    async fn execute(&self, context: CommandExecutorContext) -> Value {
        context
            .repository
            .set(&self.key, &self.value, self.expires_after)
            .await;
        Value::SimpleString("OK".to_string())
    }
}

#[cfg(test)]
mod specs_for_parse_from {
    use fake::Fake;
    use fake::Faker;
    use fake::faker::lorem::en::Word;

    use crate::command::executor::Command;
    use crate::resp::Value;

    use super::Set;

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
        let actual = Set::parse_from(&value).unwrap();

        // Assert
        let expected = Set {
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
        let actual = Set::parse_from(&value).unwrap();

        // Assert
        let expected = Set {
            key: set_key.to_string(),
            value: set_value.to_string(),
            expires_after: Some(set_expires_after),
        };
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case("SET")]
    #[case("set")]
    #[case("SeT")]
    fn sut_parses_set_command_with_case_insensitive(#[case] set: &str) {
        // Arrange
        let set_key: &str = Word().fake();
        let set_value: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString(set.to_string()),
            Value::BulkString(set_key.to_string()),
            Value::BulkString(set_value.to_string()),
        ]);

        // Act
        let actual = Set::parse_from(&value).unwrap();

        // Assert
        let expected = Set {
            key: set_key.to_string(),
            value: set_value.to_string(),
            expires_after: None,
        };
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod specs_for_execute {
    use std::sync::Arc;
    use std::time::Duration;

    use fake::Fake;
    use fake::faker::lorem::en::Word;
    use tokio::time::sleep;

    use crate::command::executor::CommandExecutor;
    use crate::command::executor::CommandExecutorContext;
    use crate::repository::InMemoryRepository;
    use crate::repository::Repository;
    use crate::resp::Value;

    use super::Set;

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
        async fn entries(&self) -> Vec<(String, (String, Option<u128>))> {
            vec![]
        }
    }

    #[tokio::test]
    async fn sut_responds_ok_when_gets_set_command() {
        // Arrange
        let context = CommandExecutorContext::new(Arc::new(DummyRepository));
        let key = Word().fake::<String>();
        let value = Word().fake::<String>();
        let cmd = Set {
            key: key.clone(),
            value: value.clone(),
            expires_after: None,
        };

        // Act
        let actual = cmd.execute(context).await;

        // Assert
        let expected = Value::SimpleString("OK".to_string());
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_value_when_gets_get_command() {
        // Arrange
        let repository = Arc::new(InMemoryRepository::new());
        let context = CommandExecutorContext::new(repository.clone());
        let key = Word().fake::<String>();
        let value = Word().fake::<String>();
        let set_cmd = Set {
            key: key.clone(),
            value: value.clone(),
            expires_after: None,
        };
        set_cmd.execute(context.clone()).await;

        // Act
        let actual = repository.get(&key).await;

        // Assert
        assert_eq!(actual, Some(value));
    }

    #[tokio::test]
    async fn sut_responds_null_when_gets_get_command_but_value_is_expired() {
        // Arrange
        let repository = Arc::new(InMemoryRepository::new());
        let context = CommandExecutorContext::new(repository.clone());
        let key = Word().fake::<String>();
        let value = Word().fake::<String>();
        let expires_after: u128 = 50;
        let set_cmd = Set {
            key: key.clone(),
            value: value.clone(),
            expires_after: Some(expires_after),
        };
        set_cmd.execute(context).await;

        // Act
        sleep(Duration::from_millis(60)).await;
        let actual = repository.get(&key).await;

        // Assert
        assert_eq!(actual, None);
    }
}
