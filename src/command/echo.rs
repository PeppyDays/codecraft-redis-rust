use crate::command::executor::Command;
use crate::command::executor::CommandExecutor;
use crate::command::executor::CommandExecutorContext;
use crate::command::parser::extract_array;
use crate::command::parser::extract_bulk_string;
use crate::command::parser::validate_array_length;
use crate::command::parser::validate_main_command;
use crate::resp::Value;

#[derive(Debug, Default, PartialEq)]
pub struct Echo {
    message: String,
}

impl Command for Echo {
    fn parse_from(value: &Value) -> Result<Self, anyhow::Error> {
        let array = extract_array(value)?;
        validate_array_length(array, 2)?;
        validate_main_command(array, "ECHO")?;
        let message = extract_bulk_string(array, 1)?;
        Ok(Echo {
            message: message.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl CommandExecutor for Echo {
    async fn execute(&self, _context: CommandExecutorContext) -> Value {
        Value::BulkString(self.message.clone())
    }
}

#[cfg(test)]
mod specs_for_parse_from {
    use fake::Fake;
    use fake::faker::lorem::en::Word;

    use crate::command::executor::Command;
    use crate::resp::Value;

    use super::Echo;

    #[test]
    fn sut_parses_echo_command_correctly() {
        // Arrange
        let message: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString("ECHO".to_string()),
            Value::BulkString(message.to_string()),
        ]);

        // Act
        let actual = Echo::parse_from(&value).unwrap();

        // Assert
        let expected = Echo {
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
        let actual = Echo::parse_from(&value).unwrap();

        // Assert
        let expected = Echo {
            message: message.to_string(),
        };
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod specs_for_execute {
    use std::sync::Arc;

    use fake::Fake;
    use fake::faker::lorem::ar_sa::Word;

    use crate::command::executor::CommandExecutor;
    use crate::command::executor::CommandExecutorContext;
    use crate::repository::Repository;
    use crate::resp::Value;

    use super::Echo;

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
    async fn sut_responds_echo_when_gets_echo_command() {
        // Arrange
        let context = CommandExecutorContext::new(Arc::new(DummyRepository));
        let message = Word().fake::<String>();
        let command = Echo {
            message: message.clone(),
        };

        // Act
        let actual = command.execute(context).await;

        // Assert
        let expected = Value::BulkString(message);
        assert_eq!(actual, expected);
    }
}
