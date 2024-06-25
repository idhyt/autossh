use crate::config::Recorder;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod plugin;

pub fn add(name: &str, path: &PathBuf, command: &str) {
    let mut recorder = Recorder::load();
    let _ = recorder.commands.add(name, path, command);
    recorder.save();
    recorder.commands.list();
}

pub fn list() {
    let recorder = Recorder::load();
    recorder.commands.list();
}

pub fn remove(name: &str) {
    let mut recorder = Recorder::load();
    let left = recorder.commands.delete(name);
    log::debug!("remove plugin success, {} index left", left);
    recorder.save();
    recorder.commands.list();
}

pub fn run(index: &u16, name: &str) {
    let recorder = Recorder::load();
    if let Some(remote) = recorder.remotes.get(index) {
        if let Some(plugin) = recorder.commands.get(name) {
            let mut vars = HashMap::from([
                ("NAME".to_string(), {
                    match &remote.name {
                        Some(name) => name.clone(),
                        None => remote.ip.clone(),
                    }
                }),
                ("USER".to_string(), remote.user.clone()),
                ("PASSWORD".to_string(), remote.password.clone()),
                ("IP".to_string(), remote.ip.clone()),
                ("PORT".to_string(), remote.port.to_string()),
            ]);
            plugin.run(&mut vars);
        }
    }
}
