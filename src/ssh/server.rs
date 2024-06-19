use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(target_family = "unix")]
use super::bind::passh;
use super::secure::{decrypt, encrypt};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Remote {
    /// the index of the remote server.
    pub index: u16,
    /// the login user.
    pub user: String,
    /// the login password.
    #[serde(deserialize_with = "depass", serialize_with = "enpass")]
    pub password: String,
    /// the login id address.
    pub ip: String,
    /// the login port.
    pub port: u16,
    /// the alias name for the login.
    pub name: Option<String>,
    /// the note for the server.
    pub note: Option<String>,
}

fn depass<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let password = String::deserialize(deserializer)?;
    Ok(decrypt(&password))
}

fn enpass<S>(password: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Serialize::serialize(&encrypt(password), serializer)
}

// impl display for Remote
impl std::fmt::Display for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ssh {}@{} -p {}", self.user, self.ip, self.port,)
    }
}

impl Remote {
    #[cfg(target_family = "unix")]
    pub fn login(&self) {
        log::debug!("login at: {}", self);

        unsafe {
            // passh -c 10 -p password ssh -p port user@ip
            let argv = vec![
                std::ffi::CString::new("passh").unwrap().into_raw(),
                std::ffi::CString::new("-c").unwrap().into_raw(),
                std::ffi::CString::new("10").unwrap().into_raw(),
                std::ffi::CString::new("-p").unwrap().into_raw(),
                std::ffi::CString::new(self.password.clone())
                    .unwrap()
                    .into_raw(),
                std::ffi::CString::new("ssh").unwrap().into_raw(),
                std::ffi::CString::new("-p").unwrap().into_raw(),
                std::ffi::CString::new(self.port.to_string())
                    .unwrap()
                    .into_raw(),
                std::ffi::CString::new(format!("{}@{}", self.user, self.ip))
                    .unwrap()
                    .into_raw(),
            ];
            let argc = argv.len() as i32;
            passh(argc, argv.as_ptr() as *mut *mut std::os::raw::c_char);
        };
    }

    #[cfg(target_os = "windows")]
    pub fn login(&self) {
        let putty_path = {
            let mut exe_path = std::env::current_exe().unwrap();
            exe_path.pop();
            exe_path.push("putty.exe");
            exe_path
        };
        if !putty_path.exists() {
            log::error!("`putty.exe` not found, you can download it from `https://www.chiark.greenend.org.uk/~sgtatham/putty/` and put it to {}.", putty_path.display());
            return;
        }
        // putty.exe -ssh user@ip -P port -pw password
        let cmd = format!(
            "{} -ssh {}@{} -P {} -pw {}",
            putty_path.display(),
            self.user,
            self.ip,
            self.port,
            self.password
        );
        log::debug!("login at: {}", self);
        std::process::Command::new("cmd")
            .args(&["/C", &cmd])
            .spawn()
            .unwrap();
    }
}
