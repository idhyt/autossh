use thiserror::Error;

#[derive(Error, Debug)]
pub enum AtshError {
    #[error("SSH error: {0}")]
    Ssh(#[from] russh::Error),

    #[error("SSH Sftp error: {0}")]
    Sftp(#[from] russh_sftp::client::error::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Remote #{index} not found")]
    RemoteNotFound { index: usize },

    #[error("ATSH_KEY not set. Export `ATSH_KEY` to protect password")]
    EncKeyNotSet,

    #[error("Decryption failed: data corrupted or wrong ATSH_KEY")]
    DecryptionFailed,

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("SSH key not found: {key_type} at {path}")]
    SshKeyNotFound { key_type: String, path: String },

    #[error("SSH keygen failed: {0}")]
    SshKeygenFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, AtshError>;