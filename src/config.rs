use std::path::Path;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: usize,
    pub rdb: Option<RdbConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 6379,
            rdb: None,
        }
    }
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
            "port" => Some(self.port.to_string()),
            "dir" => self.rdb.as_ref().map(|rdb| rdb.directory.clone()),
            "dbfilename" => self.rdb.as_ref().map(|rdb| rdb.filename.clone()),
            _ => None,
        }
    }
}
