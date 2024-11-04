use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;

use home;
use ssh2::Session;

use super::server::Remote;

impl Remote {
    fn read_pub_key(&self) -> String {
        let pub_key = home::home_dir().unwrap().join(".ssh/id_rsa.pub");
        assert!(
            pub_key.is_file(),
            "public key not found at: {}, you can generate it by `ssh-keygen`",
            pub_key.display()
        );
        std::fs::read_to_string(pub_key).unwrap()
    }

    fn get_session(&self) -> Session {
        let tcp = TcpStream::connect(format!("{}:{}", self.ip, self.port)).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();
        sess.userauth_password(&self.user, &self.password).unwrap();
        assert!(sess.authenticated(), "auth failed");
        sess
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
                log::debug!(
                    "file not found in {} , with the scp_recv error: {}",
                    file,
                    e
                );
                None
            }
        }
    }

    fn write_file(&self, sess: &Session, file: &str, data: &str, mode: i32) {
        let mut channel = sess
            .scp_send(&PathBuf::from(file), mode, data.len() as u64, None)
            .unwrap();
        channel.write_all(data.as_bytes()).unwrap();
        channel.send_eof().unwrap();
        // channel.wait_close().unwrap();
        drop(channel);
        log::debug!("write file success: {}", file);
    }

    pub fn authorized(&mut self) {
        if self.authorized {
            return;
        }

        // check public key exist in $HOME/.ssh/id_rsa.pub
        let pub_key = self.read_pub_key();

        // Connect to the local SSH server
        let sess = self.get_session();

        // get remote server home dir
        let remote_home = self.read_exec(&sess, "echo $HOME");
        log::debug!("remote home: {}", remote_home);
        // println!("remote home: {}", remote_home);

        let remote_keys = format!("{}/.ssh/authorized_keys", remote_home);
        match self.read_file(&sess, &remote_keys) {
            Some(data) => {
                if data.contains(&pub_key) {
                    log::debug!("public key found in authorized_keys");
                    // println!("public key found in authorized_keys");
                    self.authorized = true;
                } else {
                    log::debug!("add the public key to authorized_keys");
                    // println!("public key not found in authorized_keys");
                    let mut data = data;
                    data.push_str("\n");
                    data.push_str(&pub_key);
                    self.write_file(&sess, &remote_keys, &data, 0o600);
                    self.authorized = true;
                }
            }
            None => {
                log::debug!(
                    "authorized_keys not found in {} and we will create one",
                    remote_keys
                );
                self.write_file(&sess, &remote_keys, &pub_key, 0o600);
                self.authorized = true;
            }
        }

        if !self.authorized {
            panic!("remote authorized failed for {}", self);
        }

        log::debug!("remote authorized success for {}", self);
    }

    pub fn revoke(&self) {
        if !self.authorized {
            log::debug!("remote not authorized, skip revoke");
            return;
        }

        let pub_key = self.read_pub_key();
        let session = self.get_session();
        let remote_home = self.read_exec(&session, "echo $HOME");
        let remote_keys = format!("{}/.ssh/authorized_keys", remote_home);
        match self.read_file(&session, &remote_keys) {
            Some(data) => {
                if data.contains(&pub_key) {
                    log::debug!("public key found in authorized_keys, we will revoke it");
                    let data = data.replace(&pub_key, "");
                    self.write_file(&session, &remote_keys, &data, 0o600);
                    log::debug!("remote revoke success for {}", self);
                } else {
                    log::debug!("public key not found in authorized_keys, skip revoke");
                }
            }
            None => {
                log::debug!("authorized_keys not found in {}, skip revoke", remote_keys);
            }
        }
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorized() {
        // ssh idhyt@192.168.0.1 -p 22
        let mut remote = Remote {
            index: 1,
            user: "idhyt".to_string(),
            password: "password".to_string(),
            ip: "192.168.0.1".to_string(),
            port: 22,
            name: Some("test".to_string()),
            note: Some("test".to_string()),
            authorized: false,
        };
        remote.authorized();
    }
}
