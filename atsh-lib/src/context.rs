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
        std::env::var("ATSH_KEY").ok()
        // .or_else(|| std::env::var("ASKEY").ok())
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

    /// Encrypt a plaintext password. Requires `ATSH_KEY`.
    pub fn encrypt_password(&self, plain: &str) -> Result<String> {
        let key = self.require_enc_key()?;
        encrypt(plain, Some(key))
    }

    /// Decrypt an encrypted password. Requires `ATSH_KEY`.
    pub fn decrypt_password(&self, enc: &str) -> Result<String> {
        let key = self.require_enc_key()?;
        decrypt(enc, Some(key))
    }

    /// Read the local public key content.
    pub fn read_public_key(&self) -> Result<String> {
        self.inner.config.lock().unwrap().read_public()
    }

    /// Get the path to the local private key.
    pub fn private_key_path(&self) -> Result<String> {
        let path = self
            .inner
            .config
            .lock()
            .unwrap()
            .get_private()
            .to_path_buf();
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

    /// Load all remotes. Passwords are kept encrypted in memory;
    /// decrypt lazily via [`Self::decrypt_password`] when needed.
    pub fn load_remotes(&self) -> Result<Vec<Remote>> {
        let config = self.inner.config.lock().unwrap();
        Ok(config.remotes.clone())
    }

    /// Persist remotes as-is. Passwords are expected to already be encrypted
    /// (encryption happens at the point a plaintext is introduced, e.g. `add`).
    pub fn save_remotes(&self, remotes: &[Remote]) -> Result<()> {
        let mut config = self.inner.config.lock().unwrap();
        config.remotes = remotes.to_vec();
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
