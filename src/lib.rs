use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

const PONG: &[u8; 7] = b"+PONG\r\n";

pub async fn run(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    handle_requests(stream).await;
                });
            }
            Err(e) => {
                eprintln!("{e}");
            }
        };
    }
}

async fn handle_requests(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buf).await.unwrap();
        if bytes_read == 0 {
            break;
        }
        let cmd = parse(&buf[..bytes_read]).unwrap();
        match cmd {
            RedisCommand::Ping => {
                stream.write_all(PONG).await.unwrap();
            }
            RedisCommand::Echo { message } => {
                let response = format!("${}\r\n{}\r\n", message.len(), message);
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum RedisCommand {
    Ping,
    Echo { message: String },
}

fn parse(buf: &[u8]) -> Option<RedisCommand> {
    let mut parts = split(buf);
    _ = parts.next()?;
    _ = parts.next()?;
    match parts.next()? {
        cmd if cmd.eq_ignore_ascii_case(b"PING") => Some(RedisCommand::Ping),
        cmd if cmd.eq_ignore_ascii_case(b"ECHO") => {
            _ = parts.next()?;
            let message = parts.next()?;
            Some(RedisCommand::Echo {
                message: String::from_utf8_lossy(message).to_string(),
            })
        }
        _ => None,
    }
}

fn split(buf: &[u8]) -> impl Iterator<Item = &[u8]> {
    buf.split(|&b| b == b'\r' || b == b'\n')
        .filter(|part| !part.is_empty())
}

#[cfg(test)]
mod specs_for_parse {
    use fake::Fake;
    use fake::faker::lorem::en::Word;

    use super::RedisCommand;
    use super::parse;

    #[rstest::rstest]
    #[case(b"*1\r\n$4\r\nPING\r\n")]
    #[case(b"*1\r\n$4\r\nPing\r\n")]
    #[case(b"*1\r\n$4\r\nping\r\n")]
    #[case(b"*1\r\n$4\r\nPiNg\r\n")]
    #[case(b"*1\r\n$4\r\npINg\r\n")]
    fn sut_parses_ping_command_with_case_insensitive(#[case] buf: &[u8]) {
        // Act
        let actual = parse(buf);

        // Assert
        let expected = Some(RedisCommand::Ping);
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_echo_command_correctly() {
        // Arrange
        let message: &str = Word().fake();
        let str = format!("*2\r\n$4\r\nECHO\r\n${}\r\n{}\r\n", message.len(), message);
        let buf = str.as_bytes();

        // Act
        let actual = parse(buf);

        // Assert
        let expected = Some(RedisCommand::Echo {
            message: message.to_string(),
        });
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case("ECHO")]
    #[case("echo")]
    #[case("EcHo")]
    fn sut_parses_echo_command_with_case_insensitive(#[case] cmd: &str) {
        // Arrange
        let message: &str = Word().fake();
        let str = format!(
            "*2\r\n$4\r\n{}\r\n${}\r\n{}\r\n",
            cmd,
            message.len(),
            message
        );
        let buf = str.as_bytes();

        // Act
        let actual = parse(buf);

        // Assert
        let expected = Some(RedisCommand::Echo {
            message: message.to_string(),
        });
        assert_eq!(actual, expected);
    }
}
