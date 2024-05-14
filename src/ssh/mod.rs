mod bind;
mod record;
mod secure;

use record::Recorder;

pub fn add(user: &str, password: &str, ip: &str, port: &u16, name: &Option<String>) {
    let mut recorder = Recorder::load();
    let index = recorder.add(user, password, ip, port, name);
    log::debug!("add remote success, index {}", index);
    recorder.list();
}

pub fn list() {
    Recorder::load().list();
}

pub fn remove(index: &u16) {
    let mut recorder = Recorder::load();
    let index = recorder.delete(index);
    log::debug!("remove remote success, index {}", index);
    recorder.list();
}

pub fn login(index: &u16) {
    Recorder::load().login(index);
}
