use ssh2::Session;
use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use tracing::debug;

use crate::config::CONFIG;

pub struct SSHSession {
    session: Session,
}

impl SSHSession {
    pub fn new(user: &str, password: &str, ip: &str, port: u16) -> Result<SSHSession, Error> {
        let tcp = TcpStream::connect(format!("{ip}:{port}"))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        session.userauth_password(user, password)?;
        debug!(ip = ip, port = port, "create session success");
        Ok(SSHSession { session })
    }
    pub fn authenticate(&self) -> Result<(), Error> {
        // check public key exist in $HOME/.ssh/id_rsa.pub
        let pub_key = CONFIG.read_public_key()?;
        // get remote server home dir
        let remote_home = self.read_exec("echo $HOME")?;
        debug!("remote home: {}", remote_home);
        let remote_keys = format!("{}/.ssh/authorized_keys", remote_home);
        let file = Path::new(&remote_keys);
        match self.read_file(file)? {
            Some(data) => {
                if data.contains(&pub_key) {
                    debug!("public key exist in authorized_keys");
                } else {
                    debug!("add the public key to authorized_keys");
                    let mut data = data;
                    data.push_str("\n");
                    data.push_str(&pub_key);
                    self.write_file(file, &data, 0o600)?;
                }
            }
            None => {
                debug!(
                    "authorized_keys not found in {} and we will create one",
                    remote_keys
                );
                self.write_file(file, &pub_key, 0o600)?;
            }
        }
        debug!("remote authenticate success");
        Ok(())
    }

    pub fn revoke(&self) -> Result<(), Error> {
        let public_key = CONFIG.read_public_key()?;
        let remote_home = self.read_exec("echo $HOME")?;
        debug!("remote home: {}", remote_home);
        let remote_key = format!("{}/.ssh/authorized_keys", remote_home);
        let file = Path::new(&remote_key);
        match self.read_file(file)? {
            Some(data) => {
                if data.contains(&public_key) {
                    debug!("public key found in authorized_keys, we will revoke it");
                    let data = data.replace(&public_key, "");
                    self.write_file(&file, &data, 0o600)?;
                    debug!("remote revoke success");
                } else {
                    debug!("public key not found in authorized_keys, skip revoke");
                }
            }
            None => {
                debug!("authorized_keys not found in {}, skip revoke", remote_key);
            }
        }
        Ok(())
    }
    fn read_exec(&self, cmd: &str) -> Result<String, Error> {
        let mut channel = self.session.channel_session()?;
        channel.exec(cmd)?;
        let mut data = String::new();
        channel.read_to_string(&mut data)?;
        channel.wait_close()?;
        Ok(data.trim().to_string())
    }

    fn read_file(&self, file: &Path) -> Result<Option<String>, Error> {
        match self.session.scp_recv(file) {
            Ok(cs) => {
                let (mut channel, _) = cs;
                let mut data = String::new();
                channel.read_to_string(&mut data)?;
                Ok(Some(data))
            }
            Err(e) => {
                debug!(
                    file=?file,
                    error=?e,
                    "read file with error (not found or permission denied?)"
                );
                Ok(None)
            }
        }
    }

    fn write_file(&self, file: &Path, data: &str, mode: i32) -> Result<(), Error> {
        let mut channel = self.session.scp_send(file, mode, data.len() as u64, None)?;
        channel.write_all(data.as_bytes())?;
        channel.send_eof()?;
        drop(channel);
        debug!(file = ?file, "write file success");
        Ok(())
    }
}
