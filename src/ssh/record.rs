use directories::UserDirs;
use prettytable::{color, Attr, Cell, Row, Table};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;

use super::secure::{decrypt, encrypt};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Remote {
    /// the index of the remote server.
    pub index: u16,
    /// the login user.
    pub user: String,
    /// the login password.
    #[serde(deserialize_with = "depass", serialize_with = "enpass")]
    pub password: String,
    /// the login id address.
    pub ip: String,
    /// the login port.
    pub port: u16,
    /// the alias name for the login.
    pub name: Option<String>,
}

fn depass<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let password = String::deserialize(deserializer)?;
    Ok(decrypt(&password))
}

fn enpass<S>(password: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Serialize::serialize(&encrypt(password), serializer)
}

// impl display for Remote
impl std::fmt::Display for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ssh {}@{} -p {}", self.user, self.ip, self.port,)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Recorder {
    /// the remote server list.
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

        let content = std::fs::read_to_string(&file).expect("read record file failed");
        let recorder: Recorder = toml::from_str(&content).expect("parse record file failed");
        recorder
    }

    pub fn save(&self) {
        let file = Self::file();
        let content = toml::to_string(&self).unwrap();
        std::fs::write(&file, content).unwrap();
    }

    pub fn pprint(&self) {
        let mut table = Table::new();
        // table.add_row(Row::from(vec![
        //     "index".to_string(),
        //     "name".to_string(),
        //     "user".to_string(),
        //     "ip".to_string(),
        //     "port".to_string(),
        // ]));
        table.set_titles(Row::new(vec![
            Cell::new("index").with_style(Attr::Bold),
            Cell::new("name").with_style(Attr::Bold),
            Cell::new("user").with_style(Attr::Bold),
            Cell::new("ip").with_style(Attr::Bold),
            Cell::new("port").with_style(Attr::Bold),
        ]));
        for remote in self.remotes.iter() {
            // table.add_row(Row::from(vec![
            //     format!("{}", remote.index),
            //     remote.name.clone().unwrap(),
            //     remote.user.clone(),
            //     remote.ip.clone(),
            //     format!("{}", remote.port),
            // ]));
            table.add_row(Row::new(vec![
                Cell::new(&remote.index.to_string())
                    .with_style(Attr::Bold)
                    .with_style(Attr::ForegroundColor(color::BLUE)),
                Cell::new(&remote.name.clone().unwrap_or_else(|| remote.ip.clone()))
                    .with_style(Attr::ForegroundColor(color::BLUE)),
                Cell::new(&remote.user).with_style(Attr::ForegroundColor(color::BLUE)),
                Cell::new(&remote.ip).with_style(Attr::ForegroundColor(color::BLUE)),
                Cell::new(&remote.port.to_string()).with_style(Attr::ForegroundColor(color::BLUE)),
            ]));
        }
        log::debug!("the remote list:\n{}", table);
        table.printstd();
    }
}
