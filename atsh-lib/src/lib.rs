mod config;
mod connection;
mod storage;

pub mod atsh {
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    use tracing::debug;

    use crate::config::{get_work_dir, set_work_dir};
    use crate::connection::remote::Remotes;
    use crate::storage::log::setup_logging;

    pub use crate::config::set_atshkey;
    pub use crate::config::CONFIG;
    pub use crate::connection::remote::Remote;

    type Result<T> = std::result::Result<T, Error>;

    pub fn initialize(work_dir: Option<impl AsRef<Path>>) -> Result<()> {
        set_work_dir(work_dir)?;
        let w = get_work_dir();
        setup_logging(w)?;
        debug!("success initialize at {:?}", w);
        Ok(())
    }

    pub fn add(
        user: &str,
        password: &str,
        ip: &str,
        port: u16,
        name: &Option<impl AsRef<str>>,
        note: &Option<impl AsRef<str>>,
    ) -> Result<usize> {
        Remotes::add(user, password, ip, port, name, note)
    }

    pub fn add_remote(remote: &Remote) -> Result<usize> {
        add(
            &remote.user,
            &remote.password,
            &remote.ip,
            remote.port,
            &remote.name,
            &remote.note,
        )
    }

    pub fn remove(index: &Vec<usize>) -> Result<usize> {
        Remotes::delete(index)
    }

    pub fn get(index: usize) -> Result<Option<Remote>> {
        Remotes::get(index)
    }

    pub fn try_get(index: usize) -> Result<Remote> {
        Remotes::try_get(index)
    }

    pub fn get_all() -> Result<Vec<Remote>> {
        let remotes = Remotes::get_all()?;
        Ok(remotes.0)
    }

    pub fn pprint(all: bool) -> Result<()> {
        let remotes = Remotes::get_all()?;
        remotes.pprint(all);
        Ok(())
    }

    // pub fn list(all: bool) -> Result<()> {
    //     if all {
    //         Remotes::list_all()
    //     } else {
    //         Remotes::list()
    //     }
    // }

    // auth params means try auth against the server
    pub fn login(index: usize, auth: bool) -> Result<()> {
        let remote = Remotes::try_get(index)?;
        remote.login(auth)
    }

    #[deprecated(
        since = "0.1.2",
        note = "This function is not clearly expressed; use `upload/download` instead."
    )]
    pub fn copy(index: usize, path: &str) -> Result<()> {
        let paths = path.split('=').collect::<Vec<&str>>();
        if paths.len() != 2 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "path format error, like `from=to`",
            ));
        }
        let remote = Remotes::try_get(index)?;
        if std::path::PathBuf::from(paths[0]).exists() {
            remote.upload(paths[0], paths[1])
        } else {
            remote.download(paths[0], paths[1])
        }
    }

    pub fn upload(index: usize, path: &Vec<impl AsRef<str>>) -> Result<()> {
        if path.len() != 2 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "path format error, like `upload -p /local/path /remote/path`",
            ));
        }
        let (local, remote) = (path[0].as_ref(), path[1].as_ref());
        if !Path::new(local).exists() {
            return Err(Error::new(ErrorKind::NotFound, "the upload file not found"));
        }

        Remotes::try_get(index)?.upload(local, remote)
    }

    pub fn download(index: usize, path: &Vec<impl AsRef<str>>) -> Result<()> {
        if path.len() != 2 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "path format error, like `upload -p /remote/path /local/path`",
            ));
        }
        let (remote, local) = (path[0].as_ref(), path[1].as_ref());

        Remotes::try_get(index)?.download(remote, local)
    }
}
