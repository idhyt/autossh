/*
    example Plugin for run command on remote server:
    autossh plugin add -n "rce-ps" -p "passh" -c "{PLUGIN} -p '{PASSWORD}' ssh -p {PORT} {USER}@{IP} ps -a"
    will run command:
        passh -p "password" ssh -p 22 idhyt@1.2.3.4 ps -a
*/

use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use strfmt::strfmt;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Plugin {
    // /// the index of the plugin.
    // pub index: u16,
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
    pub fn add(&mut self, name: &str, path: &PathBuf, command: &str) -> bool {
        // check the name exist or not
        if self.list.iter().any(|v| v.name == name) {
            log::error!("the plugin name {} already exist", name);
            return false;
        }
        let plugin = Plugin {
            name: name.to_string(),
            path: path.to_path_buf(),
            command: command.to_string(),
        };
        log::debug!("add plugin: {}", plugin);
        self.list.push(plugin);
        true
    }

    pub fn delete(&mut self, name: &str) -> u16 {
        self.list.retain(|v| v.name != name);
        self.list.len() as u16
    }

    pub fn list(&self) {
        let mut table = Table::new();
        let titles = vec!["name", "path", "command"];
        table.set_titles(Row::new(
            titles
                .iter()
                .map(|v| Cell::new(v).style_spec("bcFg"))
                .collect::<Vec<Cell>>(),
        ));
        for plugin in self.list.iter() {
            table.add_row(Row::new(
                vec![
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

    /// get the plugin by name.
    pub fn get(&self, name: &str) -> Option<&Plugin> {
        self.list.iter().find(|v| v.name == name)
    }
}

impl Plugin {
    pub fn run(&self, kwargs: &mut HashMap<String, String>) {
        kwargs.insert("PLUGIN".to_string(), self.path.display().to_string());
        let command = strfmt(&self.command, kwargs).unwrap();
        log::debug!("run plugin with command: {}", command);
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &command]).output().unwrap()
        } else {
            Command::new("sh").arg("-c").arg(&command).output().unwrap()
        };
        if output.status.success() {
            log::info!(
                "run command output: \n{}",
                String::from_utf8_lossy(&output.stdout)
            );
        } else {
            log::error!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }
}
