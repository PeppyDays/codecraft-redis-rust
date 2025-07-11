use std::net::TcpListener;

use codecrafters_redis::run;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    run(listener)
}
