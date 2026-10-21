use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

use super::key::{create_sshkey, SSHKey};
use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub sshkey: SSHKey,
}

impl Config {
    /// Load config from the work directory, writing a default if missing.
    pub fn load_or_default(work_dir: &Path) -> Result<Self> {
        let file = work_dir.join("config.toml");
        if !file.is_file() {
            let default = Config::default_for(work_dir);
            let toml = toml::to_string(&default).expect("Failed to serialize config");
            std::fs::write(&file, toml)?;
        }
        debug!(file = ?file, "loading config");
        let content = std::fs::read_to_string(&file)?;
        Ok(toml::from_str(&content).expect("Failed to parse config.toml"))
    }

    fn default_for(work_dir: &Path) -> Self {
        Config {
            sshkey: SSHKey::new(
                work_dir.join("id_rsa.pub"),
                work_dir.join("id_rsa"),
            ),
        }
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
            warn!(file = ?private, "ssh key missing, creating one");
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
