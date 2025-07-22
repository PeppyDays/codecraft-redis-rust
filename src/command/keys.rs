use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::command::executor::Command;
use crate::command::executor::CommandExecutor;
use crate::command::executor::CommandExecutorContext;
use crate::command::parser::extract_array;
use crate::command::parser::extract_bulk_string;
use crate::command::parser::validate_array_length;
use crate::command::parser::validate_main_command;
use crate::resp::Value;

#[derive(Debug, Default, PartialEq)]
pub struct Keys {
    pattern: String,
}

impl Command for Keys {
    fn parse_from(value: &Value) -> Result<Self, anyhow::Error> {
        let array = extract_array(value)?;
        validate_array_length(array, 2)?;
        validate_main_command(array, "KEYS")?;
        let pattern = extract_bulk_string(array, 1)?;
        Ok(Keys {
            pattern: pattern.trim_matches('"').to_string(),
        })
    }
}

impl Keys {
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
}

#[async_trait::async_trait]
impl CommandExecutor for Keys {
    async fn execute(&self, context: CommandExecutorContext) -> Value {
        let entries = context.repository.entries().await;
        let now_in_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let matched_entries = entries
            .into_iter()
            .filter(|(key, (_, expiry))| {
                Keys::match_asterisk_pattern(&self.pattern, key)
                    && (expiry.is_none() || (expiry.is_some() && expiry.unwrap() >= now_in_millis))
            })
            .map(|(key, _)| Value::BulkString(key))
            .collect();
        Value::Array(matched_entries)
    }
}

#[cfg(test)]
mod specs_for_parse_from {
    use fake::Fake;
    use fake::faker::lorem::en::Word;

    use crate::command::executor::Command;
    use crate::resp::Value;

    use super::Keys;

    #[test]
    fn sut_parses_keys_command_correctly() {
        // Arrange
        let pattern: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString("KEYS".to_string()),
            Value::BulkString(pattern.to_string()),
        ]);

        // Act
        let actual = Keys::parse_from(&value).unwrap();

        // Assert
        let expected = Keys {
            pattern: pattern.to_string(),
        };
        assert_eq!(actual.pattern, expected.pattern);
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
        let actual = Keys::parse_from(&value).unwrap();

        // Assert
        let expected = Keys {
            pattern: pattern.to_string(),
        };
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case("KEYS")]
    #[case("keys")]
    #[case("KeYs")]
    fn sut_parses_keys_command_with_case_insensitive(#[case] keys: &str) {
        // Arrange
        let pattern: &str = Word().fake();
        let value = Value::Array(vec![
            Value::BulkString(keys.to_string()),
            Value::BulkString(pattern.to_string()),
        ]);

        // Act
        let actual = Keys::parse_from(&value).unwrap();

        // Assert
        let expected = Keys {
            pattern: pattern.to_string(),
        };
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod specs_for_execute {
    use std::sync::Arc;
    use std::time::Duration;

    use fake::Fake;
    use fake::faker::internet::en::Password;
    use fake::faker::lorem::en::Word;
    use tokio::time::sleep;

    use crate::command::executor::CommandExecutor;
    use crate::command::executor::CommandExecutorContext;
    use crate::repository::InMemoryRepository;
    use crate::repository::Repository;
    use crate::resp::Value;

    use super::Keys;

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

    #[tokio::test]
    async fn sut_responds_all_keys_when_keys_command_pattern_is_asterisk() {
        // Arrange
        let repository = Arc::new(InMemoryRepository::new());
        let context = CommandExecutorContext::new(repository.clone());
        let n = (3..=10).fake::<usize>();
        let keys: Vec<String> = (0..n).map(|_| Password(32..33).fake()).collect();
        for key in keys.iter() {
            repository
                .set(key, &Password(32..33).fake::<String>(), None)
                .await;
        }
        let cmd = Keys {
            pattern: "*".to_string(),
        };

        // Act
        let actual = cmd.execute(context).await;

        // Assert
        let expected = Value::Array(keys.into_iter().map(Value::BulkString).collect());
        assert_eq!(sort_value_array(&actual), sort_value_array(&expected));
    }

    #[tokio::test]
    async fn sut_responds_the_given_key_when_keys_command_pattern_is_exactly_the_key() {
        // Arrange
        let repository = Arc::new(InMemoryRepository::new());
        let context = CommandExecutorContext::new(repository.clone());
        let n = (3..=10).fake::<usize>();
        let keys: Vec<String> = (0..n).map(|_| Password(32..33).fake()).collect();
        for key in keys.iter() {
            repository.set(key, &Word().fake::<String>(), None).await;
        }
        let first_key = keys.first().unwrap();
        let cmd = Keys {
            pattern: first_key.to_string(),
        };

        // Act
        let actual = cmd.execute(context).await;

        // Assert
        let expected = Value::Array(vec![Value::BulkString(first_key.to_string())]);
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[tokio::test]
    #[case("h*", vec!["healingpaper"])]
    async fn sut_responds_the_matched_keys_as_asterisk_to_whatever(
        #[case] pattern: &str,
        #[case] _expected_keys: Vec<&str>,
    ) {
        // Arrange
        let repository = Arc::new(InMemoryRepository::new());
        let context = CommandExecutorContext::new(repository.clone());
        let keys: Vec<&str> = vec!["healingpaper", "arine"];
        for key in keys.iter() {
            repository
                .set(key, &Password(32..33).fake::<String>(), None)
                .await;
        }
        let cmd = Keys {
            pattern: pattern.to_string(),
        };

        // Act
        let actual = cmd.execute(context).await;

        // Assert
        let expected = Value::Array(vec![Value::BulkString("healingpaper".to_string())]);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn sut_responds_with_skipping_expired_keys() {
        // Arrange
        let repository = Arc::new(InMemoryRepository::new());
        let context = CommandExecutorContext::new(repository.clone());
        repository.set(Word().fake(), Word().fake(), Some(0)).await;
        let cmd = Keys {
            pattern: "*".to_string(),
        };

        // Act
        sleep(Duration::from_millis(10)).await;
        let actual = cmd.execute(context).await;

        // Assert
        let expected = Value::Array(vec![]);
        assert_eq!(actual, expected);
    }
}
