use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::{debug, warn};

use super::ctx::{get_work_dir, set_work_dir, WORK_DIR_FILE};
use super::key::{create_sshkey, get_atshkey, set_atshkey, SSHKey};
use crate::error::Result;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::new());

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub sshkey: SSHKey,
}

impl Config {
    pub fn new() -> Self {
        let config = WORK_DIR_FILE("config.toml");
        if !config.is_file() {
            let default_toml =
                toml::to_string(&Config::default()).expect("Failed to serialize config");
            std::fs::write(&config, default_toml).expect("Failed to write config.toml");
        }
        Self::load_from_file(config.as_path())
    }

    pub fn load_from_file(f: &Path) -> Self {
        debug!(file = ?f, "Loading config");
        let content = std::fs::read_to_string(f).expect("Failed to read config.toml");
        toml::from_str(&content).expect("Failed to parse config.toml")
    }

    pub fn get_public(&self) -> &Path {
        self.sshkey.get_public()
    }

    pub fn get_private(&self) -> &Path {
        self.sshkey.get_private()
    }

    pub fn try_get_private(&self) -> Result<&Path> {
        let private = self.sshkey.get_private();
        if !private.exists() {
            warn!(file = ?private, "SSH key does not exist, will create one");
            create_sshkey(Option::<&str>::None, private, false)?;
        }
        Ok(private)
    }

    pub fn read_public(&self) -> Result<String> {
        self.sshkey.read_public()
    }

    pub fn read_private(&self) -> Result<String> {
        self.sshkey.read_private()
    }

    pub fn get_enc_key(&self) -> Result<String> {
        get_atshkey()
    }

    pub fn set_enc_key(&self, key: Option<impl AsRef<str>>) -> Result<()> {
        set_atshkey(key)
    }

    pub fn get_work_dir(&self) -> &Path {
        get_work_dir()
    }

    pub fn set_work_dir(&self, dir: impl AsRef<Path>) -> Result<()> {
        set_work_dir(dir)
    }

    pub fn work_dir_file(&self, n: &str) -> PathBuf {
        WORK_DIR_FILE(n)
    }

    pub fn create_sshkey(
        &self,
        password: Option<impl AsRef<str>>,
        output: Option<impl AsRef<Path>>,
        interactive: bool,
    ) -> Result<PathBuf> {
        let key = output
            .as_ref()
            .map(|p| p.as_ref())
            .unwrap_or_else(|| self.get_private());
        create_sshkey(password, key, interactive)
    }
}