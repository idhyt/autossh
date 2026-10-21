mod config;
mod connection;
mod error;
mod storage;

pub use error::{AtshError, Result};

pub mod atsh {
    use std::path::Path;

    use tracing::debug;

    use crate::config::CONFIG;
    use crate::connection::{Remote as RemoteType, RemoteDisplay, Remotes};
    use crate::connection::SSHSession;
    use crate::error::{AtshError, Result};
    use crate::storage::log::setup_logging;

    pub use crate::config::CONFIG as Config;
    pub use crate::connection::Remote;

    pub fn initialize(work_dir: Option<impl AsRef<Path>>) -> Result<()> {
        if let Some(p) = work_dir {
            CONFIG.set_work_dir(p)?;
        }
        let w = CONFIG.get_work_dir();
        setup_logging(w)?;
        debug!("success initialize at {:?}", w);
        Ok(())
    }

    pub fn add(
        user: &str,
        password: &str,
        ip: &str,
        port: u16,
        name: &Option<impl AsRef<str>>,
        note: &Option<impl AsRef<str>>,
    ) -> Result<usize> {
        Remotes::add(user, password, ip, port, name, note)
    }

    pub fn add_remote(remote: &RemoteType) -> Result<usize> {
        add(
            &remote.user,
            &remote.password,
            &remote.ip,
            remote.port,
            &remote.name,
            &remote.note,
        )
    }

    pub async fn remove(index: &[usize]) -> Result<usize> {
        Remotes::delete(index).await
    }

    pub fn get(index: usize) -> Result<Option<RemoteType>> {
        Remotes::get(index)
    }

    pub fn try_get(index: usize) -> Result<RemoteType> {
        Remotes::try_get(index)
    }

    pub fn get_all() -> Result<Vec<RemoteType>> {
        let remotes = Remotes::get_all()?;
        Ok(remotes.0)
    }

    pub fn pprint(all: bool) -> Result<()> {
        let remotes = Remotes::get_all()?;
        RemoteDisplay::pprint(&remotes.0, all);
        Ok(())
    }

    pub async fn login(index: usize, auth: bool) -> Result<()> {
        let remote = Remotes::try_get(index)?;
        remote.login(auth).await
    }

    pub async fn upload(index: usize, path: &[impl AsRef<str>]) -> Result<()> {
        if path.len() != 2 {
            return Err(AtshError::InvalidInput(
                "path format error, like `upload -p /local/path /remote/path`".into(),
            ));
        }
        let (local, remote) = (path[0].as_ref(), path[1].as_ref());
        if !Path::new(local).exists() {
            return Err(AtshError::InvalidInput("the upload file not found".into()));
        }
        Remotes::try_get(index)?.upload(local, remote).await
    }

    pub async fn download(index: usize, path: &[impl AsRef<str>]) -> Result<()> {
        if path.len() != 2 {
            return Err(AtshError::InvalidInput(
                "path format error, like `download -p /remote/path /local/path`".into(),
            ));
        }
        let (remote, local) = (path[0].as_ref(), path[1].as_ref());
        Remotes::try_get(index)?.download(remote, local).await
    }

    pub async fn run_command(
        index: Option<usize>,
        user: Option<&str>,
        password: Option<&str>,
        ip: Option<&str>,
        port: Option<u16>,
        cmd: Option<&str>,
        _wh: (Option<u32>, Option<u32>),
    ) -> Result<()> {
        if let Some(i) = index {
            let remote = Remotes::try_get(i)?;
            return remote.run_command(cmd).await;
        }

        let mut session = SSHSession::new(
            user.ok_or_else(|| AtshError::InvalidInput("user is required".into()))?,
            password.ok_or_else(|| AtshError::InvalidInput("password is required".into()))?,
            ip.ok_or_else(|| AtshError::InvalidInput("ip is required".into()))?,
            port.ok_or_else(|| AtshError::InvalidInput("port is required".into()))?,
        )
        .await?;
        session.interactive_shell(80, 24, cmd).await?;
        Ok(())
    }
}