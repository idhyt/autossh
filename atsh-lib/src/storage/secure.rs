use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::aead::generic_array::typenum::Unsigned;
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::ChaCha20Poly1305;

use crate::config::CONFIG;
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

pub fn encrypt(data: impl AsRef<str>) -> Result<String> {
    let data = data.as_ref();
    if let Ok(key) = CONFIG.get_enc_key() {
        let key = derive_key(&key);
        let obsf = chacha_encrypt(data, &key)?;
        Ok(general_purpose::STANDARD_NO_PAD.encode(obsf))
    } else {
        Ok(data.to_string())
    }
}

pub fn decrypt(data: impl AsRef<str>) -> Result<String> {
    let data = data.as_ref();
    if let Ok(key) = CONFIG.get_enc_key() {
        let obsf = general_purpose::STANDARD_NO_PAD
            .decode(data.as_bytes())
            .map_err(|_| AtshError::DecryptionFailed)?;
        let key = derive_key(&key);
        chacha_decrypt(&obsf, &key)
    } else {
        Ok(data.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chacha() {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let key_bytes: [u8; 32] = key.as_slice().try_into().unwrap();
        let ciphertext = chacha_encrypt("plaintext message", &key_bytes).unwrap();
        let plaintext = chacha_decrypt(&ciphertext, &key_bytes).unwrap();
        assert_eq!(plaintext, "plaintext message");
    }

    #[test]
    fn test_secure() {
        let key = "32 bit key must, if not, we will resize it.";
        unsafe { std::env::set_var("ASKEY", key) };
        let data = "hello world";
        let enc = encrypt(data).unwrap();
        let dec = decrypt(&enc).unwrap();
        assert_eq!(data, dec);
    }
}