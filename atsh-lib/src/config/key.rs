use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

use crate::error::{AtshError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct SSHKey {
    public: PathBuf,
    private: PathBuf,
}

impl SSHKey {
    pub fn new(public: PathBuf, private: PathBuf) -> Self {
        SSHKey { public, private }
    }

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
    info!("generating rsa key pair...");

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
                warn!("no password provided, key will be passwordless");
                String::new()
            }
        };
        args.push("-N");
        args.push(&pass);

        for p in [output, &output.with_extension("pub")] {
            if p.is_file() {
                warn!(file = ?p, "ssh key exists, removing it");
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

    info!(path = %output.display(), "ssh key generated");
    Ok(output.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sshkey() {
        let dir = PathBuf::from("test.atsh.d");
        std::fs::create_dir_all(&dir).unwrap();
        let output = dir.join("id_rsa");
        assert!(create_sshkey(Some("123456"), &output, false).is_err());
        let result = create_sshkey(Some("12345678"), &output, false);
        assert!(result.is_ok());
    }
}