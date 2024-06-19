use directories::UserDirs;
use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::server::Remote;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Recorder {
    /// the remote server list.
    remotes: Vec<Remote>,
}

impl Recorder {
    fn file() -> PathBuf {
        UserDirs::new().unwrap().home_dir().join(".autossh.toml")
    }

    fn pprint(&self, all: bool) {
        let mut table = Table::new();
        let mut titles = vec!["index", "name", "user", "ip", "port"];
        if all {
            titles.push("password");
            titles.push("note");
        }
        table.set_titles(Row::new(
            titles
                .iter()
                .map(|v| Cell::new(v).style_spec("bcFg"))
                .collect::<Vec<Cell>>(),
        ));

        for remote in self.remotes.iter() {
            let mut row = vec![
                remote.index.to_string(),
                remote.name.clone().unwrap_or_else(|| "".to_string()),
                remote.user.clone(),
                remote.ip.clone(),
                remote.port.to_string(),
            ];
            if all {
                row.push(remote.password.clone());
                row.push(remote.note.clone().unwrap_or_else(|| "".to_string()));
            }
            table.add_row(Row::new(
                row.iter()
                    .map(|v| Cell::new(v).style_spec("lFc"))
                    .collect::<Vec<Cell>>(),
            ));
        }
        log::debug!("the remote list:\n{}", table);
        table.printstd();
    }

    fn save(&self) {
        let file = Self::file();
        let content = toml::to_string(&self).unwrap();
        std::fs::write(&file, content).unwrap();
    }

    pub fn get(&self, index: &u16) -> Option<&Remote> {
        let index = *index;
        // let indexs = self
        //     .remotes
        //     .iter()
        //     .map(|v| v.index)
        //     .collect::<Vec<u16>>();
        // if !indexs.contains(&index) {
        //     log::error!("the index {} not found", index);
        // }
        for remote in self.remotes.iter() {
            if remote.index == index {
                return Some(remote);
            }
        }
        log::error!("the index {} not found", index);
        None
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

    pub fn list(&self) {
        self.pprint(false);
    }

    pub fn list_all(&self) {
        log::info!("the record data located in `{}`", Self::file().display());
        self.pprint(true);
    }

    pub fn add(
        &mut self,
        user: &str,
        password: &str,
        ip: &str,
        port: &u16,
        name: &Option<String>,
        note: &Option<String>,
    ) -> u16 {
        let indexs = self.remotes.iter().map(|v| v.index).collect::<Vec<u16>>();
        let index = indexs.iter().max().unwrap_or(&0) + 1;
        let remote = Remote {
            index,
            user: user.to_string(),
            password: password.to_string(),
            ip: ip.to_string(),
            port: *port,
            name: name.clone(),
            note: note.clone(),
        };
        log::debug!("add remote: {}", remote);
        self.remotes.push(remote);
        self.save();
        index
    }

    pub fn delete(&mut self, index: &Vec<u16>) -> bool {
        // let index = *index;
        // self.remotes.retain(|v| v.index != index);
        self.remotes.retain(|v| !index.contains(&v.index));
        self.save();
        // index
        true
    }
}
