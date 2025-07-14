use std::net::SocketAddr;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct RedisClient {
    stream: Mutex<TcpStream>,
}

impl RedisClient {
    pub async fn new(address: SocketAddr) -> Self {
        let stream = TcpStream::connect(address).await.unwrap();
        Self {
            stream: Mutex::new(stream),
        }
    }

    pub async fn ping(&self) -> String {
        self.stream
            .lock()
            .await
            .write_all(b"*1\r\n$4\r\nPING\r\n")
            .await
            .unwrap();
        self.response().await
    }

    pub async fn echo(&self, message: &str) -> String {
        let str = format!("*2\r\n$4\r\nECHO\r\n${}\r\n{}\r\n", message.len(), message);
        let buf = str.as_bytes();
        self.stream.lock().await.write_all(buf).await.unwrap();
        self.response().await
    }

    pub async fn set(&self, key: &str, value: &str, expires_after: Option<u128>) -> String {
        let cmd_str = if expires_after.is_some() {
            "*5\r\n$3\r\nSET\r\n".to_string()
        } else {
            "*3\r\n$3\r\nSET\r\n".to_string()
        };
        let key_str = format!("${}\r\n{}\r\n", key.len(), key);
        let value_str = format!("${}\r\n{}\r\n", value.len(), value);
        let expires_after_str = if let Some(exp) = expires_after {
            format!("$2\r\npx\r\n${}\r\n{}\r\n", exp.to_string().len(), exp)
        } else {
            String::new()
        };
        let str = format!("{cmd_str}{key_str}{value_str}{expires_after_str}");
        let buf = str.as_bytes();
        self.stream.lock().await.write_all(buf).await.unwrap();
        self.response().await
    }

    pub async fn get(&self, key: &str) -> String {
        let str = format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", key.len(), key);
        let buf = str.as_bytes();
        self.stream.lock().await.write_all(buf).await.unwrap();
        self.response().await
    }

    pub async fn config_get(&self, arg: &str) -> String {
        let str = format!(
            "*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n${}\r\n{}\r\n",
            arg.len(),
            arg
        );
        let buf = str.as_bytes();
        self.stream.lock().await.write_all(buf).await.unwrap();
        self.response().await
    }

    async fn response(&self) -> String {
        let mut buf = [0; 512];
        let bytes_read = self.stream.lock().await.read(&mut buf).await.unwrap();
        String::from_utf8_lossy(&buf[..bytes_read]).to_string()
    }
}
