mod config;
mod connection;
mod context;
mod error;
mod storage;

pub use context::AppContext;
pub use error::{AtshError, Result};

pub mod atsh {
    use std::path::Path;
    use tracing::debug;

    use crate::connection::{Remote, RemoteDisplay, Remotes};
    use crate::connection::SSHSession;
    use crate::context::AppContext;
    use crate::error::{AtshError, Result};
    use crate::storage::log::setup_logging;

    pub fn initialize(work_dir: Option<impl AsRef<Path>>) -> Result<AppContext> {
        let ctx = match work_dir {
            Some(p) => AppContext::new(p)?,
            None => AppContext::default(),
        };
        setup_logging(ctx.work_dir())?;
        debug!("success initialize at {:?}", ctx.work_dir());
        Ok(ctx)
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
        Remotes::add(ctx, user, password, ip, port, name, note)
    }

    pub fn add_remote(ctx: &AppContext, remote: &Remote) -> Result<usize> {
        add(
            ctx,
            &remote.user,
            &remote.password,
            &remote.ip,
            remote.port,
            &remote.name,
            &remote.note,
        )
    }

    pub async fn remove(ctx: &AppContext, index: &[usize]) -> Result<usize> {
        Remotes::delete(ctx, index).await
    }

    pub fn get(ctx: &AppContext, index: usize) -> Result<Option<Remote>> {
        Remotes::get(ctx, index)
    }

    pub fn try_get(ctx: &AppContext, index: usize) -> Result<Remote> {
        Remotes::try_get(ctx, index)
    }

    pub fn get_all(ctx: &AppContext) -> Result<Vec<Remote>> {
        let remotes = Remotes::get_all(ctx)?;
        Ok(remotes.0)
    }

    pub fn pprint(ctx: &AppContext, all: bool) -> Result<()> {
        let remotes = Remotes::get_all(ctx)?;
        RemoteDisplay::pprint(ctx, &remotes.0, all);
        Ok(())
    }

    pub async fn login(ctx: &AppContext, index: usize, auth: bool) -> Result<()> {
        let remote = Remotes::try_get(ctx, index)?;
        remote.login(ctx, auth).await
    }

    pub async fn upload(ctx: &AppContext, index: usize, path: &[impl AsRef<str>]) -> Result<()> {
        if path.len() != 2 {
            return Err(AtshError::InvalidInput(
                "path format error, like `upload -p /local/path /remote/path`".into(),
            ));
        }
        let (local, remote) = (path[0].as_ref(), path[1].as_ref());
        if !Path::new(local).exists() {
            return Err(AtshError::InvalidInput("the upload file not found".into()));
        }
        Remotes::try_get(ctx, index)?.upload(ctx, local, remote).await
    }

    pub async fn download(ctx: &AppContext, index: usize, path: &[impl AsRef<str>]) -> Result<()> {
        if path.len() != 2 {
            return Err(AtshError::InvalidInput(
                "path format error, like `download -p /remote/path /local/path`".into(),
            ));
        }
        let (remote, local) = (path[0].as_ref(), path[1].as_ref());
        Remotes::try_get(ctx, index)?.download(ctx, remote, local).await
    }

    pub async fn run_command(
        ctx: &AppContext,
        index: Option<usize>,
        user: Option<&str>,
        password: Option<&str>,
        ip: Option<&str>,
        port: Option<u16>,
        cmd: Option<&str>,
        _wh: (Option<u32>, Option<u32>),
    ) -> Result<()> {
        if let Some(i) = index {
            let remote = Remotes::try_get(ctx, i)?;
            return remote.run_command(ctx, cmd).await;
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