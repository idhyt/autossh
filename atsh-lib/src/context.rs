use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
use crate::error::{AtshError, Result};
use crate::storage::db::Db;
use crate::storage::secure::{decrypt, encrypt};

/// Central application state, passed explicitly instead of global singletons.
///
/// Holds the resolved work directory, the loaded configuration, the encryption
/// key, and the SQLite database handle.
#[derive(Clone)]
pub struct AppContext {
    inner: Arc<Inner>,
}

struct Inner {
    work_dir: PathBuf,
    config: Config,
    enc_key: Option<String>,
    db: Mutex<Db>,
}

impl AppContext {
    /// Build a context rooted at the given work directory.
    pub fn new(work_dir: impl AsRef<Path>) -> Result<Self> {
        let work_dir = work_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&work_dir)?;

        let config = Config::load_or_default(&work_dir)?;
        let enc_key = Self::detect_enc_key();
        let db = Db::open(&work_dir.join("atsh.db"))?;

        Ok(Self {
            inner: Arc::new(Inner {
                work_dir,
                config,
                enc_key,
                db: Mutex::new(db),
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

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn enc_key(&self) -> Option<&str> {
        self.inner.enc_key.as_deref()
    }

    pub fn require_enc_key(&self) -> Result<&str> {
        self.enc_key().ok_or(AtshError::EncKeyNotSet)
    }

    pub fn db(&self) -> &Mutex<Db> {
        &self.inner.db
    }

    /// Encrypt a plaintext password using the configured key.
    pub fn encrypt_password(&self, plain: &str) -> Result<String> {
        encrypt(plain, self.enc_key())
    }

    /// Decrypt an encrypted password using the configured key.
    pub fn decrypt_password(&self, enc: &str) -> Result<String> {
        decrypt(enc, self.enc_key())
    }
}

impl Default for AppContext {
    fn default() -> Self {
        let dir = crate::config::default_work_dir();
        Self::new(dir).expect("failed to initialize app context")
    }
}