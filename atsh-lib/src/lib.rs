use home::home_dir;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};

use ssh::remote::Remotes;

mod config;
mod db;
mod ssh;

pub(crate) static WORK_DIR_FILE: LazyLock<fn(&str) -> PathBuf> = LazyLock::new(|| {
    |n| {
        let mut work_dir =
            std::env::current_exe().expect("failed to get current execute directory");
        work_dir.pop();
        work_dir.join(n)
    }
});

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SshKey {
    /// the private key location.
    pub private: PathBuf,
    /// the public key location.
    pub public: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Recorder {
    #[serde(skip)]
    path: PathBuf,
    /// the remote server list.
    pub remotes: Remotes,
    /// the ssh key location.
    pub sshkey: Option<SshKey>,
}

// impl Default for Recorder {
//     fn default() -> Self {
//         Recorder {
//             path: home_dir()
//                 .unwrap()
//                 .join(".config")
//                 .join("autossh")
//                 .join("debug.config.toml"),
//             remotes: Remotes::default(),
//             sshkey: Some(SshKey {
//                 private: home_dir().unwrap().join(".ssh").join("id_rsa"),
//                 public: home_dir().unwrap().join(".ssh").join("id_rsa.pub"),
//             }),
//         }
//     }
// }

impl Recorder {
    pub fn new(path: PathBuf) -> Self {
        Recorder {
            path,
            ..Default::default()
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        let content = toml::to_string(&self).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to parse TOML: {}", e),
            )
        })?;
        if !self.path.parent().unwrap().is_dir() {
            std::fs::create_dir_all(self.path.parent().unwrap())?;
        }
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    fn load_from(&self, path: &Path) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path)?;
        let mut recorder: Recorder = toml::from_str(&content).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to parse TOML: {}", e),
            )
        })?;
        if recorder.sshkey.is_none() {
            recorder.sshkey = Some(SshKey {
                private: home_dir().unwrap().join(".ssh").join("id_rsa"),
                public: home_dir().unwrap().join(".ssh").join("id_rsa.pub"),
            })
        }
        Ok(recorder)
    }

    pub fn load(&self) -> Result<Self, Error> {
        if !self.path.exists() {
            return Ok(Self::default());
        }

        self.load_from(&self.path)
    }
}

static RECORDS: OnceLock<RwLock<Recorder>> = OnceLock::new();

pub fn loading() -> Result<(), Error> {
    RECORDS
        .set(RwLock::new(Recorder::default()))
        .map_err(|_| Error::new(ErrorKind::Other, format!("RECORDS is already loading")))?;
    Ok(())
}

fn get_records() -> RwLockReadGuard<'static, Recorder> {
    RECORDS.get().unwrap().read()
}

fn get_mut_records() -> RwLockWriteGuard<'static, Recorder> {
    RECORDS.get().unwrap().write()
}

pub mod atsh {
    use ssh2::DisconnectCode::Reserved;
    use std::io::Error;

    use crate::ssh::remote::Remotes;
    pub fn add(
        user: &str,
        password: &str,
        ip: &str,
        port: u16,
        name: &Option<String>,
        note: &Option<String>,
    ) -> Result<usize, Error> {
        Remotes::add(user, password, ip, port, name, note)
    }

    pub fn remove(index: &Vec<usize>) -> Result<usize, Error> {
        Remotes::delete(index)
    }

    pub fn list(all: bool) -> Result<(), Error> {
        if all {
            Remotes::list_all()
        } else {
            Remotes::list()
        }
    }

    // auth params means try auth against the server
    pub fn login(index: usize, auth: bool) -> Result<(), Error> {
        let remote = Remotes::get(index)?;
        if !remote.authorized || auth {
            // TODO: auth
        }
        remote.login()
    }
}

pub fn copy(index: &usize, path: &str) {
    let paths = path.split('=').collect::<Vec<&str>>();
    assert!(paths.len() == 2, "path format error, like `from=to`");
    let recorder = get_records();
    // let remote = recorder.remotes.0[index].unwrap();
    // if std::path::PathBuf::from(paths[0]).exists() {
    //     remote.upload(paths[0], paths[1]);
    // } else {
    //     remote.download(paths[0], paths[1]);
    // }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_recorder() {
        let path = PathBuf::from("test.toml");
        let mut recorder = Recorder::new(path.clone());
        println!("{:#?}", recorder);
        assert!(recorder.remotes.0.len() == 0);
        // unsafe {
        //     std::env::set_var("ASKEY", "test");
        // }
        use crate::ssh::remote::Remote;
        let remote = Remote {
            index: 1,
            user: "user".to_string(),
            password: "password".to_string(),
            authorized: false,
            ip: "1.2.3.4".to_string(),
            port: 22222,
            name: Some("name".to_string()),
            note: None,
        };
        recorder.remotes.0.push(remote);
        recorder.save().unwrap();
        let reload = recorder.load().unwrap();
        println!("{:#?}", reload);
        assert!(recorder.remotes.0.len() == 1);
        if path.is_file() {
            std::fs::remove_file(&path).unwrap();
        }
    }
}
