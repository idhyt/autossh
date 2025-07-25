use std::io::Error;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};

mod config;
mod db;
mod ssh;

static WORK_DIR: OnceLock<PathBuf> = OnceLock::new();
pub(crate) static WORK_DIR_FILE: LazyLock<fn(&str) -> PathBuf> = LazyLock::new(|| {
    |n| {
        if cfg!(test) {
            let work_dir =
                std::env::current_exe().expect("failed to get current execute directory");
            work_dir.with_file_name("test.atsh.d")
        } else {
            let work_dir = WORK_DIR.get().expect("WORK_DIR not initialized");
            work_dir.join(n)
        }
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
    use crate::{setup_logging, WORK_DIR};

    pub use crate::ssh::{remote::Remote, secure::set_atshkey};

    type Result<T> = std::result::Result<T, Error>;

    pub fn initialize(work_dir: Option<&Path>) -> Result<()> {
        let work_dir = work_dir.map(|p| p.to_path_buf()).unwrap_or({
            let work_dir =
                std::env::current_exe().expect("failed to get current execute directory");
            // work_dir.pop();
            // work_dir
            work_dir.with_file_name(".atsh.d")
        });
        debug!("work_dir: {}", work_dir.display());

        if !work_dir.exists() {
            std::fs::create_dir_all(&work_dir)?;
        }
        setup_logging(&work_dir)?;

        WORK_DIR
            .set(work_dir)
            .expect("WORK_DIR already initialized");

        // info!(work=?WORK_DIR.get(), "success initialize");
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

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_lib() {
//     }
// }
