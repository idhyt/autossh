use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::aead::generic_array::typenum::Unsigned;
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::ChaCha20Poly1305;

use crate::error::{AtshError, Result};

fn derive_key(key: &str) -> Vec<u8> {
    let key = key.as_bytes().to_vec();
    if key.len() > 32 {
        return key[..32].to_vec();
    } else if key.len() < 32 {
        let mut key = key;
        key.resize(32, 0);
        return key;
    }
    key
}

fn chacha_encrypt(cleartext: &str, key: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(key));
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let mut obsf = cipher
        .encrypt(&nonce, cleartext.as_bytes())
        .map_err(|e| AtshError::EncryptionFailed(e.to_string()))?;
    obsf.splice(..0, nonce.iter().copied());
    Ok(obsf)
}

fn chacha_decrypt(obsf: &[u8], key: &[u8]) -> Result<String> {
    type NonceSize = <ChaCha20Poly1305 as AeadCore>::NonceSize;
    if obsf.len() < NonceSize::to_usize() {
        return Err(AtshError::DecryptionFailed);
    }
    let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(key));
    let (nonce, ciphertext) = obsf.split_at(NonceSize::to_usize());
    let nonce = GenericArray::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AtshError::DecryptionFailed)?;
    String::from_utf8(plaintext).map_err(|_| AtshError::DecryptionFailed)
}

/// Encrypt `data` with the given encryption key.
///
/// Returns the base64-encoded ciphertext (nonce prepended).
/// If `key` is `None`, the data is returned as-is (plaintext warning).
pub fn encrypt(data: impl AsRef<str>, key: Option<&str>) -> Result<String> {
    let data = data.as_ref();
    match key {
        Some(k) => {
            let derived = derive_key(k);
            let obsf = chacha_encrypt(data, &derived)?;
            Ok(general_purpose::STANDARD_NO_PAD.encode(obsf))
        }
        None => {
            tracing::warn!("encrypt called without ATSH_KEY - storing plaintext");
            Ok(data.to_string())
        }
    }
}

/// Decrypt `data` with the given encryption key.
///
/// If `key` is `None`, the data is returned as-is.
pub fn decrypt(data: impl AsRef<str>, key: Option<&str>) -> Result<String> {
    let data = data.as_ref();
    match key {
        Some(k) => {
            let obsf = general_purpose::STANDARD_NO_PAD
                .decode(data.as_bytes())
                .map_err(|_| AtshError::DecryptionFailed)?;
            let derived = derive_key(k);
            chacha_decrypt(&obsf, &derived)
        }
        None => Ok(data.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chacha() {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let key_str = general_purpose::STANDARD_NO_PAD.encode(key.as_slice());
        let ciphertext = encrypt("plaintext message", Some(&key_str)).unwrap();
        let plaintext = decrypt(&ciphertext, Some(&key_str)).unwrap();
        assert_eq!(plaintext, "plaintext message");
    }

    #[test]
    fn test_plaintext_fallback() {
        let enc = encrypt("hello", None).unwrap();
        assert_eq!(enc, "hello");
        let dec = decrypt(&enc, None).unwrap();
        assert_eq!(dec, "hello");
    }

    #[test]
    fn test_wrong_key() {
        let enc = encrypt("secret", Some("my-strong-key-32chars-long!!")).unwrap();
        let result = decrypt(&enc, Some("wrong-key-32chars-long!!!!")).unwrap_err();
        assert!(matches!(result, AtshError::DecryptionFailed));
    }
}