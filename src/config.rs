use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub server: Server,
    pub replication: Option<Replication>,
    pub rdb: Option<RdbConfig>,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub port: usize,
}

impl Default for Server {
    fn default() -> Self {
        Server { port: 6379 }
    }
}

#[derive(Clone, Debug)]
pub struct Replication {
    pub server_host: String,
    pub server_port: usize,
}

#[derive(Clone, Debug)]
pub struct RdbConfig {
    pub directory: String,
    pub filename: String,
}

impl RdbConfig {
    pub fn path(&self) -> String {
        Path::new(&self.directory)
            .join(&self.filename)
            .to_string_lossy()
            .to_string()
    }
}

impl Config {
    pub fn get(&self, arg: &str) -> Option<String> {
        match arg {
            "port" => Some(self.server.port.to_string()),
            "dir" => self.rdb.as_ref().map(|rdb| rdb.directory.clone()),
            "dbfilename" => self.rdb.as_ref().map(|rdb| rdb.filename.clone()),
            _ => None,
        }
    }
}
