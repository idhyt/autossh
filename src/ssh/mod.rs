mod secure;
pub mod server;
mod session;

use crate::config::Recorder;

pub fn add(
    user: &str,
    password: &str,
    ip: &str,
    port: &u16,
    name: &Option<String>,
    note: &Option<String>,
) {
    let mut recorder = Recorder::load();
    let index = recorder.remotes.add(user, password, ip, port, name, note);
    log::debug!("add remote success, index {}", index);
    recorder.save();
    recorder.remotes.list();
}

pub fn list(all: &bool) {
    let recorder = Recorder::load();
    if *all {
        recorder.remotes.list_all();
    } else {
        recorder.remotes.list();
    }
}

pub fn remove(index: &Vec<u16>) {
    let mut recorder = Recorder::load();
    let left = recorder.remotes.delete(index);
    log::debug!("remove remote success, {} index left", left);
    recorder.save();
    recorder.remotes.list();
}

pub fn login(index: &u16, auth: &bool) {
    if *auth {
        let mut recorder = Recorder::load();
        let remote = recorder.remotes.get_mut(index).unwrap();
        remote.authorized();
        recorder.save();
    }
    Recorder::load().remotes.get(index).unwrap().login();
}

pub fn copy(index: &u16) {
    let recorder = Recorder::load();
    let remote = recorder.remotes.get(index).unwrap();
    log::info!(
        r#"copy command example:
    > scp -P {p} /path/to/local {u}@{i}:/path/to/remote
    > rsync -rvzhP --port={p} /path/to/local {u}@{i}:/path/to/remote
    > passowrd: {pass}
        "#,
        p = &remote.port,
        u = &remote.user,
        i = &remote.ip,
        pass = &remote.password
    );
}

// pub fn authorize(index: &Vec<u16>) {
//     let mut recorder = Recorder::load();
//     for remote in recorder.remotes.list.iter_mut() {
//         if index.contains(&remote.index) {
//             remote.authorized();
//         }
//     }
//     recorder.save();
// }

// pub fn revoke(index: &Vec<u16>) {
//     let mut recorder = Recorder::load();
//     for remote in recorder.remotes.list.iter_mut() {
//         if index.contains(&remote.index) {
//             remote.revoke();
//         }
//     }
//     recorder.save();
// }
