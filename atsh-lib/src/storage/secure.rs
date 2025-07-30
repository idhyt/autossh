use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::aead::generic_array::typenum::Unsigned;
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::ChaCha20Poly1305;

use crate::config::CONFIG;

fn generate_key(key: Option<impl AsRef<str>>) -> Vec<u8> {
    if key.is_none() {
        return ChaCha20Poly1305::generate_key(&mut OsRng).to_vec();
    }
    let key = key.unwrap();
    let key = key.as_ref().as_bytes().to_vec();

    // the key must 32 bytes
    if key.len() > 32 {
        return key[..32].to_vec();
    } else if key.len() < 32 {
        // resize it
        let mut key = key;
        key.resize(32, 0);
        return key;
    }
    return key;
}

fn chacha_encrypt(cleartext: &str, key: &[u8]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(key));
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let mut obsf = cipher
        .encrypt(&nonce, cleartext.as_bytes())
        .expect("encrypt failed by chacha20");
    obsf.splice(..0, nonce.iter().copied());
    obsf
}

fn chacha_decrypt(obsf: &[u8], key: &[u8]) -> String {
    type NonceSize = <ChaCha20Poly1305 as AeadCore>::NonceSize;
    let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(key));
    let (nonce, ciphertext) = obsf.split_at(NonceSize::to_usize());
    let nonce = GenericArray::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .expect("decrypt failed by chacha20");
    String::from_utf8(plaintext).unwrap()
}

pub fn encrypt(data: impl AsRef<str>) -> String {
    let data = data.as_ref();
    if let Ok(key) = CONFIG.get_enc_key() {
        // log::debug!("we found `ASKEY` and will encrypt.");
        let key = generate_key(Some(&key));
        let obsf = chacha_encrypt(data, &key);
        general_purpose::STANDARD_NO_PAD.encode(obsf)
    } else {
        data.to_string()
    }
}

pub fn decrypt(data: impl AsRef<str>) -> String {
    let data = data.as_ref();
    if let Ok(key) = CONFIG.get_enc_key() {
        // log::debug!("we found `ASKEY` and will decrypt.");
        let obsf = general_purpose::STANDARD_NO_PAD
            .decode(data.as_bytes())
            .expect("decode failed by base64");
        let key = generate_key(Some(&key));
        chacha_decrypt(&obsf, &key)
    } else {
        data.to_string()
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chacha() {
        let key = generate_key(None::<&str>);
        println!("chacha key: {:?}", key);
        let ciphertext = chacha_encrypt("plaintext message", &key);
        println!("encrypt: {:?}", ciphertext);
        let plaintext = chacha_decrypt(&ciphertext, &key);
        println!("decrypt: {:?}", plaintext);
        assert_eq!(plaintext, "plaintext message");
    }

    #[test]
    fn test_secure() {
        // set env
        let key = generate_key(None::<&str>);
        println!("secure key: {:?}", key);
        let key = "32 bit key must, if not, we will resize it.";
        unsafe { std::env::set_var("ASKEY", key) };
        let key = generate_key(Some(key));
        println!("secure key: {:?}", key);
        let data = "hello world";
        let enc = encrypt(data);
        println!("encrypt: {:?}", enc);
        let dec = decrypt(&enc);
        println!("decrypt: {:?}", dec);
        assert_eq!(data, dec);
    }
}
