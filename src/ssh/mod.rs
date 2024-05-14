mod bind;
mod record;
mod secure;
mod server;

use record::Recorder;

pub fn add(user: &str, password: &str, ip: &str, port: &u16, name: &Option<String>) {
    let mut recorder = Recorder::load();
    let index = recorder.add(user, password, ip, port, name);
    log::debug!("add remote success, index {}", index);
    recorder.list();
}

pub fn list(all: &bool) {
    let recorder = Recorder::load();
    if *all {
        recorder.list_all();
    } else {
        recorder.list();
    }
}

pub fn remove(index: &u16) {
    let mut recorder = Recorder::load();
    let index = recorder.delete(index);
    log::debug!("remove remote success, index {}", index);
    recorder.list();
}

pub fn login(index: &u16) {
    Recorder::load().get(index).unwrap().login();
}
