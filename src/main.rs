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
    let config = Config::from(args);
    Config::initialize(&config);

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let repository = Arc::new(InMemoryRepository::new());
    run(listener, repository).await
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(long = "dir")]
    rdb_directory: Option<String>,

    #[arg(long = "dbfilename")]
    rdb_filename: Option<String>,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        if args.rdb_directory.is_none() && args.rdb_filename.is_none() {
            return Config { rdb: None };
        }
        Config {
            rdb: Some(RdbConfig {
                directory: args.rdb_directory.unwrap(),
                filename: args.rdb_filename.unwrap(),
            }),
        }
    }
}
