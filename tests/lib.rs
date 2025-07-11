use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::net::{SocketAddr, TcpListener};
use std::thread;

use codecrafters_redis::run;

#[test]
fn sut_responds_pong_when_gets_ping() {
    // Arrange
    let server = RedisServer::new();
    let mut stream = TcpStream::connect(server.address).unwrap();

    // Act
    server.ping(&mut stream);

    // Assert
    let actual = get_response(&mut stream);
    assert_eq!(actual, "+PONG\r\n");
}

#[test]
fn sut_responds_pongs_whenever_gets_pings() {
    // Arrange
    let server = RedisServer::new();
    let mut stream = TcpStream::connect(server.address).unwrap();

    // Act
    for _ in 0..10 {
        server.ping(&mut stream);
        let actual = get_response(&mut stream);
        assert_eq!(actual, "+PONG\r\n");
    }
}

struct RedisServer {
    address: SocketAddr,
}

impl RedisServer {
    fn new() -> Self {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        thread::spawn(|| run(listener));
        Self { address }
    }
}

impl RedisServer {
    fn ping(&self, stream: &mut TcpStream) {
        stream.write_all(b"*1\r\n$4\r\nPING\r\n").unwrap();
    }
}

fn get_response(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
}
