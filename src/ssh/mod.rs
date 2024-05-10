mod record;

use std::io::Write;

use record::{Recorder, Remote};

pub fn add(user: &str, password: &str, ip: &str, port: &u16, name: &Option<String>) {
    log::debug!("add: {:?}, {:?}, {:?}, {:?}", user, ip, port, name);

    let mut recorder = Recorder::load();
    log::debug!("records: {:#?}", recorder);

    let indexs = recorder
        .remotes
        .iter()
        .map(|v| v.index)
        .collect::<Vec<u16>>();
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
    recorder.remotes.push(remote);
    recorder.save();
    recorder.pprint();
    log::info!("add remote success");
}

pub fn list() {
    let recorder = Recorder::load();
    recorder.pprint();
}

pub fn remove(index: &u16) {
    let mut recorder = Recorder::load();
    let index = *index;
    let indexs = recorder
        .remotes
        .iter()
        .map(|v| v.index)
        .collect::<Vec<u16>>();
    if !indexs.contains(&index) {
        log::error!("the index {} not found", index);
        return;
    }
    recorder.remotes.retain(|v| v.index != index);
    recorder.save();
    recorder.pprint();
    log::info!("remove remote success");
}

pub fn login(index: &u16) {
    let recorder = Recorder::load();
    let index = *index;
    let indexs = recorder
        .remotes
        .iter()
        .map(|v| v.index)
        .collect::<Vec<u16>>();
    if !indexs.contains(&index) {
        log::error!("the index {} not found", index);
        return;
    }
    let remote = recorder.remotes.iter().find(|v| v.index == index).unwrap();
    let cmd = format!("ssh {}@{} -p {}", remote.user, remote.ip, remote.port);
    log::info!("login remote: {}", cmd);
    match std::process::Command::new("sh").arg("-c").arg(cmd).spawn() {
        Ok(mut child) => {
            child.stdin.as_ref().unwrap().write(remote.password.as_bytes()).unwrap();
            child.wait().unwrap();
        }
        Err(e) => log::error!("login error: {:#?}", e),
    }
}
