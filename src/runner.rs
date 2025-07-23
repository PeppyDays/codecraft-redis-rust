use futures::stream::StreamExt;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

use crate::command::executor::CommandExecutorContext;
use crate::command::executor::execute;
use crate::command::executor::parse;
use crate::config::Config;
use crate::repository::Repository;
use crate::resp::Value;
use crate::snapshot::RdbFileReader;

pub async fn run(listener: TcpListener, repository: Arc<impl Repository>) {
    let context = CommandExecutorContext::new(repository);

    if let Some(rdb_config) = &Config::global().rdb {
        let path = rdb_config.path();
        if let Ok(file) = File::open(path).await {
            let reader = RdbFileReader::new(file);
            load(reader, context.clone()).await;
        }
    }

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                let context = context.clone();
                tokio::spawn(async move {
                    handle(context, &mut stream).await;
                });
            }
            Err(e) => {
                eprintln!("{e}");
            }
        };
    }
}

async fn load<R: tokio::io::AsyncRead + tokio::io::AsyncSeekExt + Unpin + Send>(
    reader: RdbFileReader<R>,
    context: CommandExecutorContext,
) {
    if let Ok(mut entries) = reader.entries().await {
        while let Some((_, key, value, expiry)) = entries.next().await {
            let now_in_millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            // Skip expired keys
            if let Some(expiry) = expiry {
                if expiry <= now_in_millis {
                    continue;
                }
            }

            let mut v = vec![
                Value::BulkString("SET".to_string()),
                Value::BulkString(key),
                Value::BulkString(value),
            ];
            if let Some(expiry) = expiry {
                v.push(Value::BulkString("PX".to_string()));
                v.push(Value::BulkString((expiry - now_in_millis).to_string()));
            }
            let value = Value::Array(v);
            let command = parse(&value).unwrap();
            execute(command, context.clone()).await;
        }
    }
}

async fn handle(
    context: CommandExecutorContext,
    stream: &mut (impl AsyncReadExt + AsyncWriteExt + Unpin),
) {
    let mut buf = [0; 1024];

    loop {
        let value = read(stream, &mut buf).await;
        if value.is_none() {
            break;
        }
        let value = value.unwrap();

        let command = parse(&value).unwrap();
        let value = execute(command, context.clone()).await;

        write(stream, &value).await;
    }
}

async fn read(stream: &mut (impl AsyncReadExt + Unpin), buf: &mut [u8]) -> Option<Value> {
    let bytes_read = stream.read(buf).await.unwrap();
    if bytes_read == 0 {
        return None;
    }
    Some(Value::from(&buf[..bytes_read]))
}

async fn write(stream: &mut (impl AsyncWriteExt + Unpin), value: &Value) {
    let bytes = value.serialize();
    stream.write_all(&bytes).await.unwrap();
}
