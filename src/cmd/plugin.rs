use prettytable::{Cell, Row, Table};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Plugin {
    /// the index of the plugin.
    pub index: u16,
    /// the plugin name.
    pub name: String,
    /// the plugin executable file path.
    pub path: PathBuf,
    /// the plugin command.
    pub command: String,
}

// impl display for Plugin
impl std::fmt::Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} at {} with command: {}",
            self.name,
            self.path.display(),
            self.command
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Plugins {
    /// the plugin list.
    list: Vec<Plugin>,
}

impl Plugins {
    pub fn add(&mut self, name: &str, path: &PathBuf, command: &str) -> u16 {
        let indexs = self.list.iter().map(|v| v.index).collect::<Vec<u16>>();
        let index = indexs.iter().max().unwrap_or(&0) + 1;
        let plugin = Plugin {
            index,
            name: name.to_string(),
            path: path.to_path_buf(),
            command: command.to_string(),
        };
        log::debug!("add plugin: {}", plugin);
        self.list.push(plugin);
        index
    }

    pub fn delete(&mut self, index: &Vec<u16>) -> u16 {
        self.list.retain(|v| !index.contains(&v.index));
        self.list.len() as u16
    }

    pub fn list(&self) {
        let mut table = Table::new();
        let titles = vec!["index", "name", "path", "command"];
        table.set_titles(Row::new(
            titles
                .iter()
                .map(|v| Cell::new(v).style_spec("bcFg"))
                .collect::<Vec<Cell>>(),
        ));
        for plugin in self.list.iter() {
            table.add_row(Row::new(
                vec![
                    plugin.index.to_string(),
                    plugin.name.clone(),
                    plugin.path.display().to_string(),
                    plugin.command.clone(),
                ]
                .iter()
                .map(|v| Cell::new(v).style_spec("lFc"))
                .collect::<Vec<Cell>>(),
            ));
        }
        log::debug!("the plugin list:\n{:#?}", self.list);
        table.printstd();
    }
}
