use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::secure::{decrypt, encrypt, panic_if_not_secure};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Remote {
    /// the index of the remote server.
    pub index: u16,
    /// the login user.
    pub user: String,
    /// the login password.
    #[serde(deserialize_with = "depass", serialize_with = "enpass")]
    pub password: String,
    /// have the authorized to login.
    pub authorized: bool,
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
    pub fn login(&self) {
        log::debug!("login at: {}", self);
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
        panic_if_not_secure();

        let indexs = self.list.iter().map(|v| v.index).collect::<Vec<u16>>();
        let index = indexs.iter().max().unwrap_or(&0) + 1;
        let remote = Remote {
            index,
            user: user.to_string(),
            password: password.to_string(),
            authorized: false,
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
