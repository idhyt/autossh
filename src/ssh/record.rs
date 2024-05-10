use directories::UserDirs;
use prettytable::{Row, Table};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Remote {
    /// the index of the remote server.
    pub index: u16,
    /// the login user.
    pub user: String,
    /// the login password.
    pub password: String,
    /// the login id address.
    pub ip: String,
    /// the login port.
    pub port: u16,
    /// the alias name for the login.
    pub name: Option<String>,
}

// impl display for Remote
impl std::fmt::Display for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ssh {}@{} -p {}", self.user, self.ip, self.port,)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Recorder {
    pub remotes: Vec<Remote>,
}

impl Recorder {
    fn file() -> PathBuf {
        UserDirs::new().unwrap().home_dir().join(".autossh.toml")
    }

    pub fn load() -> Self {
        let file = Self::file();
        if !file.is_file() {
            log::debug!("record file not found in {}", file.display());
            return Self::default();
        }

        let content = std::fs::read_to_string(&file).unwrap();
        let recorder: Recorder = toml::from_str(&content).unwrap();
        recorder
    }

    pub fn save(&self) {
        let file = Self::file();
        let content = toml::to_string(&self).unwrap();
        std::fs::write(&file, content).unwrap();
    }

    pub fn pprint(&self) {
        let mut table = Table::new();
        table.add_row(Row::from(vec![
            "index".to_string(),
            "name".to_string(),
            "user".to_string(),
            "ip".to_string(),
            "port".to_string(),
        ]));
        for remote in self.remotes.iter() {
            table.add_row(Row::from(vec![
                format!("{}", remote.index),
                remote.name.clone().unwrap(),
                remote.user.clone(),
                remote.ip.clone(),
                format!("{}", remote.port),
            ]));
        }
        log::info!("the remote list:\n{}", table);
    }
}
