use std::path::Path;
use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

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
    pub fn initialize(config: &Config) {
        GLOBAL_CONFIG
            .set(config.clone())
            .expect("Config already initialized");
    }

    pub fn global() -> &'static Config {
        GLOBAL_CONFIG.get().expect("Config not initialized")
    }

    pub fn get(&self, arg: &str) -> Option<&str> {
        match arg {
            "dir" => self.rdb.as_ref().map(|rdb| rdb.directory.as_str()),
            "dbfilename" => self.rdb.as_ref().map(|rdb| rdb.filename.as_str()),
            _ => None,
        }
    }
}
