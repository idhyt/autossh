use std::path::PathBuf;

mod key;
mod load;

pub use load::Config;

/// Resolve the default work directory, without global state.
///
/// Order of precedence:
/// 1. `ATSH_WORK_DIR` environment variable
/// 2. `$XDG_CONFIG_HOME/atsh.d` (or `$HOME/.config/atsh.d`)
/// 3. `$HOME/.atsh.d`
/// 4. `<executable-dir>/.atsh.d`
pub fn default_work_dir() -> PathBuf {
    if let Some(w) = std::env::var_os("ATSH_WORK_DIR") {
        return PathBuf::from(w);
    }
    if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config).join("atsh.d");
    }
    let home = std::env::home_dir();
    if let Some(h) = &home {
        return h.join(".config").join("atsh.d");
    }
    if let Some(h) = &home {
        return h.join(".atsh.d");
    }
    std::env::current_exe()
        .ok()
        .map(|e| e.with_file_name(".atsh.d"))
        .unwrap_or_else(|| {
            panic!("WORK_DIR not found: ATSH_WORK_DIR, HOME, or executable dir unavailable")
        })
}
