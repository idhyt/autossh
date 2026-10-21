use std::path::PathBuf;

mod key;
mod load;

pub use load::Config;

/// Resolve the default work directory, without global state.
///
/// Order of precedence:
/// 1. `ATSH_WORK_DIR` environment variable
/// 2. `$HOME/.atsh.d`
/// 3. `<executable-dir>/.atsh.d`
pub fn default_work_dir() -> PathBuf {
    if let Some(w) = std::env::var_os("ATSH_WORK_DIR") {
        return PathBuf::from(w);
    }
    if let Some(h) = std::env::home_dir() {
        return h.join(".atsh.d");
    }
    std::env::current_exe()
        .ok()
        .map(|e| e.with_file_name(".atsh.d"))
        .expect("WORK_DIR not found")
}
