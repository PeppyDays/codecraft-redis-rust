use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub server: Server,
    pub replication: Replication,
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

#[derive(Clone, Debug, Default)]
pub struct Replication {
    pub master: ReplicationMaster,
    pub slave: Option<ReplicationSlave>,
}

impl Replication {
    pub fn is_master(&self) -> bool {
        self.slave.is_none()
    }

    pub fn is_slave(&self) -> bool {
        self.slave.is_some()
    }
}

#[derive(Clone, Debug)]
pub struct ReplicationMaster {
    pub id: String,
    pub offset: usize,
}

impl Default for ReplicationMaster {
    fn default() -> Self {
        Self {
            id: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
            offset: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReplicationSlave {
    pub master_address: String,
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
