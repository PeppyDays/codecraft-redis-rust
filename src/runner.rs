use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

use crate::command::executor::CommandExecutorContext;
use crate::command::executor::execute;
use crate::command::executor::parse;
use crate::config::Config;
use crate::repository::Repository;
use crate::resp::Value;
use crate::snapshot::RdbFileReader;

pub async fn run(listener: TcpListener, repository: Arc<impl Repository>, config: Arc<Config>) {
    let context = CommandExecutorContext::new(repository, config.clone());

    if let Some(rdb_config) = &config.rdb {
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

async fn load<R: AsyncRead + AsyncSeekExt + Unpin + Send>(
    reader: RdbFileReader<R>,
    context: CommandExecutorContext,
) {
    let mut entries = reader.entries().await;
    while let Some(entry) = entries.next().await {
        if let Some(expiry) = &entry.expiry {
            if expiry.is_expired() {
                continue;
            }
        }

        context.repository.set(entry).await;
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
