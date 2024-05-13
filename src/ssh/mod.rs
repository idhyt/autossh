mod bind;
mod record;
mod secure;

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
    log::debug!("add remote success");
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
    log::debug!("remove remote success");
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
    log::debug!("login remote: {}", cmd);

    unsafe {
        // passh -c 10 -p password ssh -p port user@ip
        let argv = vec![
            std::ffi::CString::new("passh").unwrap().into_raw(),
            std::ffi::CString::new("-c").unwrap().into_raw(),
            std::ffi::CString::new("10").unwrap().into_raw(),
            std::ffi::CString::new("-p").unwrap().into_raw(),
            std::ffi::CString::new(remote.password.clone())
                .unwrap()
                .into_raw(),
            std::ffi::CString::new("ssh").unwrap().into_raw(),
            std::ffi::CString::new("-p").unwrap().into_raw(),
            std::ffi::CString::new(remote.port.to_string())
                .unwrap()
                .into_raw(),
            std::ffi::CString::new(format!("{}@{}", remote.user, remote.ip))
                .unwrap()
                .into_raw(),
        ];
        let argc = argv.len() as i32;
        bind::passh(argc, argv.as_ptr() as *mut *mut std::os::raw::c_char);
    };
}
