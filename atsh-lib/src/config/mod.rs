mod ctx;
mod key;

pub use ctx::{get_work_dir, set_work_dir, WORK_DIR_FILE};
pub use key::{get_atshkey, set_atshkey};

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;
use tracing::debug;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::new());

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub sshkey: key::SSHKey,
}

impl Config {
    pub fn new() -> Self {
        let config = WORK_DIR_FILE("config.toml");
        // if config file not exists, create it
        if !config.is_file() {
            std::fs::write(
                &config,
                toml::to_string(&Config::default()).expect("Failed to serialize config"),
            )
            .expect("Failed to write config.toml");
        }

        Self::load_from_file(config.as_path())
    }

    pub fn load_from_file(f: &Path) -> Self {
        debug!(file = ?f, "Loading config");
        let content = std::fs::read_to_string(f).expect("Failed to read config.toml");
        let config: Config = toml::from_str(&content).expect("Failed to parse config.toml");
        config
    }

    pub fn get_public(&self) -> &Path {
        self.sshkey.get_public()
    }

    pub fn get_private(&self) -> &Path {
        self.sshkey.get_private()
    }

    pub fn read_public(&self) -> Result<String, std::io::Error> {
        self.sshkey.read_public()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_config() {
        let config = &CONFIG;
        println!("config: {:#?}", config.sshkey);
        assert!(config.sshkey.get_public().exists());
    }
}
