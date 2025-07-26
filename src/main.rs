use std::net::Ipv4Addr;
use std::sync::Arc;

use clap::Parser;
use tokio::net::TcpListener;

use codecrafters_redis::config::Config;
use codecrafters_redis::config::RdbConfig;
use codecrafters_redis::repository::InMemoryRepository;
use codecrafters_redis::runner::run;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Arc::new(Config::from(args));

    let url = format!("{}:{}", Ipv4Addr::LOCALHOST, config.port);
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
    port: Option<usize>,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let mut config = Config::default();

        if let Some(port) = args.port {
            config.port = port;
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
