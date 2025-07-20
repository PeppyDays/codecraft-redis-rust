use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

use crate::command::executor::CommandExecutorContext;
use crate::command::executor::execute;
use crate::command::executor::parse;
use crate::repository::Repository;
use crate::resp::Value;

pub async fn run(listener: TcpListener, repository: Arc<impl Repository>) {
    let context = CommandExecutorContext::new(repository);

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
