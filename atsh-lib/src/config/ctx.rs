use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};
use tracing::debug;

use crate::error::{AtshError, Result};

static WORK_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static WORK_DIR_FILE: LazyLock<fn(&str) -> PathBuf> =
    LazyLock::new(|| |n| get_work_dir().join(n));

pub fn set_work_dir(w: impl AsRef<Path>) -> Result<()> {
    let wd = w.as_ref().to_path_buf();
    if !wd.exists() {
        std::fs::create_dir_all(&wd)?;
    }
    debug!(work_dir = ?wd, "The work directory by user");
    WORK_DIR
        .set(wd)
        .map_err(|_| AtshError::Config("WORK_DIR already initialized".into()))?;
    Ok(())
}

pub fn get_work_dir() -> &'static PathBuf {
    WORK_DIR.get_or_init(|| {
        let wd = if cfg!(test) {
            PathBuf::from("test.atsh.d")
        } else if let Some(w) = std::env::var_os("ATSH_WORK_DIR") {
            let wd = PathBuf::from(w);
            debug!(work_dir = ?wd, "The work directory by environment `ATSH_WORK_DIR`");
            wd
        } else {
            let wd = std::env::home_dir()
                .map(|h| h.join(".atsh.d"))
                .unwrap_or_else(|| {
                    std::env::current_exe()
                        .ok()
                        .map(|e| e.with_file_name(".atsh.d"))
                        .expect("WORK_DIR not found")
                });
            wd
        };
        if !wd.exists() {
            std::fs::create_dir_all(&wd).expect("Failed to create work directory");
        }
        debug!(work_dir = ?wd, "The work directory by default");
        wd
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_dir_default() {
        let w = get_work_dir();
        let w = set_work_dir(Path::new("test.atsh.d"));
        assert!(w.is_err());
    }
}