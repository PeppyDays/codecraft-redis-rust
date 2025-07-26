use crate::command::executor::Command;
use crate::command::executor::CommandExecutor;
use crate::command::executor::CommandExecutorContext;
use crate::command::parser::extract_array;
use crate::command::parser::extract_bulk_string;
use crate::command::parser::validate_array_length;
use crate::command::parser::validate_main_command;
use crate::command::parser::validate_sub_command;
use crate::resp::Value;

#[derive(Debug, Default, PartialEq)]
pub struct ConfigGet {
    key: String,
}

impl Command for ConfigGet {
    fn parse_from(value: &Value) -> Result<Self, anyhow::Error> {
        let array = extract_array(value)?;
        validate_array_length(array, 3)?;
        validate_main_command(array, "CONFIG")?;
        validate_sub_command(array, "GET")?;

        let key = extract_bulk_string(array, 2)?;
        Ok(ConfigGet {
            key: key.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl CommandExecutor for ConfigGet {
    async fn execute(&self, context: &CommandExecutorContext) -> Value {
        match context.config.get(&self.key) {
            Some(value) => Value::Array(vec![
                Value::BulkString(self.key.to_string()),
                Value::BulkString(value.to_string()),
            ]),
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

    use super::ConfigGet;

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
        let actual = ConfigGet::parse_from(&value).unwrap();

        // Assert
        let expected = ConfigGet {
            key: config_key.to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case("CONFIG", "GET")]
    #[case("config", "get")]
    #[case("CoNfIg", "GeT")]
    fn sut_parses_config_get_command_with_case_insensitive(
        #[case] config: &str,
        #[case] get: &str,
    ) {
        // Arrange
        let config_key: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString(config.to_string()),
            Value::BulkString(get.to_string()),
            Value::BulkString(config_key.to_string()),
        ]);

        // Act
        let actual = ConfigGet::parse_from(&value).unwrap();

        // Assert
        let expected = ConfigGet {
            key: config_key.to_string(),
        };
        assert_eq!(actual, expected);
    }
}
