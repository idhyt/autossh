use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::{debug, info, warn};

use super::rssh::SSHSession;
use crate::config::CONFIG;
use crate::error::{AtshError, Result};
use crate::storage::db::{
    delete_index, get_connection, insert, query_all, query_index, update_authorized,
};
use crate::storage::secure::{decrypt, encrypt};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Remote {
    pub index: usize,
    pub user: String,
    #[serde(deserialize_with = "depass", serialize_with = "enpass")]
    pub password: String,
    pub ip: String,
    pub port: u16,
    pub authorized: bool,
    pub name: Option<String>,
    pub note: Option<String>,
}

fn depass<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let password = String::deserialize(deserializer)?;
    Ok(decrypt(&password).unwrap_or(password))
}

fn enpass<S>(
    password: &String,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encrypted = encrypt(password).unwrap_or_else(|_| password.clone());
    Serialize::serialize(&encrypted, serializer)
}

impl std::fmt::Display for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "[{name}] {}@{}:{}", self.user, self.ip, self.port)
        } else {
            write!(f, "{}@{}:{}", self.user, self.ip, self.port)
        }
    }
}

impl Remote {
    pub fn save(&self) -> Result<usize> {
        CONFIG.get_enc_key()?;
        let n = {
            let conn = get_connection().lock();
            insert(&conn, self)?
        };
        info!(remote = self.to_string(), "success add record");
        Ok(n)
    }

    pub fn delete_from_db(&self) -> Result<()> {
        let conn = get_connection().lock();
        delete_index(&conn, self.index)?;
        info!(remote = self.to_string(), "success delete record");
        Ok(())
    }

    pub async fn authorize_public_key(&self) -> Result<()> {
        CONFIG.get_enc_key()?;
        let mut session =
            SSHSession::new(&self.user, &self.password, &self.ip, self.port).await?;
        let pub_key = CONFIG.read_public().map_err(|e| AtshError::Config(e.to_string()))?;
        session.authenticate(pub_key.as_str()).await?;
        if !self.authorized {
            let conn = get_connection().lock();
            update_authorized(&conn, self.index, true)?;
        }
        info!(remote = self.to_string(), "success add authenticate");
        Ok(())
    }

    pub async fn deauthorize_public_key(&self) -> Result<()> {
        CONFIG.get_enc_key()?;
        let mut session =
            SSHSession::new(&self.user, &self.password, &self.ip, self.port).await?;
        let pub_key = CONFIG.read_public().map_err(|e| AtshError::Config(e.to_string()))?;
        session.revoke(pub_key.as_str()).await?;
        info!(remote = self.to_string(), "success remove authenticate");
        Ok(())
    }

    pub async fn login(&self, reauth: bool) -> Result<()> {
        if !self.authorized || reauth {
            self.authorize_public_key().await?;
        }
        let private_key_path = CONFIG
            .get_private()
            .to_str()
            .ok_or_else(|| AtshError::InvalidInput("invalid private key path".into()))?;
        let mut session =
            SSHSession::connect_with_key(&self.user, private_key_path, &self.ip, self.port)
                .await?;
        session.interactive_login_shell().await?;
        info!(remote = self.to_string(), "success login");
        Ok(())
    }

    pub async fn upload(&self, from: &str, to: &str) -> Result<()> {
        if !self.authorized {
            debug!(remote = self.to_string(), "no authorized, try authenticate");
            self.authorize_public_key().await?;
        }
        let private_key_path = CONFIG
            .get_private()
            .to_str()
            .ok_or_else(|| AtshError::InvalidInput("invalid private key path".into()))?;
        let mut session =
            SSHSession::connect_with_key(&self.user, private_key_path, &self.ip, self.port)
                .await?;
        session.scp_upload(from, to).await?;
        info!(from = from, to = to, "success upload");
        Ok(())
    }

    pub async fn download(&self, from: &str, to: &str) -> Result<()> {
        let private_key_path = CONFIG
            .get_private()
            .to_str()
            .ok_or_else(|| AtshError::InvalidInput("invalid private key path".into()))?;
        let mut session =
            SSHSession::connect_with_key(&self.user, private_key_path, &self.ip, self.port)
                .await?;
        session.scp_download(from, to).await?;
        info!(from = from, to = to, "success download");
        Ok(())
    }

    pub async fn run_command(&self, cmd: Option<&str>) -> Result<()> {
        let private_key_path = CONFIG
            .get_private()
            .to_str()
            .ok_or_else(|| AtshError::InvalidInput("invalid private key path".into()))?;
        let mut session =
            SSHSession::connect_with_key(&self.user, private_key_path, &self.ip, self.port)
                .await?;
        session.interactive_shell(80, 24, cmd).await?;
        Ok(())
    }
}

pub struct Remotes(pub Vec<Remote>);

impl Remotes {
    fn load() -> Result<Remotes> {
        let conn = get_connection().lock();
        let remotes = query_all(&conn)?;
        Ok(Remotes(remotes))
    }

    pub fn get(idx: usize) -> Result<Option<Remote>> {
        let conn = get_connection().lock();
        let remote = query_index(&conn, idx)?;
        if remote.is_some() {
            info!(index = idx, "success get remote");
        } else {
            warn!(index = idx, "remote not found");
        }
        Ok(remote)
    }

    pub fn try_get(idx: usize) -> Result<Remote> {
        Remotes::get(idx)?.ok_or(AtshError::RemoteNotFound { index: idx })
    }

    pub fn get_all() -> Result<Remotes> {
        let remotes = Remotes::load()?;
        info!("success get all remotes {}", remotes.0.len());
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
            index: 0,
            user: user.to_string(),
            password: password.to_string(),
            ip: ip.to_string(),
            port,
            authorized: false,
            name: name.as_ref().map(|n| n.as_ref().to_string()),
            note: note.as_ref().map(|n| n.as_ref().to_string()),
        };
        remote.save()
    }

    pub async fn delete(indexs: &[usize]) -> Result<usize> {
        let remotes: Vec<Remote> = indexs
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .iter()
            .filter_map(|&idx| Remotes::get(*idx).transpose())
            .collect::<std::result::Result<Vec<_>, _>>()?;
        debug!(total = remotes.len(), "delete");
        for remote in remotes.iter() {
            debug!(index = remote.index, "delete");
            if remote.authorized {
                if let Err(e) = remote.deauthorize_public_key().await {
                    warn!("failed to revoke auth for remote {}: {}", remote, e);
                }
            }
            remote.delete_from_db()?;
        }
        Ok(remotes.len())
    }
}

pub struct RemoteDisplay;

impl RemoteDisplay {
    pub fn pprint(remotes: &[Remote], all: bool) {
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

        for remote in remotes {
            let mut row = vec![
                remote.index.to_string(),
                remote.name.clone().unwrap_or_default(),
                remote.user.clone(),
                remote.ip.clone(),
                remote.port.to_string(),
            ];
            if all {
                let pwd_display = if CONFIG.get_enc_key().is_ok() {
                    remote.password.clone()
                } else if remote.password.len() > 8 {
                    format!(
                        "{}..{}",
                        &remote.password[..3],
                        &remote.password[remote.password.len() - 5..]
                    )
                } else {
                    "****".to_string()
                };
                row.push(pwd_display);
                row.push(remote.authorized.to_string());
                row.push(remote.note.clone().unwrap_or_default());
            }
            table.add_row(Row::new(
                row.iter()
                    .map(|v| Cell::new(v).style_spec("lFc"))
                    .collect::<Vec<Cell>>(),
            ));
        }
        debug!("the remote list total: {}", remotes.len());
        table.printstd();
    }
}