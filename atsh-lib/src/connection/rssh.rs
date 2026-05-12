use russh::client::{self, Handle};
use russh::keys::ssh_key;
use russh::{ChannelMsg, Error};
use std::sync::Arc;
use tracing::debug;

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
    pub async fn new(
        username: &str,
        password: &str,
        host: &str,
        port: u16,
    ) -> Result<SSHSession, Error> {
        // first we check the remote server availability

        let config = Arc::new(client::Config::default());
        let handler = ClientHandler;
        let mut session = client::connect(config, (host, port), handler).await?;
        // match session.authenticate_password(username, password).await? {
        //     AuthResult::Success => {
        //         debug!("Auth success");
        //     }
        //     AuthResult::Failure {
        //         remaining_methods,
        //         partial_success,
        //     } => {
        //         error!("Auth failure: {:?}, {}", remaining_methods, partial_success);
        //         return Err(Error::NotAuthenticated);
        //     }
        // }
        session.authenticate_password(username, password).await?;
        Ok(SSHSession { session })
    }

    async fn execute(&mut self, command: &str) -> Result<String, Error> {
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
