use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// use std::io::{BufRead, BufReader};
use std::io::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info, warn};

use super::rssh::SSHSession;
use crate::config::CONFIG;
use crate::storage::db::{
    delete_index, get_connection, insert, query_all, query_index, update_authorized,
};
use crate::storage::secure::{decrypt, encrypt};
use crate::Result;

#[derive(Debug, Default, Serialize, Deserialize)]
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

fn depass<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let password = String::deserialize(deserializer)?;
    Ok(decrypt(&password))
}

fn enpass<S>(password: &String, serializer: S) -> std::result::Result<S::Ok, S::Error>
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
    pub fn add_record(&self) -> Result<usize> {
        // Force check the ATSH_KEY exist or not
        CONFIG.get_enc_key()?;
        let n = {
            let conn = get_connection().lock();
            insert(&conn, &self)
        }
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        info!(remote = self.to_string(), "success add record");
        Ok(n)
    }

    pub fn delete_record(&self) -> Result<()> {
        // 删除数据库
        let conn = get_connection().lock();
        delete_index(&conn, self.index).map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        info!(remote = self.to_string(), "success delete record");
        Ok(())
    }

    pub async fn add_auth(&self) -> Result<()> {
        // check the `ATSH_KEY` exist or not
        // if not exist, { kind: Other, error: "Authentication failed (username/password)" }
        let _ = CONFIG.get_enc_key()?;
        let mut session = SSHSession::new(&self.user, &self.password, &self.ip, self.port).await?;
        session.authenticate(CONFIG.read_public()?.as_str()).await?;
        // 更新数据库
        if !self.authorized {
            // update authorized to database
            // self.authorized = true;
            let conn = get_connection().lock();
            update_authorized(&conn, self.index, true)?;
        }
        info!(remote = self.to_string(), "success add authenticate");
        Ok(())
    }

    pub async fn remove_auth(&self) -> Result<()> {
        let mut session = SSHSession::new(&self.user, &self.password, &self.ip, self.port).await?;
        session.revoke(CONFIG.read_public()?.as_str()).await?;
        info!(remote = self.to_string(), "success remove authenticate");
        Ok(())
    }

    pub async fn login(&self, reauth: bool) -> Result<()> {
        // 如果没有认证，或者通过 `--auth` 参数重新认证
        if !self.authorized || reauth {
            self.add_auth().await?;
        }
        let sshkey = CONFIG.get_private();
        let _ = Command::new("ssh")
            .arg(format!("{}@{}", self.user, self.ip))
            .arg("-p")
            .arg(self.port.to_string())
            .arg("-i")
            .arg(sshkey.to_str().unwrap())
            .status()
            .await?;
        info!(remote = self.to_string(), "success login");
        Ok(())
    }

    async fn scp(&self, args: &Vec<&str>) -> Result<()> {
        // 如果没有认证，则先认证
        if !self.authorized {
            debug!(remote = self.to_string(), "no authorized, try authenticate");
            self.add_auth().await?;
        }
        // info!("\n🚨 scp {}\n🚨 input `y` to run and other to cancel.", cmd);
        // let mut read = String::new();
        // std::io::stdin().read_line(&mut read)?;
        // let read = read.trim();
        // if read == "y" {}
        debug!(args=?args, "scp");

        let mut child = Command::new("scp")
            .args(args)
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stderr is none"))?;

        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            println!("{}", line);
        }

        let _ = child.wait().await?;

        Ok(())
    }

    pub async fn upload(&self, from: &str, to: &str) -> Result<()> {
        // debug!(from = ?from, to=?to, "upload");
        // scp -r -P 22 -i /home/idhyt/.ssh/id_rsa ./test.txt idhyt@1.2.3.4:/tmp
        let port = self.port.to_string();
        let remote = format!("{}@{}:{}", self.user, self.ip, to);
        let cmd = vec![
            "-r",
            "-P",
            &port,
            "-i",
            CONFIG.get_private().to_str().unwrap(),
            from,
            &remote,
        ];
        self.scp(&cmd).await?;
        info!(from=?from, to=?to, "susccess upload");
        Ok(())
    }

    pub async fn download(&self, from: &str, to: &str) -> Result<()> {
        // debug!(from = ?from, to=?to, "download");
        // scp -r -P 22 -i /home/idhyt/.ssh/id_rsa idhyt@1.2.3.4:/tmp/test.txt ./
        let port = self.port.to_string();
        let remote = format!("{}@{}:{}", self.user, self.ip, from);
        let cmd = vec![
            "-r",
            "-P",
            &port,
            "-i",
            CONFIG.get_private().to_str().unwrap(),
            &remote,
            to,
        ];
        self.scp(&cmd).await?;
        info!(from=?from, to=?to, "susccess download");
        Ok(())
    }
}

// #[derive(Serialize, Deserialize)]
pub struct Remotes(pub Vec<Remote>);

impl Remotes {
    fn load() -> Result<Remotes> {
        let conn = get_connection().lock();
        let remotes = query_all(&conn).map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        Ok(Remotes(remotes))
    }
    pub fn get(idx: usize) -> Result<Option<Remote>> {
        let remote = {
            let conn = get_connection().lock();
            query_index(&conn, idx)
        }
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        if remote.is_some() {
            info!(index = idx, "susccess get remote");
        } else {
            warn!(index = idx, "remote not found");
        }
        Ok(remote)
    }

    pub fn try_get(idx: usize) -> Result<Remote> {
        if let Some(remote) = Remotes::get(idx)? {
            Ok(remote)
        } else {
            Err(Box::new(Error::new(
                std::io::ErrorKind::NotFound,
                format!("index {} remote not found", idx),
            )))
        }
    }

    pub fn get_all() -> Result<Remotes> {
        let remotes = Remotes::load()?;
        info!("susccess get all remotes {}", remotes.0.len());
        Ok(remotes)
    }

    pub fn add(
        user: &str,
        password: &str,
        ip: &str,
        port: u16,
        name: &Option<impl AsRef<str>>,
        note: &Option<impl AsRef<str>>,
    ) -> Result<usize> {
        let remote = Remote {
            index: 0, // not used
            user: user.to_string(),
            password: password.to_string(),
            ip: ip.to_string(),
            port,
            authorized: false,
            name: name.as_ref().map(|n| n.as_ref().to_string()),
            note: note.as_ref().map(|n| n.as_ref().to_string()),
        };
        // we not authorized the remote server until the first login
        // remote.authorized();
        // debug!(remote = remote.to_string(), "add");
        remote.add_record()
    }

    pub async fn delete(indexs: &Vec<usize>) -> Result<usize> {
        let remotes: Vec<Remote> = indexs
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .iter()
            .filter_map(|&idx| Remotes::get(*idx).transpose())
            .collect::<std::result::Result<Vec<_>, _>>()?;
        debug!(total = remotes.len(), "delete");
        for remote in remotes.iter() {
            debug!(index = remote.index, "delete");
            // remove auth
            if remote.authorized {
                remote.remove_auth().await?;
            }
            // delete database
            remote.delete_record()?;
        }
        Ok(remotes.len())
    }

    pub fn pprint(&self, all: bool) {
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
                if let Ok(_) = CONFIG.get_enc_key() {
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
        debug!("the remote list total: {}", self.0.len());
        table.printstd();
    }
}
