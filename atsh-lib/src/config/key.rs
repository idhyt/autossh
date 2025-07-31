use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;
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
        let (public, private) = (WORK_DIR_FILE("id_rsa.pub"), WORK_DIR_FILE("id_rsa"));
        //         let msg = format!(
        //             r#"
        //   🔹 No SSH key specified. Defaulting to: {:?}
        //   🔹 SSH key used for remote host authentication.
        //   🔹 To generate a new key (passphrase-free):
        //         ssh-keygen -t rsa -b 2048 -C "atsh" -N "" -f {:?}"#,
        //             private,
        //             WORK_DIR_FILE("id_rsa")
        //         );
        //         warn!("💡 The first run to create a default config{}", msg);
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

pub fn create_sshkey(
    password: Option<impl AsRef<str>>,
    output: impl AsRef<Path>,
    interactive: bool,
) -> Result<PathBuf, Error> {
    info!("🔑 Starting generating rsa key pair...");

    let output = output.as_ref();
    let mut args = vec![
        "-t",
        "rsa",
        "-b",
        "2048",
        "-C",
        "atsh",
        "-f",
        output.to_str().unwrap(),
    ];

    let status = {
        if interactive {
            Command::new("ssh-keygen").args(&args).status()?
        } else {
            let pass = match password {
                Some(p) => {
                    let p = p.as_ref();
                    if p.len() < 8 {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "password must be at least 8 characters long",
                        ));
                    }
                    debug!(
                        len = p.len(),
                        "✅ sshkey with setting password {}..{}",
                        &p[..1],
                        // &p[p.len().saturating_sub(2)..]
                        &p[p.len() - 2..]
                    );
                    p.to_string()
                }
                None => {
                    warn!("⚠️  No password provided, will use empty password");
                    "".to_string()
                }
            };
            args.push("-N");
            args.push(&pass);

            // clean exist key
            for p in vec![output, &output.with_extension("pub")] {
                if p.is_file() {
                    warn!(file = ?p, "SSH Key exists, remove it");
                    std::fs::remove_file(p)?;
                }
            }
            Command::new("ssh-keygen").args(&args).status()?
        }
    };

    // let status = Command::new("ssh-keygen").args(&args).status()?;
    if !status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "Failed to generate SSH key (exit code: {:?})",
                status.code()
            ),
        ));
    }

    info!("✅ SSH key generated successfully at: {:?}", output);

    Ok(output.to_owned())
}

// fn ask_sshkey_password() -> Result<Option<String>, Error> {
//     use std::io::{self, Write};

//     println!("🔑 Starting generating rsa key pair...");
//     let mut password = Option::<String>::None;
//     loop {
//         println!("🔐 Use password? (Y/y for password, N/n for password-less login)");
//         print!("🛠️ Your choice [Y/N]: ");
//         io::stdout().flush()?;

//         let mut input = String::new();
//         io::stdin().read_line(&mut input)?;

//         match input.trim().to_lowercase().as_str() {
//             "y" | "yes" => {
//                 print!("🔑 Enter password: ");
//                 io::stdout().flush()?;
//                 let mut input = String::new();
//                 io::stdin().read_line(&mut input)?;
//                 // let check = input.trim();
//                 // if check.len() < 8 {
//                 //     println!("⚠️ Password must be at least 8 characters long.");
//                 // } else {
//                 //     password = Some(check.to_string());
//                 //     debug!(
//                 //         "✅ sshkey with ask input password: {}..{}",
//                 //         &check[..2],
//                 //         &check[check.len() - 2..]
//                 //     );
//                 //     break;
//                 // }
//                 password = Some(input.trim().to_string());
//                 break;
//             }
//             "n" | "no" => {
//                 println!("🚀 Using password-less login");
//                 break;
//             }
//             _ => println!("⚠️ Invalid input, please try again"),
//         }
//     }

//     Ok(password)
// }

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

    #[test]
    fn test_sshkey() {
        let password = "123456";
        let output = WORK_DIR_FILE("id_rsa");
        let s = create_sshkey(Some(password), &output, false);
        assert!(s.is_err());
        assert!(s
            .err()
            .unwrap()
            .to_string()
            .contains("least 8 characters long"));

        let password = "12345678";
        let s = create_sshkey(Some(password), &output, false);
        println!("{:#?}", s);
        assert!(s.is_ok());
        assert_eq!(s.unwrap(), output);

        let check = Command::new("ssh-keygen")
            .args(&["-y", "-f", output.to_str().unwrap(), "-P", password])
            .status();
        assert!(check.is_ok());
        assert!(check.unwrap().success());

        // let s = create_sshkey(Some(password), &output, true);
        // assert!(s.is_ok());
    }
}
