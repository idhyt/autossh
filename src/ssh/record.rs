use directories::UserDirs;
use prettytable::{color, Attr, Cell, Row, Table};
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
        let mut titles = vec![
            Cell::new("index").with_style(Attr::Bold),
            Cell::new("name").with_style(Attr::Bold),
            Cell::new("user").with_style(Attr::Bold),
            Cell::new("ip").with_style(Attr::Bold),
            Cell::new("port").with_style(Attr::Bold),
        ];
        if all {
            titles.push(Cell::new("password").with_style(Attr::Bold));
        }
        // for title in titles.iter_mut() {
        //     title.align(Alignment::CENTER);
        // }
        table.set_titles(Row::new(titles));
        for remote in self.remotes.iter() {
            let mut row = vec![
                Cell::new(&remote.index.to_string())
                    .with_style(Attr::Bold)
                    .with_style(Attr::ForegroundColor(color::BLUE)),
                Cell::new(&remote.name.clone().unwrap_or_else(|| remote.ip.clone()))
                    .with_style(Attr::ForegroundColor(color::BLUE)),
                Cell::new(&remote.user).with_style(Attr::ForegroundColor(color::BLUE)),
                Cell::new(&remote.ip).with_style(Attr::ForegroundColor(color::BLUE)),
                Cell::new(&remote.port.to_string()).with_style(Attr::ForegroundColor(color::BLUE)),
            ];
            if all {
                row.push(
                    Cell::new(&remote.password).with_style(Attr::ForegroundColor(color::BLUE)),
                );
            }
            table.add_row(Row::new(row));
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
        self.pprint(true);
    }

    pub fn add(
        &mut self,
        user: &str,
        password: &str,
        ip: &str,
        port: &u16,
        name: &Option<String>,
    ) -> u16 {
        let indexs = self.remotes.iter().map(|v| v.index).collect::<Vec<u16>>();
        let index = indexs.iter().max().unwrap_or(&0) + 1;
        let remote = Remote {
            index,
            user: user.to_string(),
            password: password.to_string(),
            ip: ip.to_string(),
            port: *port,
            name: if let Some(name) = name {
                Some(name.to_string())
            } else {
                Some(ip.to_string())
            },
        };
        log::debug!("add remote: {}", remote);
        self.remotes.push(remote);
        self.save();
        index
    }

    pub fn delete(&mut self, index: &u16) -> u16 {
        let index = *index;
        self.remotes.retain(|v| v.index != index);
        self.save();
        index
    }
}
