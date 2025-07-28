use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;

pub struct Replicator {
    stream: TcpStream,
}

impl Replicator {
    pub async fn new<A: ToSocketAddrs>(address: A) -> Self {
        let stream = TcpStream::connect(address).await.unwrap();
        Self { stream }
    }

    pub async fn initiate(&mut self) {
        self.ping().await;
    }

    async fn ping(&mut self) {
        let buf = b"*1\r\n$4\r\nPING\r\n";
        self.stream.write_all(buf).await.unwrap();
    }
}
