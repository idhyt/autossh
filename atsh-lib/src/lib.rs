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

    use crate::connection::RemoteDisplay;
    use crate::context::AppContext;
    use crate::error::{AtshError, Result};
    use crate::storage::log::setup_logging;

    pub use crate::connection::{Remote, Remotes};

    pub fn initialize(work_dir: Option<impl AsRef<Path>>) -> Result<AppContext> {
        let ctx = match work_dir {
            Some(p) => AppContext::new(p)?,
            None => AppContext::default(),
        };
        setup_logging(ctx.work_dir())?;
        debug!(work_dir = ?ctx.work_dir(), "initialized");
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

    pub async fn remove(ctx: &AppContext, index: &[usize]) -> Result<usize> {
        Remotes::delete(ctx, index).await
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
        Remotes::try_get(ctx, index)?
            .upload(ctx, local, remote)
            .await
    }

    pub async fn download(ctx: &AppContext, index: usize, path: &[impl AsRef<str>]) -> Result<()> {
        if path.len() != 2 {
            return Err(AtshError::InvalidInput(
                "path format error, like `download -p /remote/path /local/path`".into(),
            ));
        }
        let (remote, local) = (path[0].as_ref(), path[1].as_ref());
        Remotes::try_get(ctx, index)?
            .download(ctx, remote, local)
            .await
    }

    pub async fn run_command(
        ctx: &AppContext,
        index: usize,
        cmd: &str,
        _wh: (Option<u32>, Option<u32>),
    ) -> Result<()> {
        let remote = Remotes::try_get(ctx, index)?;
        remote.run_command(ctx, Some(cmd)).await
    }

    pub fn get_remotes(ctx: &AppContext) -> Result<Remotes> {
        Remotes::get_all(ctx)
    }

    pub fn get_remote(ctx: &AppContext, index: usize) -> Result<Remote> {
        let all = get_remotes(ctx)?;
        all.0
            .into_iter()
            .find(|r| r.index == index)
            .ok_or(AtshError::RemoteNotFound { index })
    }
}
