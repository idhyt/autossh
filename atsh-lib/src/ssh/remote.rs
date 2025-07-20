use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::io::{BufRead, BufReader};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{debug, info};

use super::secure::{check_secure, decrypt, encrypt};
use super::session::SSHSession;
use crate::config::CONFIG;
use crate::db::{self, delete_index, get_connection, update_authorized};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Remote {
    /// the index of the remote server.
    // #[serde(rename = "idx")]
    pub index: usize,
    /// the login user.
    pub user: String,
    /// the login password.
    #[serde(deserialize_with = "depass", serialize_with = "enpass")]
    pub password: String,
    /// the login id address.
    pub ip: String,
    /// the login port.
    pub port: u16,
    /// have the authorized to login.
    pub authorized: bool,
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
        write!(f, "{}@{}:{}", self.user, self.ip, self.port,)
    }
}

impl Remote {
    // 确保先从数据库查询
    pub fn delete(&self) -> Result<(), Error> {
        // 删除认证
        let session = SSHSession::new(&self.user, &self.password, &self.ip, self.port)?;
        session.revoke()?;
        // 删除数据库
        let conn = get_connection().lock();
        delete_index(&conn, self.index).map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub fn login(&self, reauth: bool) -> Result<(), Error> {
        // 如果没有认证，或者通过 `--auth` 参数重新认证
        if !self.authorized || reauth {
            let session = SSHSession::new(&self.user, &self.password, &self.ip, self.port)?;
            session.authenticate()?;
        }
        if !self.authorized {
            // update authorized to database
            // self.authorized = true;
            let conn = get_connection().lock();
            update_authorized(&conn, self.index, true)
                .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        }

        let sshkey = CONFIG.get_private_key();
        debug!(remote=?self, "login");
        Command::new("ssh")
            .arg(format!("{}@{}", self.user, self.ip))
            .arg("-p")
            .arg(self.port.to_string())
            .arg("-i")
            .arg(sshkey.to_str().unwrap())
            .status()?;
        Ok(())
    }

    fn scp(&self, from: &str, to: &str, upload: bool) -> Result<(), Error> {
        let private_key = CONFIG.get_private_key();
        if upload {}
        let cmd = if upload {
            assert!(PathBuf::from(from).exists(), "file not found at: {}", from);
            format!(
                "-r -P {p} -i {k} {l} {u}@{i}:{r}",
                p = &self.port,
                k = private_key.display(),
                u = &self.user,
                i = &self.ip,
                l = from,
                r = to,
            )
        } else {
            format!(
                "-r -P {p} -i {k} {u}@{i}:{r} {l}",
                p = &self.port,
                k = private_key.display(),
                u = &self.user,
                i = &self.ip,
                r = from,
                l = to,
            )
        };
        info!("\n🚨 scp {}\n🚨 input `y` to run and other to cancel.", cmd);
        let mut read = String::new();
        std::io::stdin().read_line(&mut read)?;
        let read = read.trim();
        if read == "y" {
            debug!("run command: scp {}", cmd);
            let stderr = Command::new("scp")
                .args(&cmd.split(' ').collect::<Vec<&str>>())
                // .stdin(Stdio::piped())
                // .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
                .stderr;
            if stderr.is_none() {
                return Err(Error::new(ErrorKind::BrokenPipe, "stderr is none"));
            }
            let reader = BufReader::new(stderr.unwrap());
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{}", line));
        }
        Ok(())
    }

    pub fn upload(&self, from: &str, to: &str) -> Result<(), Error> {
        debug!("upload file from {} to {}:{}", from, self, to);
        self.scp(from, to, true)
    }

    pub fn download(&self, from: &str, to: &str) -> Result<(), Error> {
        debug!("download file from {}:{} to {}", self, from, to);
        self.scp(from, to, false)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Remotes(pub Vec<Remote>);

impl Remotes {
    fn load() -> Result<Remotes, Error> {
        let conn = db::get_connection().lock();
        let remotes = db::query_all(&conn).map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        Ok(Remotes(remotes))
    }
    pub fn get(idx: usize) -> Result<Remote, Error> {
        let remote = {
            let conn = db::get_connection().lock();
            db::query_index(&conn, idx)
        }
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        if let Some(r) = remote {
            Ok(r)
        } else {
            Err(Error::new(
                ErrorKind::NotFound,
                format!("Remote {} not found", idx),
            ))
        }
    }

    pub fn add(
        user: &str,
        password: &str,
        ip: &str,
        port: u16,
        name: &Option<String>,
        note: &Option<String>,
    ) -> Result<usize, Error> {
        check_secure()?;
        let remote = Remote {
            index: 0, // not used
            user: user.to_string(),
            password: password.to_string(),
            ip: ip.to_string(),
            port,
            authorized: false,
            name: name.clone(),
            note: note.clone(),
        };
        // we not authorized the remote server until the first login
        // remote.authorized();
        debug!(remote = ?remote, "add");
        let n = {
            let conn = db::get_connection().lock();
            db::insert(&conn, &remote)
        }
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        Ok(n)
    }

    pub fn delete(index: &Vec<usize>) -> Result<usize, Error> {
        let mut n = 0;

        for idx in index.iter() {
            debug!("delete index: {}", idx);
            // 查询是否存在
            let find = {
                let conn = db::get_connection().lock();
                db::query_index(&conn, *idx)
                    .map_err(|e| Error::new(std::io::ErrorKind::NotFound, e))?
            };
            if let Some(remote) = find {
                remote.delete()?;
                n += 1;
            }
        }
        Ok(n)
    }

    pub fn list() -> Result<(), Error> {
        let remotes = Remotes::load()?;
        remotes.pprint(false);
        Ok(())
    }

    pub fn list_all() -> Result<(), Error> {
        let remotes = Remotes::load()?;
        remotes.pprint(true);
        Ok(())
    }

    fn pprint(&self, all: bool) {
        let mut table = Table::new();
        let mut titles = vec!["index", "name", "user", "ip", "port"];
        if all {
            titles.push("password");
            titles.push("authorized");
            titles.push("note");
        }
        table.set_titles(Row::new(
            titles
                .iter()
                .map(|v| Cell::new(v).style_spec("bcFg"))
                .collect::<Vec<Cell>>(),
        ));

        for remote in self.0.iter() {
            let mut row = vec![
                remote.index.to_string(),
                remote.name.clone().unwrap_or_else(|| "".to_string()),
                remote.user.clone(),
                remote.ip.clone(),
                remote.port.to_string(),
            ];
            if all {
                if let Ok(_) = check_secure() {
                    row.push(remote.password.clone());
                } else {
                    row.push(format!(
                        "{}..{}",
                        &remote.password[..3],
                        &remote.password[remote.password.len() - 5..]
                    ));
                }
                // row.push(remote.password.clone());
                row.push(remote.authorized.to_string());
                row.push(remote.note.clone().unwrap_or_else(|| "".to_string()));
            }
            table.add_row(Row::new(
                row.iter()
                    .map(|v| Cell::new(v).style_spec("lFc"))
                    .collect::<Vec<Cell>>(),
            ));
        }
        debug!("the remote list:\n{:#?}", self.0);
        table.printstd();
    }
}
