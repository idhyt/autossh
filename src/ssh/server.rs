use prettytable::{Cell, Row, Table};
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Remotes {
    pub list: Vec<Remote>,
}

impl Remotes {
    fn pprint(&self, all: bool) {
        let mut table = Table::new();
        let mut titles = vec!["index", "name", "user", "ip", "port"];
        if all {
            titles.push("password");
            titles.push("note");
        }
        table.set_titles(Row::new(
            titles
                .iter()
                .map(|v| Cell::new(v).style_spec("bcFg"))
                .collect::<Vec<Cell>>(),
        ));

        for remote in self.list.iter() {
            let mut row = vec![
                remote.index.to_string(),
                remote.name.clone().unwrap_or_else(|| "".to_string()),
                remote.user.clone(),
                remote.ip.clone(),
                remote.port.to_string(),
            ];
            if all {
                row.push(remote.password.clone());
                row.push(remote.note.clone().unwrap_or_else(|| "".to_string()));
            }
            table.add_row(Row::new(
                row.iter()
                    .map(|v| Cell::new(v).style_spec("lFc"))
                    .collect::<Vec<Cell>>(),
            ));
        }
        log::debug!("the remote list:\n{:#?}", self.list);
        table.printstd();
    }

    pub fn get(&self, index: &u16) -> Option<&Remote> {
        let index = *index;
        for remote in self.list.iter() {
            if remote.index == index {
                return Some(remote);
            }
        }
        log::error!("the index {} not found", index);
        None
    }

    pub fn list(&self) {
        self.pprint(false);
    }

    pub fn list_all(&self) {
        self.pprint(true);
    }

    pub fn add(
        &mut self,
        user: &str,
        password: &str,
        ip: &str,
        port: &u16,
        name: &Option<String>,
        note: &Option<String>,
    ) -> u16 {
        let indexs = self.list.iter().map(|v| v.index).collect::<Vec<u16>>();
        let index = indexs.iter().max().unwrap_or(&0) + 1;
        let remote = Remote {
            index,
            user: user.to_string(),
            password: password.to_string(),
            ip: ip.to_string(),
            port: *port,
            name: name.clone(),
            note: note.clone(),
        };
        log::debug!("add remote: {}", remote);
        self.list.push(remote);
        index
    }

    pub fn delete(&mut self, index: &Vec<u16>) -> u16 {
        // let index = *index;
        // self.remotes.retain(|v| v.index != index);
        self.list.retain(|v| !index.contains(&v.index));
        // index
        self.list.len() as u16
    }
}
