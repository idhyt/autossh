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

#[derive(Debug, PartialEq, Eq)]
pub enum AuthStatus {
    Add,
    Exist,
    Remove,
    NotFound,
    // Failure,
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
        Ok(output.trim().to_string())
    }

    pub async fn get_remote_home(&mut self) -> Result<String, Error> {
        let home = self.execute("echo $HOME").await?;
        debug!("remote home: {}", home);
        Ok(home)
    }

    pub async fn authenticate(
        &mut self,
        public_key: &str,
    ) -> Result<AuthStatus, Box<dyn std::error::Error>> {
        let command = format!(
            "mkdir -p {home}/.ssh && \
             touch {home}/.ssh/authorized_keys && \
             chmod 700 {home}/.ssh && \
             chmod 600 {home}/.ssh/authorized_keys && \
             if ! grep -qF '{key}' {home}/.ssh/authorized_keys; then \
                 echo '{key}' >> {home}/.ssh/authorized_keys && echo 'ADDED'; \
             else \
                 echo 'ALREADY_EXISTS'; \
             fi",
            home = self.get_remote_home().await?,
            key = public_key.replace('\'', "'\\''"),
        );

        match self.execute(&command).await?.as_str() {
            "ADDED" => Ok(AuthStatus::Add),
            "ALREADY_EXISTS" => Ok(AuthStatus::Exist),
            output => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("authentication with unexpected output: {}", output),
            ))),
        }
    }

    pub async fn revoke(
        &mut self,
        public_key: &str,
    ) -> Result<AuthStatus, Box<dyn std::error::Error>> {
        let command = format!(
            "if [ -f ~/.ssh/authorized_keys ]; then \
                 if grep -qF '{key}' {home}/.ssh/authorized_keys; then \
                     grep -vF '{key}' {home}/.ssh/authorized_keys > {home}/.ssh/authorized_keys.tmp && \
                     mv {home}/.ssh/authorized_keys.tmp {home}/.ssh/authorized_keys && \
                     chmod 600 {home}/.ssh/authorized_keys && \
                     echo 'REMOVED'; \
                 else \
                     echo 'NOT_FOUND'; \
                 fi; \
             else \
                 echo 'FILE_NOT_EXIST'; \
             fi",
            home = self.get_remote_home().await?,
            key = public_key.replace('\'', "'\\''"),
        );

        match self.execute(&command).await?.as_str() {
            "REMOVED" => Ok(AuthStatus::Remove),
            "FILE_NOT_EXIST" | "NOT_FOUND" => Ok(AuthStatus::NotFound),
            output => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("revoke with unexpected output: {}", output),
            ))),
        }
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

        let home = session.get_remote_home().await.unwrap();
        println!("remote home: {}", home);

        let output = session.authenticate(public_key).await.unwrap();
        assert_eq!(output, AuthStatus::Add);

        let output = session.authenticate(public_key).await.unwrap();
        assert_eq!(output, AuthStatus::Exist);

        let output = session.revoke(public_key).await.unwrap();
        assert_eq!(output, AuthStatus::Remove);

        let output = session.revoke(public_key).await.unwrap();
        assert_eq!(output, AuthStatus::NotFound);
    }
}
