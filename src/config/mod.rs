use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ssh::server::Remotes;
use home;

lazy_static::lazy_static! {
    static ref CONFIG: PathBuf = {
        let mut file = home::home_dir().unwrap().join(".config").join("autossh");
        if !file.is_dir() {
            std::fs::create_dir_all(&file).unwrap();
        }
        file.push("config.toml");
        file
    };
    pub static ref SSHKEY: SshKey = {
        let record = Recorder::load();
        if let Some(sshkey) = record.sshkey {
            sshkey
        } else {
            SshKey::default()
        }
    };
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SshKey {
    /// the private key location.
    pub private: PathBuf,
    /// the public key location.
    pub public: PathBuf,
}

impl SshKey {
    fn default() -> Self {
        SshKey {
            private: home::home_dir().unwrap().join(".ssh").join("id_rsa"),
            public: home::home_dir().unwrap().join(".ssh").join("id_rsa.pub"),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Recorder {
    /// the remote server list.
    pub remotes: Remotes,
    /// the ssh key location.
    pub sshkey: Option<SshKey>,
}

impl Recorder {
    pub fn save(&self) {
        let content = toml::to_string(&self).unwrap();
        std::fs::write(&*CONFIG, content).unwrap();
    }

    pub fn load() -> Self {
        let recorder = if CONFIG.is_file() {
            let content = std::fs::read_to_string(&*CONFIG).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Recorder::default()
        };
        recorder
    }
}
