use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use tracing::{debug, info, warn};

use super::ctx::WORK_DIR_FILE;
use crate::error::{AtshError, Result};

static ATSH_KEY: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| {
    if let Ok(key) = std::env::var("ATSH_KEY") {
        debug!("`ATSH_KEY` found in environment variable");
        return Mutex::new(Some(key));
    }
    if let Ok(key) = std::env::var("ASKEY") {
        warn!("Deprecated `ASKEY`, use `ATSH_KEY` instead");
        return Mutex::new(Some(key));
    }
    Mutex::new(None)
});

pub fn get_atshkey() -> Result<String> {
    let key = ATSH_KEY.lock().clone();
    key.ok_or(AtshError::EncKeyNotSet)
}

pub fn set_atshkey(key: Option<impl AsRef<str>>) -> Result<()> {
    match key {
        None => {
            info!("Cleaning ATSH_KEY...");
            *ATSH_KEY.lock() = None;
            Ok(())
        }
        Some(k) => {
            let set = k.as_ref();
            if set.len() < 5 {
                return Err(AtshError::EncKeyTooShort);
            }
            info!("Set ATSH_KEY to {}...", &set[..2]);
            *ATSH_KEY.lock() = Some(set.to_string());
            Ok(())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SSHKey {
    public: PathBuf,
    private: PathBuf,
}

impl Default for SSHKey {
    fn default() -> Self {
        let (public, private) = (WORK_DIR_FILE("id_rsa.pub"), WORK_DIR_FILE("id_rsa"));
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

    fn read_key(&self, key_type: &str) -> Result<String> {
        let key = match key_type {
            "public" => self.get_public(),
            "private" => self.get_private(),
            _ => return Err(AtshError::InvalidInput("unknown key type".into())),
        };
        if !key.is_file() {
            return Err(AtshError::SshKeyNotFound {
                key_type: key_type.to_string(),
                path: key.display().to_string(),
            });
        }
        Ok(std::fs::read_to_string(key)?)
    }

    pub fn read_public(&self) -> Result<String> {
        self.read_key("public")
    }

    pub fn read_private(&self) -> Result<String> {
        self.read_key("private")
    }
}

pub fn create_sshkey(
    password: Option<impl AsRef<str>>,
    output: impl AsRef<Path>,
    interactive: bool,
) -> Result<PathBuf> {
    info!("Starting generating rsa key pair...");

    let output = output.as_ref();
    let mut args = vec![
        "-t",
        "rsa",
        "-b",
        "2048",
        "-C",
        "atsh",
        "-f",
        output
            .to_str()
            .ok_or_else(|| AtshError::InvalidInput("invalid output path".into()))?,
    ];

    let status = if interactive {
        Command::new("ssh-keygen").args(&args).status()?
    } else {
        let pass = match password {
            Some(p) => {
                let p = p.as_ref();
                if p.len() < 8 {
                    return Err(AtshError::InvalidInput(
                        "password must be at least 8 characters long".into(),
                    ));
                }
                debug!(len = p.len(), "sshkey with password");
                p.to_string()
            }
            None => {
                warn!("No password provided, will use empty password");
                String::new()
            }
        };
        args.push("-N");
        args.push(&pass);

        for p in [output, &output.with_extension("pub")] {
            if p.is_file() {
                warn!(file = ?p, "SSH Key exists, remove it");
                std::fs::remove_file(p)?;
            }
        }
        Command::new("ssh-keygen").args(&args).status()?
    };

    if !status.success() {
        return Err(AtshError::SshKeygenFailed(format!(
            "exit code: {:?}",
            status.code()
        )));
    }

    info!("SSH key generated successfully at: {:?}", output);
    Ok(output.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key() {
        assert!(set_atshkey(Option::<&str>::None).is_ok());
        assert!(get_atshkey().is_err());
        assert!(set_atshkey(Some("abcdefg")).is_ok());
        assert_eq!(get_atshkey().unwrap(), "abcdefg");
        assert!(set_atshkey(Some("abc")).is_err());
    }

    #[test]
    fn test_sshkey() {
        let output = WORK_DIR_FILE("id_rsa");
        assert!(create_sshkey(Some("123456"), &output, false).is_err());
        let result = create_sshkey(Some("12345678"), &output, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), output);
    }
}