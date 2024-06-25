use crate::config::Recorder;
use std::path::PathBuf;

pub mod plugin;

pub fn add(name: &str, path: &PathBuf, command: &str) {
    let mut recorder = Recorder::load();
    let index = recorder.commands.add(name, path, command);
    log::debug!("add plugin success, index {}", index);
    recorder.save();
    recorder.commands.list();
}

pub fn list() {
    let recorder = Recorder::load();
    recorder.commands.list();
}

pub fn remove(index: &Vec<u16>) {
    let mut recorder = Recorder::load();
    let left = recorder.commands.delete(index);
    log::debug!("remove plugin success, {} index left", left);
    recorder.save();
    recorder.commands.list();
}
