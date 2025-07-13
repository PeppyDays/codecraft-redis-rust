#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    Null,
}

impl Value {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Self::SimpleString(s) => format!("+{s}\r\n").into_bytes(),
            Self::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s).into_bytes(),
            Self::Array(arr) => {
                let mut result = format!("*{}\r\n", arr.len()).into_bytes();
                for value in arr {
                    result.extend(value.serialize());
                }
                result
            }
            Self::Null => b"$-1\r\n".to_vec(),
        }
    }

    fn deserialize(buf: &[u8]) -> (Self, &[u8]) {
        match buf[0] {
            b'+' => Self::parse_simple_string(buf),
            b'$' => Self::parse_bulk_string(buf),
            b'*' => Self::parse_array(buf),
            _ => panic!("Invalid RESP format: expected simple or bulk string"),
        }
    }

    fn parse_simple_string(buf: &[u8]) -> (Self, &[u8]) {
        let (word, rest) = Self::split_on_next_crlf(buf.get(1..).unwrap());
        let word = Self::convert_to_string(word);
        (Self::SimpleString(word), rest)
    }

    fn parse_bulk_string(buf: &[u8]) -> (Self, &[u8]) {
        let (size, rest) = Self::split_on_next_crlf(buf.get(1..).unwrap());
        let size = Self::convert_to_usize(size);
        let (word, rest) = Self::split_on_next_crlf(rest);
        let word = Self::convert_to_string(word);
        if word.len() != size {
            panic!(
                "Bulk string size mismatch: expected {}, got {}",
                size,
                word.len()
            );
        }
        (Self::BulkString(word), rest)
    }

    fn parse_array(buf: &[u8]) -> (Self, &[u8]) {
        let (size, mut rest) = Self::split_on_next_crlf(buf.get(1..).unwrap());
        let size = Self::convert_to_usize(size);
        let mut values = Vec::with_capacity(size);

        for _ in 0..size {
            let (value, next_rest) = Self::deserialize(rest);
            values.push(value);
            rest = next_rest;
        }

        (Self::Array(values), rest)
    }

    fn convert_to_usize(buf: &[u8]) -> usize {
        String::from_utf8_lossy(buf).parse::<usize>().unwrap()
    }

    fn convert_to_string(buf: &[u8]) -> String {
        String::from_utf8_lossy(buf).to_string()
    }

    fn split_on_next_crlf(buf: &[u8]) -> (&[u8], &[u8]) {
        for i in 1..buf.len() {
            if buf[i - 1] == b'\r' && buf[i] == b'\n' {
                return (buf.get(0..i - 1).unwrap(), buf.get(i + 1..).unwrap());
            }
        }
        panic!("No CRLF found in buffer");
    }
}

impl From<&[u8]> for Value {
    fn from(buf: &[u8]) -> Self {
        let (value, _) = Self::deserialize(buf);
        value
    }
}

#[cfg(test)]
mod specs_for_from_bytes_to_value {
    use super::Value;

    #[test]
    fn sut_deserialises_simple_string_correctly() {
        // Arrange
        let buf: &[u8] = b"+PING\r\n";

        // Act
        let actual = Value::from(buf);

        // Assert
        let expected = Value::SimpleString("PING".to_string());
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_deserialises_bulk_string_correctly() {
        // Arrange
        let buf: &[u8] = b"$4\r\nECHO\r\n";

        // Act
        let actual = Value::from(buf);

        // Assert
        let expected = Value::BulkString("ECHO".to_string());
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_deserialises_array_correctly() {
        // Arrange
        let buf: &[u8] = b"*2\r\n+PING\r\n$4\r\nECHO\r\n";

        // Act
        let actual = Value::from(buf);

        // Assert
        let expected = Value::Array(vec![
            Value::SimpleString("PING".to_string()),
            Value::BulkString("ECHO".to_string()),
        ]);
        assert_eq!(actual, expected);
    }
}
