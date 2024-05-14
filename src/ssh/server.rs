use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::bind::passh;
use super::secure::{decrypt, encrypt};

#[derive(Debug, Default, Serialize, Deserialize)]
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
}
