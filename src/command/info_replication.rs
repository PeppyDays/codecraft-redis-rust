use crate::command::executor::Command;
use crate::command::executor::CommandExecutor;
use crate::command::executor::CommandExecutorContext;
use crate::command::parser::extract_array;
use crate::command::parser::validate_array_length;
use crate::command::parser::validate_main_command;
use crate::command::parser::validate_sub_command;
use crate::resp::Value;

#[derive(Debug, Default, PartialEq)]
pub struct InfoReplication;

impl Command for InfoReplication {
    fn parse_from(value: &Value) -> Result<Self, anyhow::Error> {
        let array = extract_array(value)?;
        validate_array_length(array, 2)?;
        validate_main_command(array, "INFO")?;
        validate_sub_command(array, "replication")?;
        Ok(InfoReplication)
    }
}

#[async_trait::async_trait]
impl CommandExecutor for InfoReplication {
    async fn execute(&self, context: CommandExecutorContext) -> Value {
        if context.config.replication.is_some() {
            return Value::BulkString("role:slave".to_string());
        }
        Value::BulkString("role:master".to_string())
    }
}

#[cfg(test)]
mod specs_for_parse_from {
    use crate::command::executor::Command;
    use crate::command::info_replication::InfoReplication;
    use crate::resp::Value;

    #[test]
    fn sut_parses_info_replication_command_correctly() {
        // Arrange
        let value = Value::Array(vec![
            Value::BulkString("INFO".to_string()),
            Value::BulkString("replication".to_string()),
        ]);

        // Act
        let actual = InfoReplication::parse_from(&value).unwrap();

        // Assert
        let expected = InfoReplication;
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_raises_error_if_main_command_is_not_info() {
        // Arrange
        let value = Value::Array(vec![
            Value::BulkString("INFU".to_string()),
            Value::BulkString("replication".to_string()),
        ]);

        // Act
        let actual = InfoReplication::parse_from(&value);

        // Assert
        assert!(actual.is_err());
    }
}

#[cfg(test)]
mod specs_for_execute {
    use std::sync::Arc;

    use crate::command::executor::CommandExecutor;
    use crate::command::executor::CommandExecutorContext;
    use crate::command::info_replication::InfoReplication;
    use crate::config::Config;
    use crate::repository::fixture::DummyRepository;
    use crate::resp::Value;

    #[tokio::test]
    async fn sut_responds_replication_role_as_master_if_replication_is_not_set() {
        // Arrange
        let context = CommandExecutorContext {
            repository: Arc::new(DummyRepository),
            config: Arc::new(Config::default()),
        };
        let command = InfoReplication;

        // Act
        let actual = command.execute(context).await;

        // Assert
        let expected = Value::BulkString("role:master".to_string());
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_replication_role_as_slave_if_replication_is_set() {
        // Arrange
        let config = Config {
            replication: Some(crate::config::Replication {
                server_host: "localhost".to_string(),
                server_port: 6380,
            }),
            ..Default::default()
        };
        let context = CommandExecutorContext {
            repository: Arc::new(DummyRepository),
            config: Arc::new(config),
        };
        let command = InfoReplication;

        // Act
        let actual = command.execute(context).await;

        // Assert
        let expected = Value::BulkString("role:slave".to_string());
        assert_eq!(actual, expected);
    }
}
