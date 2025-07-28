use serde::{Deserialize, Serialize};
use std::env::home_dir;
use std::io::{Error, ErrorKind};
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
        let config = WORK_DIR_FILE("config.toml");
        // if config file not exists, create it
        if !config.is_file() {
            let mut default: Config =
                toml::from_str(DEFAULT).expect("Failed to parse default config");
            if default.sshkey.is_none() {
                let home = home_dir().expect("Failed to get home directory");
                let (public, private) = (
                    home.join(".ssh").join("id_rsa.pub"),
                    home.join(".ssh").join("id_rsa"),
                );
                let msg = format!(
                    r#"
  🔹 No SSH key specified. Defaulting to: {:?}
  🔹 SSH key used for remote host authentication.
  🔹 To generate a new key (passphrase-free):
        ssh-keygen -t rsa -b 2048 -C "atsh" -N "" -f {:?}"#,
                    private,
                    WORK_DIR_FILE("atsh_key")
                );
                warn!("💡 The first run to create a default config{}", msg);
                default.sshkey = Some(SSHKey { public, private });
            }
            std::fs::write(
                &config,
                toml::to_string(&default).expect("Failed to serialize config"),
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

    pub fn get_public_key(&self) -> &Path {
        self.sshkey.as_ref().unwrap().public.as_path()
    }

    pub fn get_private_key(&self) -> &Path {
        self.sshkey.as_ref().unwrap().private.as_path()
    }

    pub fn read_public_key(&self) -> Result<String, Error> {
        let key = self.get_public_key();
        if !key.is_file() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "public key not found, you can generate it by `ssh-keygen` and set it to config",
            ));
        }
        std::fs::read_to_string(key)
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
