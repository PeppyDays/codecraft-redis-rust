use crate::command::executor::Command;
use crate::command::executor::CommandExecutor;
use crate::command::executor::CommandExecutorContext;
use crate::command::parser::extract_array;
use crate::command::parser::validate_array_length;
use crate::command::parser::validate_main_command;
use crate::resp::Value;

#[derive(Debug, Default, PartialEq)]
pub struct Ping;

impl Command for Ping {
    fn parse_from(value: &Value) -> Result<Self, anyhow::Error> {
        let array = extract_array(value)?;
        validate_array_length(array, 1)?;
        validate_main_command(array, "PING")?;
        Ok(Ping)
    }
}

#[async_trait::async_trait]
impl CommandExecutor for Ping {
    async fn execute(&self, _context: &CommandExecutorContext) -> Value {
        Value::SimpleString("PONG".to_string())
    }
}

#[cfg(test)]
mod specs_for_parse_from {
    use crate::command::executor::Command;
    use crate::resp::Value;

    use super::Ping;

    #[rstest::rstest]
    #[case("PING")]
    #[case("ping")]
    #[case("PiNg")]
    fn sut_parses_ping_command_with_case_insensitive(#[case] ping: &str) {
        // Arrange
        let value = Value::Array(vec![Value::BulkString(ping.to_string())]);

        // Act
        let actual = Ping::parse_from(&value).unwrap();

        // Assert
        let expected = Ping;
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod specs_for_execute {
    use crate::command::executor::CommandExecutor;
    use crate::command::executor::CommandExecutorContext;
    use crate::command::executor::fixture::command_executor_context;
    use crate::resp::Value;

    use super::Ping;

    #[rstest::rstest]
    #[tokio::test]
    async fn sut_responds_pong_when_gets_ping_command(
        #[from(command_executor_context)] context: CommandExecutorContext,
    ) {
        // Arrange
        let command = Ping;

        // Act
        let actual = command.execute(&context).await;

        // Assert
        let expected = Value::SimpleString("PONG".to_string());
        assert_eq!(actual, expected);
    }
}
