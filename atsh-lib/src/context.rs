use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::connection::Remote;
use crate::error::{AtshError, Result};
use crate::storage::secure::{decrypt, encrypt};

/// Central application state, passed explicitly instead of global singletons.
///
/// Holds the resolved work directory, the loaded configuration (including the
/// remote records stored in `config.toml`), and the encryption key.
#[derive(Clone)]
pub struct AppContext {
    inner: Arc<Inner>,
}

struct Inner {
    work_dir: PathBuf,
    config: Mutex<Config>,
    enc_key: Option<String>,
}

impl AppContext {
    /// Build a context rooted at the given work directory.
    pub fn new(work_dir: impl AsRef<Path>) -> Result<Self> {
        let work_dir = work_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&work_dir)?;

        let config = Config::load_or_default(&work_dir)?;
        let enc_key = Self::detect_enc_key();

        Ok(Self {
            inner: Arc::new(Inner {
                work_dir,
                config: Mutex::new(config),
                enc_key,
            }),
        })
    }

    fn detect_enc_key() -> Option<String> {
        std::env::var("ATSH_KEY")
            .ok()
            .or_else(|| std::env::var("ASKEY").ok())
    }

    pub fn work_dir(&self) -> &Path {
        &self.inner.work_dir
    }

    pub fn work_dir_file(&self, name: &str) -> PathBuf {
        self.inner.work_dir.join(name)
    }

    pub fn enc_key(&self) -> Option<&str> {
        self.inner.enc_key.as_deref()
    }

    pub fn require_enc_key(&self) -> Result<&str> {
        self.enc_key().ok_or(AtshError::EncKeyNotSet)
    }

    /// Encrypt a plaintext password using the configured key.
    pub fn encrypt_password(&self, plain: &str) -> Result<String> {
        encrypt(plain, self.enc_key())
    }

    /// Decrypt an encrypted password using the configured key.
    pub fn decrypt_password(&self, enc: &str) -> Result<String> {
        decrypt(enc, self.enc_key())
    }

    /// Read the local public key content.
    pub fn read_public_key(&self) -> Result<String> {
        self.inner.config.lock().unwrap().read_public()
    }

    /// Get the path to the local private key.
    pub fn private_key_path(&self) -> Result<String> {
        let path = self.inner.config.lock().unwrap().get_private().to_path_buf();
        path.to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AtshError::InvalidInput("invalid private key path".into()))
    }

    /// Generate a new SSH key pair.
    pub fn create_sshkey(
        &self,
        password: Option<impl AsRef<str>>,
        output: Option<impl AsRef<Path>>,
        interactive: bool,
    ) -> Result<PathBuf> {
        self.inner
            .config
            .lock()
            .unwrap()
            .create_sshkey(password, output, interactive)
    }

    /// Load all remotes, decrypting passwords.
    pub fn load_remotes(&self) -> Result<Vec<Remote>> {
        let config = self.inner.config.lock().unwrap();
        let mut remotes = config.remotes.clone();
        drop(config);
        for r in remotes.iter_mut() {
            r.password = self.decrypt_password(&r.password)?;
        }
        Ok(remotes)
    }

    /// Persist remotes, encrypting passwords.
    pub fn save_remotes(&self, remotes: &[Remote]) -> Result<()> {
        let mut config = self.inner.config.lock().unwrap();
        let mut out: Vec<Remote> = Vec::with_capacity(remotes.len());
        for r in remotes {
            let mut c = r.clone();
            c.password = self.encrypt_password(&c.password)?;
            out.push(c);
        }
        config.remotes = out;
        drop(config);
        self.save_config()
    }

    /// Persist the current in-memory config (including remotes) to disk.
    fn save_config(&self) -> Result<()> {
        let config = self.inner.config.lock().unwrap();
        let toml = toml::to_string(&*config).map_err(|e| AtshError::Config(e.to_string()))?;
        std::fs::write(self.work_dir_file("config.toml"), toml)?;
        Ok(())
    }
}

impl Default for AppContext {
    fn default() -> Self {
        let dir = crate::config::default_work_dir();
        Self::new(dir).expect("failed to initialize app context")
    }
}
