use russh::client::{self, AuthResult, Handle};
use russh::keys::ssh_key;
use russh::{ChannelMsg, Disconnect};
use std::sync::Arc;
use tracing::{debug, error};

use crate::error::{AtshError, Result};

struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct SSHSession {
    session: Handle<ClientHandler>,
}

fn handle_auth_result(auth_result: AuthResult, method_name: &str) -> Result<()> {
    match auth_result {
        AuthResult::Success => {
            debug!("authenticated ({})", method_name);
            Ok(())
        }
        AuthResult::Failure {
            remaining_methods,
            partial_success,
        } => {
            let msg = format!(
                "authentication ({}) failed, remaining_methods: {:?}, partial_success: {}",
                method_name, remaining_methods, partial_success
            );
            error!("{}", msg);
            Err(AtshError::AuthFailed(msg))
        }
    }
}

impl SSHSession {
    async fn connect(host: &str, port: u16) -> Result<client::Handle<ClientHandler>> {
        let config = client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(5 * 60)),
            ..<_>::default()
        };
        let config = Arc::new(config);
        let handler = ClientHandler;
        Ok(client::connect(config, (host, port), handler).await?)
    }

    pub async fn connect_with_key(
        username: &str,
        private_key_path: &str,
        host: &str,
        port: u16,
    ) -> Result<SSHSession> {
        use russh::keys::PrivateKeyWithHashAlg;

        let key_content = std::fs::read_to_string(private_key_path)
            .map_err(|_| AtshError::SshKeyNotFound {
                key_type: "private".into(),
                path: private_key_path.into(),
            })?;
        let key_pair = russh::keys::decode_secret_key(&key_content, None)
            .map_err(|e| AtshError::Config(format!("invalid private key: {}", e)))?;

        let mut session = Self::connect(host, port).await?;
        let hash_alg = session.best_supported_rsa_hash().await?.flatten();
        let auth_result = session
            .authenticate_publickey(
                username,
                PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg),
            )
            .await?;
        handle_auth_result(auth_result, "publickey")?;
        Ok(SSHSession { session })
    }

    pub async fn new(username: &str, password: &str, host: &str, port: u16) -> Result<SSHSession> {
        let mut session = Self::connect(host, port).await?;
        let auth_result = session.authenticate_password(username, password).await?;
        handle_auth_result(auth_result, "password")?;
        Ok(SSHSession { session })
    }

    pub async fn execute(&mut self, command: &str) -> Result<String> {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut output = String::new();
        loop {
            match channel.wait().await {
                Some(ChannelMsg::Data { ref data }) => {
                    output.push_str(&String::from_utf8_lossy(data));
                }
                Some(ChannelMsg::ExitStatus { .. }) => break,
                None => break,
                _ => {}
            }
        }
        let output = output.trim().to_string();
        debug!(command = command, "executed command");
        Ok(output)
    }

    pub async fn get_home(&mut self) -> Result<String> {
        self.execute("echo $HOME").await
    }

    async fn file_contains(&mut self, f: &str, s: &str) -> Result<bool> {
        let escaped = s.replace('\'', "'\\''");
        let command = format!(
            "[ -f {f} ] && grep -qF '{escaped}' {f} && echo Y || echo N"
        );
        Ok(self.execute(&command).await? == "Y")
    }

    pub async fn authenticate(&mut self, public_key: &str) -> Result<()> {
        let auth_home = self.get_home().await?;
        let auth_key = format!("{auth_home}/.ssh/authorized_keys");
        let pkey = public_key.trim();

        let setup = format!(
            "[ -d {ah}/.ssh ] || (mkdir -p {ah}/.ssh && chmod 700 {ah}/.ssh); \
             [ -f {ak} ] || (touch {ak} && chmod 600 {ak})",
            ah = auth_home,
            ak = auth_key
        );
        self.execute(&setup).await?;

        if self.file_contains(&auth_key, pkey).await? {
            debug!("public key already present in authorized_keys, skipping");
            return Ok(());
        }

        let escaped_key = pkey.replace('\'', "'\\''");
        let command = format!("echo '{escaped_key}' >> {ak}", ak = auth_key);
        self.execute(&command).await?;

        if !self.file_contains(&auth_key, pkey).await? {
            return Err(AtshError::AuthorizeFailed(
                "failed to add public key to authorized_keys".into(),
            ));
        }
        Ok(())
    }

    pub async fn revoke(&mut self, public_key: &str) -> Result<()> {
        let auth_home = self.get_home().await?;
        let auth_key = format!("{auth_home}/.ssh/authorized_keys");
        let pkey = public_key.trim();

        if !self.file_contains(&auth_key, pkey).await? {
            debug!("public key not found in authorized_keys, nothing to revoke");
            return Ok(());
        }

        let escaped_key = pkey.replace('\'', "'\\''");
        let command = format!(
            "grep -vF '{escaped_key}' {ak} > {ak}.tmp && mv {ak}.tmp {ak} && chmod 600 {ak}",
            ak = auth_key
        );
        self.execute(&command).await?;

        if self.file_contains(&auth_key, pkey).await? {
            return Err(AtshError::RevokeFailed(
                "failed to revoke public key from authorized_keys".into(),
            ));
        }
        Ok(())
    }

    pub async fn scp_upload(
        &mut self,
        local_path: &str,
        remote_path: &str,
    ) -> Result<()> {
        use russh_sftp::client::SftpSession;
        use tokio::io::AsyncWriteExt;

        let channel = self.session.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await?;
        let sftp = SftpSession::new(channel.into_stream()).await?;

        let remote_path = resolve_remote_path(&sftp, local_path, remote_path).await;
        let data = tokio::fs::read(local_path).await?;
        let mut file = sftp.create(&remote_path).await?;
        file.write_all(&data).await?;
        file.flush().await?;
        file.shutdown().await?;

        debug!(local = local_path, remote = remote_path, "upload complete");
        Ok(())
    }

    pub async fn scp_download(
        &mut self,
        remote_path: &str,
        local_path: &str,
    ) -> Result<()> {
        use russh_sftp::client::SftpSession;
        use tokio::io::AsyncReadExt;

        let channel = self.session.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await?;
        let sftp = SftpSession::new(channel.into_stream()).await?;

        let local_path = resolve_local_path(local_path, remote_path);
        let mut file = sftp.open(remote_path).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        tokio::fs::write(&local_path, data).await?;

        debug!(local = local_path, remote = remote_path, "download complete");
        Ok(())
    }

    pub async fn interactive_shell(
        &mut self,
        width: u32,
        height: u32,
        cmd: Option<&str>,
    ) -> Result<()> {
        self.run_pty(width, height, cmd, true).await
    }

    pub async fn interactive_login_shell(&mut self) -> Result<()> {
        let (width, height) = term_size::dimensions().unwrap_or((80, 24));
        self.run_pty(width as u32, height as u32, None, false)
            .await
    }

    /// Shared PTY-based bidirectional I/O loop.
    ///
    /// Opens a PTY, optionally executes a command (or requests a shell),
    /// then pumps stdin -> channel and channel -> stdout until exit.
    /// When `check_exit_status` is true, a non-zero exit status becomes an error.
    async fn run_pty(
        &mut self,
        width: u32,
        height: u32,
        cmd: Option<&str>,
        check_exit_status: bool,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        use tokio::sync::mpsc;

        let mut channel = self.session.channel_open_session().await?;
        channel
            .request_pty(
                true,
                &std::env::var("TERM").unwrap_or_else(|_| "xterm".into()),
                width,
                height,
                0,
                0,
                &[],
            )
            .await?;

        if let Some(command) = cmd {
            channel.exec(true, command).await?;
        } else {
            channel.request_shell(true).await?;
        }

        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);
        let mut stdout = tokio::io::stdout();
        let mut stdin_closed = false;

        // Use a detached blocking thread for stdin to avoid the tokio blocking
        // pool lingering after the remote session ends.
        let stdin_tx = tx.clone();
        std::thread::spawn(move || {
            use std::io::Read;
            let mut handle = std::io::stdin().lock();
            let mut buf = [0u8; 4096];
            loop {
                match handle.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if stdin_tx.blocking_send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let mut exit_status: u32 = 0;
        loop {
            tokio::select! {
                data = rx.recv() => {
                    match data {
                        Some(buf) => {
                            channel.data(&mut std::io::Cursor::new(&buf[..])).await?;
                        }
                        None => {
                            if !stdin_closed {
                                stdin_closed = true;
                                channel.eof().await?;
                            }
                        }
                    }
                }
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { data }) => {
                            stdout.write_all(&data).await?;
                            stdout.flush().await?;
                        }
                        Some(ChannelMsg::ExitStatus { exit_status: es }) => {
                            exit_status = es;
                            if !stdin_closed {
                                channel.eof().await?;
                            }
                            break;
                        }
                        None => break,
                        _ => {}
                    }
                }
            }
        }

        // Cancel the stdin reader before returning, otherwise the process can
        // hang waiting for one more keypress after the remote shell exits.
        debug!("pty loop exited, disconnecting session");
        self.session
            .disconnect(Disconnect::ByApplication, "Session ended", "en")
            .await?;
        debug!("session disconnected");

        if check_exit_status && exit_status != 0 {
            return Err(AtshError::Ssh(russh::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("exit status: {}", exit_status),
            ))));
        }
        Ok(())
    }
}

fn extract_filename(path: &str) -> &str {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
}

async fn resolve_remote_path(
    sftp: &russh_sftp::client::SftpSession,
    local_path: &str,
    remote_path: &str,
) -> String {
    if remote_path.ends_with('/')
        || sftp
            .metadata(remote_path)
            .await
            .map(|m| m.is_dir())
            .unwrap_or(false)
    {
        let name = extract_filename(local_path);
        format!("{}/{}", remote_path.trim_end_matches('/'), name)
    } else {
        remote_path.to_string()
    }
}

fn resolve_local_path(local_path: &str, remote_path: &str) -> String {
    if local_path.ends_with('/') || std::path::Path::new(local_path).is_dir() {
        let name = extract_filename(remote_path);
        format!("{}/{}", local_path.trim_end_matches('/'), name)
    } else {
        local_path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires a reachable SSH server"]
    async fn test_ssh() {
        let remote = "192.168.1.11";
        let port = 22;
        let username = "idhyt";
        let password = "12345678";
        let public_key = "test,public.key1111111";
        let conn = SSHSession::new(username, password, remote, port).await;
        if let Err(e) = &conn {
            println!("session error: {:#?}", e);
            panic!("failed to connect");
        }
        let mut session = conn.unwrap();

        let home = session.get_home().await.unwrap();
        println!("remote home: {}", home);

        session.authenticate(public_key).await.unwrap();
        session.authenticate(public_key).await.unwrap();
        session.revoke(public_key).await.unwrap();
        session.revoke(public_key).await.unwrap();
    }
}