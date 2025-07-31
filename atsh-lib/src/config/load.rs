use serde::{Deserialize, Serialize};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::{debug, warn};

use super::ctx::{get_work_dir, set_work_dir, WORK_DIR_FILE};
use super::key::{create_sshkey, get_atshkey, set_atshkey, SSHKey};

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::new());

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub sshkey: SSHKey,
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

    /// get ssh public key
    pub fn get_public(&self) -> &Path {
        self.sshkey.get_public()
    }

    /// get ssh private key
    pub fn get_private(&self) -> &Path {
        self.sshkey.get_private()
    }
    /// try get private key, if not exists, create one
    pub fn try_get_private(&self) -> Result<&Path, Error> {
        let private = self.sshkey.get_private();
        if !private.exists() {
            warn!(file = ?private, "💡 SSH key does not exist, will create one");
            create_sshkey(Option::<&str>::None, private, false)?;
        }
        Ok(private)
    }

    /// read ssh public key
    pub fn read_public(&self) -> Result<String, Error> {
        self.sshkey.read_public()
    }

    /// get `ATSH_KEY` which used to encrypt the data like password
    pub fn get_enc_key(&self) -> Result<String, Error> {
        get_atshkey()
    }

    /// set `ATSH_KEY` which used to encrypt the data like password
    pub fn set_enc_key(&self, key: Option<impl AsRef<str>>) -> Result<(), Error> {
        set_atshkey(key)
    }

    /// get work directory
    pub fn get_work_dir(&self) -> &Path {
        &get_work_dir()
    }

    /// set work directory
    pub fn set_work_dir(&self, dir: impl AsRef<Path>) -> Result<(), Error> {
        set_work_dir(dir)
    }

    /// generate work directory file path
    pub fn work_dir_file(&self, n: &str) -> PathBuf {
        WORK_DIR_FILE(n)
    }

    /// create ssh key
    /// `password`: ssh key password, if None, no password
    /// `output`: ssh key output file path, if None, use work directory
    /// interactive: interactive mode, if true, call the `ssh-keygen` interactively
    pub fn create_sshkey(
        &self,
        password: Option<impl AsRef<str>>,
        output: Option<impl AsRef<Path>>,
        interactive: bool,
    ) -> Result<PathBuf, Error> {
        let key = output
            .as_ref()
            .map(|p| p.as_ref())
            .unwrap_or(self.get_private());
        create_sshkey(password, key, interactive)
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_config() {
//         let config = &CONFIG;
//         println!("config: {:#?}", config.sshkey);
//         assert!(config.sshkey.get_public().exists());
//     }
// }
