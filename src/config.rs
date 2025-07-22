use std::path::Path;
use std::sync::RwLock;

static GLOBAL_CONFIG: RwLock<Option<Config>> = RwLock::new(None);

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub rdb: Option<RdbConfig>,
}

#[derive(Clone, Debug, Default)]
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
    pub fn initialize(self) {
        let mut config = GLOBAL_CONFIG.write().unwrap();
        *config = Some(self);
    }

    pub fn global() -> Config {
        GLOBAL_CONFIG.read().unwrap().clone().expect("Config not initialized")
    }

    pub fn get(&self, arg: &str) -> Option<&str> {
        match arg {
            "dir" => self.rdb.as_ref().map(|rdb| rdb.directory.as_str()),
            "dbfilename" => self.rdb.as_ref().map(|rdb| rdb.filename.as_str()),
            _ => None,
        }
    }
}
