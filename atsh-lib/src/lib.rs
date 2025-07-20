use chrono::Local;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};

mod config;
mod db;
mod ssh;

static WORK_DIR: OnceLock<PathBuf> = OnceLock::new();
pub(crate) static WORK_DIR_FILE: LazyLock<fn(&str) -> PathBuf> = LazyLock::new(|| {
    |n| {
        // let mut work_dir =
        //     std::env::current_exe().expect("failed to get current execute directory");
        // work_dir.pop();
        // work_dir.join(n)
        let work_dir = WORK_DIR.get().expect("WORK_DIR not initialized");
        work_dir.join(n)
    }
});

fn setup_logging(work_dir: &Path) -> Result<(), Error> {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    let log_dir = work_dir.join("logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
    }
    let log_file = log_dir.join(format!("{}.log", Local::now().format("%Y-%m-%d")));
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, log_file);

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_line_number(true),
        )
        .with(fmt::Layer::new().with_writer(file_appender));

    subscriber.init();
    tracing::info!(
        "Logging initialized (RUST_LOG={})",
        std::env::var("RUST_LOG").unwrap_or_default()
    );
    Ok(())
}

pub mod atsh {
    use crate::ssh::remote::Remotes;
    use crate::{WORK_DIR, setup_logging};
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    use tracing::debug;

    pub fn initialize(work_dir: Option<&Path>) -> Result<(), Error> {
        let work_dir = work_dir.map(|p| p.to_path_buf()).unwrap_or({
            let mut work_dir =
                std::env::current_exe().expect("failed to get current execute directory");
            work_dir.pop();
            work_dir
        });
        debug!("work_dir: {}", work_dir.display());

        if !work_dir.exists() {
            std::fs::create_dir_all(&work_dir)?;
        }
        setup_logging(&work_dir)?;

        WORK_DIR
            .set(work_dir)
            .expect("WORK_DIR already initialized");
        Ok(())
    }

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
