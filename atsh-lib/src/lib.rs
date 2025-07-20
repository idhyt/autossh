use std::path::PathBuf;
use std::sync::LazyLock;

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

pub mod atsh {
    use crate::ssh::remote::Remotes;
    use std::io::{Error, ErrorKind};
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

    pub fn copy(index: usize, path: &str) -> Result<(), Error> {
        let paths = path.split('=').collect::<Vec<&str>>();
        if paths.len() != 2 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "path format error, like `from=to`",
            ));
        }
        let remote = Remotes::get(index)?;
        if std::path::PathBuf::from(paths[0]).exists() {
            remote.upload(paths[0], paths[1])
        } else {
            remote.download(paths[0], paths[1])
        }
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_lib() {
//     }
// }
