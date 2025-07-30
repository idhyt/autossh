use std::env::{current_exe, home_dir};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};

static WORK_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static WORK_DIR_FILE: LazyLock<fn(&str) -> PathBuf> = LazyLock::new(|| {
    |n| {
        let w = get_work_dir();
        w.join(n)
    }
});

pub fn get_work_dir() -> &'static PathBuf {
    WORK_DIR.get_or_init(|| set_work_dir(Option::<&str>::None).expect("WORK_DIR not initialized"))
}

pub fn set_work_dir(w: Option<impl AsRef<Path>>) -> Result<PathBuf, Error> {
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
