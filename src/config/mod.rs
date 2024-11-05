use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ssh::server::Remotes;
use home;

lazy_static::lazy_static! {
    static ref CONFIG: PathBuf = home::home_dir().unwrap().join(".autossh.toml");
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SshKey {
    /// the private key location.
    pub private: PathBuf,
    /// the public key location.
    pub public: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Recorder {
    /// the remote server list.
    pub remotes: Remotes,
    /// the ssh key location.
    pub key: Option<SshKey>,
}

impl Recorder {
    pub fn save(&self) {
        let content = toml::to_string(&self).unwrap();
        std::fs::write(&*CONFIG, content).unwrap();
    }

    pub fn load() -> Self {
        let content = std::fs::read_to_string(&*CONFIG).unwrap();
        let recorder: Recorder = toml::from_str(&content).unwrap();
        recorder
    }
}
