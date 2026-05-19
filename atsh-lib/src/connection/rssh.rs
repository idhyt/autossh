use russh::client::{self, AuthResult, Handle};
use russh::keys::ssh_key;
// use russh::keys::{decode_secret_key, PrivateKeyWithHashAlg};
use russh::{ChannelMsg, Error};
use std::sync::Arc;
use tracing::{debug, error};

struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct SSHSession {
    session: Handle<ClientHandler>,
}

impl SSHSession {
    async fn connect(host: &str, port: u16) -> Result<client::Handle<ClientHandler>, Error> {
        let config = client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(5 * 60)),
            ..<_>::default()
        };

        let config = Arc::new(config);
        let handler = ClientHandler;
        client::connect(config, (host, port), handler)
            .await
            .map_err(Error::from)
    }

    fn handle_auth_result(auth_result: AuthResult, method_name: &str) -> Result<(), Error> {
        match auth_result {
            AuthResult::Success => {
                debug!("Authentication (with {}) succeeded", method_name);
                Ok(())
            }
            AuthResult::Failure {
                remaining_methods,
                partial_success,
            } => {
                error!(
                    "Authentication (with {}) failed, remaining_methods: {:?}, partial_success: {}",
                    method_name, remaining_methods, partial_success
                );
                Err(Error::UnsupportedAuthMethod)
            }
        }
    }

    pub async fn new(
        username: &str,
        password: &str,
        host: &str,
        port: u16,
    ) -> Result<SSHSession, Error> {
        let mut session = Self::connect(host, port).await?;
        let auth_result = session.authenticate_password(username, password).await?;
        Self::handle_auth_result(auth_result, "password")?;
        Ok(SSHSession { session })
    }

    // pub async fn new_with_key(
    //     username: &str,
    //     private_key: &str,
    //     host: &str,
    //     port: u16,
    // ) -> Result<SSHSession, Error> {
    //     let key_pair = decode_secret_key(private_key, None)?;
    //     // let key_pair = russh::keys::load_secret_key(private_key, None)?;
    //     // println!("key_pair: {:#?}", key_pair);
    //     let mut session = Self::connect(host, port).await?;
    //     let auth_result = session
    //         .authenticate_publickey(
    //             username,
    //             PrivateKeyWithHashAlg::new(
    //                 Arc::new(key_pair),
    //                 session.best_supported_rsa_hash().await?.flatten(),
    //             ),
    //         )
    //         .await?;
    //     Self::handle_auth_result(auth_result, "publickey")?;
    //     Ok(SSHSession { session })
    // }

    pub async fn execute(&mut self, command: &str) -> Result<String, Error> {
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
        debug!("executed: {}, output: {}", command, output);
        Ok(output)
    }

    pub async fn get_home(&mut self) -> Result<String, Error> {
        self.execute("echo $HOME").await
    }

    async fn file_contains(&mut self, f: &str, s: &str) -> Result<bool, Error> {
        let command = format!(
            "[ -f {f} ] && grep -qF '{s}' {f} && echo Y || echo N",
            f = f,
            s = s.replace('\'', "'\\''")
        );
        Ok(self.execute(&command).await? == "Y")
    }

    pub async fn authenticate(
        &mut self,
        public_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let auth_home = self.get_home().await?;
        let auth_key = format!("{auth_home}/.ssh/authorized_keys");
        let pkey = public_key.trim();

        let command = format!(
            "[ -d {ah}/.ssh ] || (mkdir -p {ah}/.ssh && chmod 700 {ah}/.ssh); \
             [ -f {ak} ] || (touch {ak} && chmod 600 {ak})",
            ah = auth_home,
            ak = auth_key
        );
        self.execute(&command).await?;

        if self.file_contains(&auth_key, pkey).await? {
            debug!("public key already exists in authorized_keys, skipping");
            return Ok(());
        }

        // add public_key to authorized_keys
        let command = format!(
            "echo '{pk}' >> {ak}",
            ak = auth_key,
            pk = pkey.replace('\'', "'\\''")
        );
        self.execute(&command).await?;

        // check the file contains the public key
        if !self.file_contains(&auth_key, pkey).await? {
            return Err("failed to add public key to authorized_keys".into());
        }

        Ok(())
    }

    pub async fn revoke(&mut self, public_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let auth_home = self.get_home().await?;
        let auth_key = format!("{auth_home}/.ssh/authorized_keys");
        let pkey = public_key.trim();

        if !self.file_contains(&auth_key, pkey).await? {
            debug!("public key not found in authorized_keys, skipping");
            return Ok(());
        }

        let command = format!(
            "grep -vF '{pk}' {ak} > {ak}.tmp && mv {ak}.tmp {ak} && chmod 600 {ak}",
            ak = auth_key,
            pk = pkey.replace('\'', "'\\''"),
        );
        self.execute(&command).await?;

        // check the file contains the public key
        if self.file_contains(&auth_key, pkey).await? {
            return Err("failed to revoke public key from authorized_keys".into());
        }

        Ok(())
    }

    // https://github.com/Eugeny/russh/blob/main/russh/examples/client_exec_interactive.rs
    pub async fn interactive_shell(
        &mut self,
        width: u32,
        height: u32,
        cmd: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use russh::Disconnect;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::sync::mpsc;

        let mut channel = self.session.channel_open_session().await?;
        channel
            .request_pty(
                true,
                &std::env::var("TERM").unwrap_or("xterm".into()),
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
        // 生成一个异步任务，专门读取标准输入，并把数据通过 tx 发送
        let mut stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut stdin_closed = false;

        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            loop {
                match stdin.read(&mut buf).await {
                    Ok(0) => break, // 标准输入关闭
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).await.is_err() {
                            break; // 接收端已关闭
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        loop {
            tokio::select! {
                // 从异步任务收到的用户输入
                data = rx.recv() => {
                    match data {
                        Some(buf) => {
                            channel.data(&mut std::io::Cursor::new(&buf[..])).await?;
                        }
                        None => {
                            // 发送端关闭意味着标准输入已结束
                            if !stdin_closed {
                                stdin_closed = true;
                                channel.eof().await?;
                            }
                        }
                    }
                }

                // 来自 SSH 服务端的事件
                msg = channel.wait() => {
                    match msg {
                        Some(russh::ChannelMsg::Data { data }) => {
                            stdout.write_all(&data).await?;
                            stdout.flush().await?;
                        }
                        Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                            debug!("Interactive shell exit code: {:?}", exit_status);
                            if !stdin_closed {
                                channel.eof().await?;
                            }
                            break;
                        }
                        None => {
                            // 通道关闭
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        self.session
            .disconnect(Disconnect::ByApplication, "Session ended", "en")
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ssh() {
        let remote = "192.168.1.11";
        let port = 22;
        let username = "idhyt";
        let password = "12345678";
        let public_key = "test,public.key1111111";
        let conn = SSHSession::new(username, password, remote, port).await;
        if let Err(e) = &conn {
            println!("session error: {:#?}", e);
            assert!(false);
        }
        let mut session = conn.unwrap();

        let home = session.get_home().await.unwrap();
        println!("remote home: {}", home);

        let output = session.authenticate(public_key).await;
        assert!(output.is_ok());

        let output = session.authenticate(public_key).await;
        assert!(output.is_ok());

        let output = session.revoke(public_key).await;
        assert!(output.is_ok());

        let output = session.revoke(public_key).await;
        assert!(output.is_ok());
    }
}
