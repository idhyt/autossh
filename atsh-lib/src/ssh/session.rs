use ssh2::Session;
use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use tracing::debug;

use super::remote::Remote;
use crate::config::CONFIG;

impl Remote {
    fn get_session(&self) -> Result<Session, Error> {
        let tcp = TcpStream::connect(format!("{}:{}", self.ip, self.port))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_password(&self.user, &self.password)?;
        debug!(remote = ?self, "remote connect success");
        Ok(sess)
    }

    fn read_exec(&self, sess: &Session, cmd: &str) -> String {
        let mut channel = sess.channel_session().unwrap();
        channel.exec(cmd).unwrap();
        let mut data = String::new();
        channel.read_to_string(&mut data).unwrap();
        channel.wait_close().unwrap();
        data.trim().to_string()
    }

    fn read_file(&self, sess: &Session, file: &str) -> Option<String> {
        match sess.scp_recv(&PathBuf::from(&PathBuf::from(file))) {
            Ok(cs) => {
                let (mut channel, _) = cs;
                let mut data = String::new();
                channel.read_to_string(&mut data).unwrap();
                Some(data)
            }
            Err(e) => {
                debug!(
                    "file not found in {} , with the scp_recv error: {}",
                    file, e
                );
                None
            }
        }
    }

    fn write_file(&self, sess: &Session, file: &str, data: &str, mode: i32) -> Result<(), Error> {
        let mut channel = sess.scp_send(&PathBuf::from(file), mode, data.len() as u64, None)?;
        channel.write_all(data.as_bytes())?;
        channel.send_eof()?;
        drop(channel);
        debug!(file = ?file, "write success");
        Ok(())
    }

    pub fn authorized(&self) -> Result<(), Error> {
        // check public key exist in $HOME/.ssh/id_rsa.pub
        let pub_key = CONFIG.read_public_key()?;
        // Connect to the local SSH server
        let sess = self.get_session()?;

        // get remote server home dir
        let remote_home = self.read_exec(&sess, "echo $HOME");
        debug!("remote home: {}", remote_home);

        let remote_keys = format!("{}/.ssh/authorized_keys", remote_home);
        match self.read_file(&sess, &remote_keys) {
            Some(data) => {
                if data.contains(&pub_key) {
                    debug!("public key exist in authorized_keys");
                } else {
                    debug!("add the public key to authorized_keys");
                    let mut data = data;
                    data.push_str("\n");
                    data.push_str(&pub_key);
                    self.write_file(&sess, &remote_keys, &data, 0o600)?;
                }
            }
            None => {
                debug!(
                    "authorized_keys not found in {} and we will create one",
                    remote_keys
                );
                self.write_file(&sess, &remote_keys, &pub_key, 0o600)?;
            }
        }
        debug!(remote = ?self, "remote authorized success");
        Ok(())
    }

    pub fn revoke(&self) -> Result<(), Error> {
        let pub_key = CONFIG.read_public_key()?;
        let session = self.get_session()?;
        let remote_home = self.read_exec(&session, "echo $HOME");
        let remote_keys = format!("{}/.ssh/authorized_keys", remote_home);
        match self.read_file(&session, &remote_keys) {
            Some(data) => {
                if data.contains(&pub_key) {
                    debug!("public key found in authorized_keys, we will revoke it");
                    let data = data.replace(&pub_key, "");
                    self.write_file(&session, &remote_keys, &data, 0o600)?;
                    debug!(remote = ?self, "remote revoke success");
                } else {
                    debug!("public key not found in authorized_keys, skip revoke");
                }
            }
            None => {
                debug!("authorized_keys not found in {}, skip revoke", remote_keys);
            }
        }
        Ok(())
    }
}
