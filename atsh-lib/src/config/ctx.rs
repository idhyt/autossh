use std::env::{current_exe, home_dir};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};
use tracing::debug;

static WORK_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static WORK_DIR_FILE: LazyLock<fn(&str) -> PathBuf> =
    LazyLock::new(|| |n| get_work_dir().join(n));

/// if you want to change the work directory,
/// call before call the `get_work_dir`,
/// only set once before call the `get_work_dir`
pub fn set_work_dir(w: impl AsRef<Path>) -> Result<(), Error> {
    let wd = w.as_ref().to_path_buf();
    if !wd.exists() {
        std::fs::create_dir_all(&wd)?;
    }
    debug!(work_dir=?wd, "The work directory by user");
    // we set WORK_DIR immediately
    WORK_DIR
        .set(wd)
        .map_err(|_| Error::new(ErrorKind::AlreadyExists, "WORK_DIR already initialized"))?;
    Ok(())
}

/// get work directory
/// if not set, will use `$HOME/.atsh`
pub fn get_work_dir() -> &'static PathBuf {
    WORK_DIR.get_or_init(|| {
        let wd = if cfg!(test) {
            PathBuf::from("test.atsh.d")
        } else {
            let wd = home_dir()
                // the system home directory
                .map(|h| h.join(".atsh.d"))
                // current executable directory
                .or_else(|| current_exe().ok().map(|e| e.with_file_name(".atsh.d")))
                // Error
                // .ok_or_else(|| Error::new(ErrorKind::NotFound, "WORK_DIR not found"))?;
                .expect("WORK_DIR not found");
            wd
        };
        if !wd.exists() {
            std::fs::create_dir_all(&wd).expect("Failed to create work directory");
        }
        debug!(work_dir=?wd, "The work directory by default");
        wd
    })
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
        let w = set_work_dir(Path::new("test.atsh.d"));
        println!("set work dir: {:?}", w);
        assert!(w.is_err());
        assert!(w.err().unwrap().to_string().contains("already initialized"));
    }

    #[test]
    #[ignore = "Only for debug"]
    fn test_work_dir_set() {
        let sw = PathBuf::from("debug.atsh.d");
        let w = set_work_dir(&sw);
        assert!(w.is_ok());
        let w = get_work_dir();
        println!("set work dir: {:?}", w);
        assert_eq!(w.to_owned(), sw);
    }
}
