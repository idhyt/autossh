use home::home_dir;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::{debug, warn};

use crate::WORK_DIR_FILE;

const DEFAULT: &str = include_str!("default.toml");
pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::new());

#[derive(Debug, Default, Serialize, Deserialize)]
struct SSHKey {
    /// the public key location.
    public: PathBuf,
    /// the private key location.
    private: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    sshkey: Option<SSHKey>,
}

impl Config {
    pub fn new() -> Self {
        let config = if cfg!(test) {
            WORK_DIR_FILE("test.config.toml")
        } else {
            WORK_DIR_FILE("config.toml")
        };
        if !config.is_file() {
            std::fs::write(&config, DEFAULT).expect("Failed to write config.toml");
            warn!(
                file = ?config,
                "💡 The first run creates a default config"
            )
        }
        Self::load_from_file(config.as_path())
    }

    pub fn load_from_file(f: &Path) -> Self {
        debug!(file = ?f, "Loading config");
        let content = std::fs::read_to_string(f).expect("Failed to read config.toml");
        let mut config: Config = toml::from_str(&content).expect("Failed to parse config.toml");
        if config.sshkey.is_none() {
            let home = home_dir().unwrap();
            config.sshkey = Some(SSHKey {
                public: home.join(".ssh").join("id_rsa.pub"),
                private: home.join(".ssh").join("id_rsa"),
            });
            warn!(
                "💡 No sshkey specified, using default at {:?}/.ssh/id_rsa",
                home
            );
        }
        config
    }

    pub fn get_public_key(&self) -> &Path {
        self.sshkey.as_ref().unwrap().public.as_path()
    }

    pub fn get_private_key(&self) -> &Path {
         self.sshkey.as_ref().unwrap().private.as_path()
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_config() {
        let config = &CONFIG;
        println!("config: {:#?}", config.sshkey);
        assert!(config.sshkey.is_some());
    }
}
