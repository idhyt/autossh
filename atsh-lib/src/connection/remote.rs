use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::rssh::SSHSession;
use crate::context::AppContext;
use crate::error::{AtshError, Result};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Remote {
    pub index: usize,
    pub user: String,
    pub password: String,
    pub ip: String,
    pub port: u16,
    pub authorized: bool,
    pub name: Option<String>,
    pub note: Option<String>,
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
    pub fn save(&self, ctx: &AppContext) -> Result<usize> {
        ctx.require_enc_key()?;
        let mut remotes = ctx.load_remotes()?;
        let index = remotes.iter().map(|r| r.index).max().unwrap_or(0) + 1;
        let mut record = self.clone();
        record.index = index;
        remotes.push(record);
        ctx.save_remotes(&remotes)?;
        info!(remote = self.to_string(), "added record");
        Ok(index)
    }

    pub fn delete_from_db(&self, ctx: &AppContext) -> Result<()> {
        let mut remotes = ctx.load_remotes()?;
        remotes.retain(|r| r.index != self.index);
        ctx.save_remotes(&remotes)?;
        info!(remote = self.to_string(), "deleted record");
        Ok(())
    }

    pub async fn authorize_public_key(&self, ctx: &AppContext) -> Result<()> {
        info!(remote = self.to_string(), "authorizing public key");
        let password = &self.password;
        let mut session = SSHSession::new(&self.user, password, &self.ip, self.port).await?;
        let pub_key = ctx.read_public_key()?;
        session.authenticate(pub_key.as_str()).await?;
        if !self.authorized {
            let mut remotes = ctx.load_remotes()?;
            if let Some(r) = remotes.iter_mut().find(|r| r.index == self.index) {
                r.authorized = true;
            }
            ctx.save_remotes(&remotes)?;
        }
        info!(remote = self.to_string(), "public key authorized");
        Ok(())
    }

    pub async fn deauthorize_public_key(&self, ctx: &AppContext) -> Result<()> {
        info!(remote = self.to_string(), "revoking public key");
        let mut session = SSHSession::new(&self.user, &self.password, &self.ip, self.port).await?;
        let pub_key = ctx.read_public_key()?;
        session.revoke(pub_key.as_str()).await?;
        info!(remote = self.to_string(), "public key revoked");
        Ok(())
    }

    pub async fn login(&self, ctx: &AppContext, reauth: bool) -> Result<()> {
        info!(remote = self.to_string(), "logging in");
        if !self.authorized || reauth {
            self.authorize_public_key(ctx).await?;
        }
        let key_path = ctx.private_key_path()?;
        let mut session =
            SSHSession::connect_with_key(&self.user, &key_path, &self.ip, self.port).await?;
        let result = session.interactive_login_shell().await;
        match result {
            Ok(()) => info!(remote = self.to_string(), "login session ended"),
            Err(e) => {
                warn!(remote = self.to_string(), error = %e, "login session terminated unexpectedly");
                return Err(e);
            }
        }
        Ok(())
    }

    pub async fn upload(&self, ctx: &AppContext, from: &str, to: &str) -> Result<()> {
        info!(from = from, to = to, remote = self.to_string(), "uploading");
        if !self.authorized {
            debug!(
                remote = self.to_string(),
                "not authorized, attempting authentication"
            );
            self.authorize_public_key(ctx).await?;
        }
        let key_path = ctx.private_key_path()?;
        let mut session =
            SSHSession::connect_with_key(&self.user, &key_path, &self.ip, self.port).await?;
        session.scp_upload(from, to).await?;
        info!(from = from, to = to, "uploaded");
        Ok(())
    }

    pub async fn download(&self, ctx: &AppContext, from: &str, to: &str) -> Result<()> {
        info!(
            from = from,
            to = to,
            remote = self.to_string(),
            "downloading"
        );
        let key_path = ctx.private_key_path()?;
        let mut session =
            SSHSession::connect_with_key(&self.user, &key_path, &self.ip, self.port).await?;
        session.scp_download(from, to).await?;
        info!(from = from, to = to, "downloaded");
        Ok(())
    }

    pub async fn run_command(&self, ctx: &AppContext, cmd: Option<&str>) -> Result<()> {
        info!(remote = self.to_string(), command = cmd, "running command");
        let key_path = ctx.private_key_path()?;
        let mut session =
            SSHSession::connect_with_key(&self.user, &key_path, &self.ip, self.port).await?;
        session.interactive_shell(80, 24, cmd).await?;
        info!(remote = self.to_string(), "command finished");
        Ok(())
    }
}

pub struct Remotes(pub Vec<Remote>);

impl Remotes {
    fn load(ctx: &AppContext) -> Result<Remotes> {
        let remotes = ctx.load_remotes()?;
        Ok(Remotes(remotes))
    }

    pub fn get(ctx: &AppContext, idx: usize) -> Result<Option<Remote>> {
        let remotes = ctx.load_remotes()?;
        let remote = remotes.into_iter().find(|r| r.index == idx);
        if remote.is_some() {
            debug!(index = idx, "found remote");
        } else {
            warn!(index = idx, "remote not found");
        }
        Ok(remote)
    }

    pub fn try_get(ctx: &AppContext, idx: usize) -> Result<Remote> {
        Remotes::get(ctx, idx)?.ok_or(AtshError::RemoteNotFound { index: idx })
    }

    pub fn get_all(ctx: &AppContext) -> Result<Remotes> {
        let remotes = Remotes::load(ctx)?;
        info!(count = remotes.0.len(), "loaded remotes");
        Ok(remotes)
    }

    pub fn add(
        ctx: &AppContext,
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
        remote.save(ctx)
    }

    pub async fn delete(ctx: &AppContext, indexs: &[usize]) -> Result<usize> {
        let remotes: Vec<Remote> = indexs
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .iter()
            .filter_map(|&idx| Remotes::get(ctx, *idx).transpose())
            .collect::<std::result::Result<Vec<_>, _>>()?;
        debug!(total = remotes.len(), "deleting remotes");
        for remote in remotes.iter() {
            debug!(index = remote.index, "deleting remote");
            if remote.authorized {
                if let Err(e) = remote.deauthorize_public_key(ctx).await {
                    warn!("failed to revoke auth for remote {}: {}", remote, e);
                }
            }
            remote.delete_from_db(ctx)?;
        }
        Ok(remotes.len())
    }
}

pub struct RemoteDisplay;

impl RemoteDisplay {
    pub fn pprint(ctx: &AppContext, remotes: &[Remote], all: bool) {
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

        let has_key = ctx.enc_key().is_some();
        for remote in remotes {
            let mut row = vec![
                remote.index.to_string(),
                remote.name.clone().unwrap_or_default(),
                remote.user.clone(),
                remote.ip.clone(),
                remote.port.to_string(),
            ];
            if all {
                let pwd_display = if has_key {
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
        debug!(total = remotes.len(), "printed remote list");
        table.printstd();
    }
}