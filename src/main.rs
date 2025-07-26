use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;

use clap::Parser;
use codecrafters_redis::config::ReplicationSlave;
use tokio::net::TcpListener;

use codecrafters_redis::config::Config;
use codecrafters_redis::config::RdbConfig;
use codecrafters_redis::repository::InMemoryRepository;
use codecrafters_redis::runner::run;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Arc::new(Config::from(args));

    let url = format!("{}:{}", Ipv4Addr::LOCALHOST, config.server.port);
    let listener = TcpListener::bind(url).await.unwrap();
    let repository = Arc::new(InMemoryRepository::new());
    run(listener, repository, config).await
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(long = "dir")]
    rdb_directory: Option<String>,

    #[arg(long = "dbfilename")]
    rdb_filename: Option<String>,

    #[arg(long = "port")]
    server_port: Option<usize>,

    #[arg(long = "replicaof")]
    replication_url: Option<String>,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let mut config = Config::default();

        if let Some(server_port) = args.server_port {
            config.server.port = server_port;
        }
        if let Some(replication_url) = args.replication_url {
            let parts: Vec<&str> = replication_url.split(' ').collect();
            if parts.len() == 2 {
                if let Ok(port) = parts[1].parse::<usize>() {
                    config.replication.slave = Some(ReplicationSlave {
                        host: Ipv4Addr::from_str(parts[0]).unwrap(),
                        port,
                    });
                }
            }
        }
        if args.rdb_directory.is_some() && args.rdb_filename.is_some() {
            config.rdb = Some(RdbConfig {
                directory: args.rdb_directory.unwrap(),
                filename: args.rdb_filename.unwrap(),
            });
        };

        config
    }
}
