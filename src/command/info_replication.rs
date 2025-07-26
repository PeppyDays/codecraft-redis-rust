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
        let mut properties = Vec::new();

        if context.config.replication.slave.is_some() {
            properties.push("role:slave".to_string());
        } else {
            properties.push("role:master".to_string());
            properties.push(format!(
                "master_replid:{}",
                context.config.replication.master.id,
            ));
            properties.push(format!(
                "master_repl_offset:{}",
                context.config.replication.master.offset,
            ));
        }

        Value::BulkString(properties.join("\r\n"))
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
    use std::net::Ipv4Addr;
    use std::sync::Arc;

    use crate::command::executor::CommandExecutor;
    use crate::command::executor::CommandExecutorContext;
    use crate::command::info_replication::InfoReplication;
    use crate::config::Config;
    use crate::config::Replication;
    use crate::config::ReplicationMaster;
    use crate::config::ReplicationSlave;
    use crate::repository::fixture::DummyRepository;
    use crate::resp::Value;

    #[rstest::rstest]
    #[case("role:master")]
    #[case("master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb")]
    #[case("master_repl_offset:0")]
    #[tokio::test]
    async fn sut_responds_master_information_of_replication_if_replication_is_not_set(
        #[case] expected: &str,
    ) {
        // Arrange
        let context = CommandExecutorContext {
            repository: Arc::new(DummyRepository),
            config: Arc::new(Config::default()),
        };
        let command = InfoReplication;

        // Act
        let actual = extract_bulk_string(command.execute(context).await).unwrap();

        // Assert
        assert!(actual.contains(expected));
    }

    #[tokio::test]
    async fn sut_responds_replication_role_as_slave_if_replication_is_set() {
        // Arrange
        let config = Config {
            replication: Replication {
                master: ReplicationMaster::default(),
                slave: Some(ReplicationSlave {
                    host: Ipv4Addr::LOCALHOST,
                    port: 6380,
                }),
            },
            ..Default::default()
        };
        let context = CommandExecutorContext {
            repository: Arc::new(DummyRepository),
            config: Arc::new(config),
        };
        let command = InfoReplication;

        // Act
        let actual = extract_bulk_string(command.execute(context).await).unwrap();

        // Assert
        let expected = "role:slave";
        assert!(actual.contains(expected));
    }

    fn extract_bulk_string(value: Value) -> Result<String, anyhow::Error> {
        match value {
            Value::BulkString(str) => Ok(str),
            _ => Err(anyhow::anyhow!("not a bulk string")),
        }
    }
}
