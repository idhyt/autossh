use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::env::home_dir;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::{debug, info, warn};

use super::ctx::WORK_DIR_FILE;

// `ATSH_KEY` start
//  The key used to encrypt the data like password.
static ATSH_KEY: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| {
    if let Ok(key) = std::env::var("ATSH_KEY") {
        debug!("`ATSH_KEY` found in environment variable");
        return Mutex::new(Some(key));
    }
    if let Ok(key) = std::env::var("ASKEY") {
        warn!("💡 Deprecated `ASKEY` in next version and use `ATSH_KEY` instead");
        return Mutex::new(Some(key));
    }
    // warn!("💥 export `ASKEY` to protect password! 💥");
    Mutex::new(None)
});

pub fn get_atshkey() -> Result<String, Error> {
    let key = {
        let k = ATSH_KEY.lock();
        k.clone()
    };
    if let Some(k) = key {
        Ok(k)
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            "💥 Export `ATSH_KEY` to protect password",
        ))
    }
}

pub fn set_atshkey(key: Option<impl AsRef<str>>) -> Result<(), Error> {
    if key.is_none() {
        info!("🔑 Cleaning ATSH_KEY...");
        *ATSH_KEY.lock() = None;
        return Ok(());
    }
    let key = key.unwrap();
    let set = key.as_ref();
    if set.len() < 5 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "💥 ATSH_KEY must be at least 5 characters",
        ));
    }
    info!("🔑 Set ATSH_KEY to {}...", &set[..2]);
    *ATSH_KEY.lock() = Some(set.to_string());
    Ok(())
}
// `ATSH_KEY` end

/////////////////////////////////////////////////////////////////////////////////////////////////////

// The ssh key to use for remote host authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct SSHKey {
    /// the public key location.
    public: PathBuf,
    /// the private key location.
    private: PathBuf,
}

impl Default for SSHKey {
    fn default() -> Self {
        let home = home_dir().expect("Failed to get home directory");
        let (public, private) = (
            home.join(".ssh").join("id_rsa.pub"),
            home.join(".ssh").join("id_rsa"),
        );
        let msg = format!(
            r#"
  🔹 No SSH key specified. Defaulting to: {:?}
  🔹 SSH key used for remote host authentication.
  🔹 To generate a new key (passphrase-free):
        ssh-keygen -t rsa -b 2048 -C "atsh" -N "" -f {:?}"#,
            private,
            WORK_DIR_FILE("id_rsa")
        );
        warn!("💡 The first run to create a default config{}", msg);
        SSHKey { public, private }
    }
}

impl SSHKey {
    pub fn get_public(&self) -> &Path {
        self.public.as_path()
    }

    pub fn get_private(&self) -> &Path {
        self.private.as_path()
    }

    pub fn read_public(&self) -> Result<String, Error> {
        let key = self.get_public();
        if !key.is_file() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "public key not found, you can generate it by `ssh-keygen` and set it to config",
            ));
        }
        std::fs::read_to_string(key)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_key() {
        let s = set_atshkey(Option::<&str>::None);
        assert!(s.is_ok());
        let key = get_atshkey();
        assert!(key.is_err());
        let s = set_atshkey(Some("abcdefg"));
        assert!(s.is_ok());
        let key = get_atshkey();
        assert!(key.is_ok());
        assert_eq!(key.unwrap(), "abcdefg");
        let s = set_atshkey(Some("abc"));
        assert!(s.is_err());
        assert!(s
            .err()
            .unwrap()
            .to_string()
            .contains("at least 5 characters"));
    }
}
