use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cmd::plugin::Plugins;
use crate::ssh::server::Remotes;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Recorder {
    /// the remote server list.
    pub remotes: Remotes,
    /// the plugin list.
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Plugins,
}

impl Recorder {
    fn file() -> PathBuf {
        let file = UserDirs::new().unwrap().home_dir().join(".autossh.toml");
        if !file.is_file() {
            // log::debug!("record file not found in {}", file.display());
            // create the file if not found.
            let content = toml::to_string(&Self::default()).unwrap();
            std::fs::write(&file, content).unwrap();
            log::debug!("init the record file in `{}`", file.display());
        } else {
            log::debug!("the record data located in `{}`", file.display());
        }
        file
    }

    pub fn save(&self) {
        let file = Self::file();
        let content = toml::to_string(&self).unwrap();
        std::fs::write(&file, content).unwrap();
    }

    pub fn load() -> Self {
        let file = Self::file();
        let content = std::fs::read_to_string(&file).expect("read record file failed");
        let recorder: Recorder = toml::from_str(&content).expect("parse record file failed");
        recorder
    }
}
