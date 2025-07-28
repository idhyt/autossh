use std::env::{current_exe, home_dir};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};

mod config;
mod db;
mod ssh;

static WORK_DIR: OnceLock<PathBuf> = OnceLock::new();

fn get_work_dir() -> &'static PathBuf {
    WORK_DIR.get_or_init(|| set_work_dir(Option::<&str>::None).expect("WORK_DIR not initialized"))
}

fn set_work_dir(w: Option<impl AsRef<Path>>) -> Result<PathBuf, Error> {
    let work_dir = match w {
        // mean set by other program
        Some(w) => {
            let wd = w.as_ref().to_path_buf();
            // we set WORK_DIR immediately
            WORK_DIR.set(wd.clone()).map_err(|_| {
                Error::new(ErrorKind::AlreadyExists, "WORK_DIR already initialized")
            })?;
            wd
        }
        // mean set by this lib default
        None => {
            if cfg!(test) {
                PathBuf::from("test.atsh.d")
            } else {
                let wd = home_dir()
                    // the system home directory
                    .map(|h| h.join(".atsh.d"))
                    // current executable directory
                    .or_else(|| current_exe().ok().map(|e| e.with_file_name(".atsh.d")))
                    // Error
                    .ok_or_else(|| Error::new(ErrorKind::NotFound, "WORK_DIR not found"))?;
                wd
            }
        }
    };

    if !work_dir.exists() {
        std::fs::create_dir_all(&work_dir)?;
    }

    Ok(work_dir)
}

pub(crate) static WORK_DIR_FILE: LazyLock<fn(&str) -> PathBuf> = LazyLock::new(|| {
    |n| {
        let w = get_work_dir();
        w.join(n)
    }
});

fn setup_logging(work_dir: &Path) -> Result<(), Error> {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::{
        filter::LevelFilter,
        fmt::{
            self,
            time::{LocalTime, UtcTime},
        },
        layer::SubscriberExt,
        util::SubscriberInitExt,
        EnvFilter,
    };

    let log_dir = work_dir.join("logs");
    if !log_dir.is_dir() {
        std::fs::create_dir_all(&log_dir)?;
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let debug = filter
        .max_level_hint()
        .map(|level| level >= LevelFilter::DEBUG)
        .unwrap_or(false);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stderr)
                .with_target(debug)
                .with_line_number(debug)
                .with_timer(LocalTime::rfc_3339()),
        )
        .with(
            fmt::Layer::new()
                .json()
                .with_writer(
                    RollingFileAppender::builder()
                        .rotation(Rotation::DAILY)
                        // .filename_prefix("atsh.log")
                        .filename_suffix("json")
                        .build(log_dir)
                        .expect("Failed to create log file"),
                )
                .with_target(debug)
                .with_line_number(debug)
                .with_timer(UtcTime::rfc_3339()),
        );

    subscriber.init();
    tracing::debug!(
        "Logging initialized (RUST_LOG={})",
        std::env::var("RUST_LOG").unwrap_or_default()
    );
    Ok(())
}

pub mod atsh {
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    use tracing::debug;

    use crate::ssh::remote::Remotes;
    use crate::{get_work_dir, set_work_dir, setup_logging};

    pub use crate::ssh::{remote::Remote, secure::set_atshkey};

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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_work_dir_default() {
        // set default once
        let w = get_work_dir();
        println!("default work dir: {:?}", w);
        // can't set again by others
        let w = set_work_dir(Some(Path::new("test.atsh.d")));
        // println!("set work dir: {:?}", w);
        assert!(w.is_err());
        assert!(w.err().unwrap().to_string().contains("already initialized"));
    }

    #[test]
    #[ignore = "Only for debug"]
    fn test_work_dir_set() {
        let sw = PathBuf::from("test.atsh.d");
        let w = set_work_dir(Some(&sw));
        println!("set work dir: {:?}", w);
        assert!(w.is_ok());
        let w = get_work_dir();
        println!("set work dir: {:?}", w);
        assert_eq!(w.to_owned(), sw);
    }
}
