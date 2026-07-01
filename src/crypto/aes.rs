use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use anyhow::{Result, anyhow};
use rand::Rng;

use aes_gcm::{AeadCore, Aes128Gcm, Aes256Gcm, Key, KeyInit, Nonce, aead::Aead, aead::OsRng};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const IV_LEN: usize = 16;

/// Generate a random alphanumeric string. `len = None` defaults to 16.
///
/// Thin wrapper over [`crate::str_util::random_alphanumeric`].
pub fn gen_rand_string(len: Option<usize>) -> String {
    crate::str_util::random_alphanumeric(len.unwrap_or(16))
}

/// Generate `n` random bytes.
pub fn gen_rand_bytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    rand::thread_rng().fill(&mut buf[..]);
    buf
}

fn cbc_iv(key: &[u8]) -> Result<&[u8]> {
    if key.len() < IV_LEN {
        return Err(anyhow!("cbc key too short; need >= {IV_LEN} bytes"));
    }
    Ok(&key[..IV_LEN])
}

/// AES-CBC-128 encrypt (key must be 16 bytes).
pub fn encrypt_aes_cbc_128(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 16 {
        return Err(anyhow!("aes-128 key must be 16 bytes, got {}", key.len()));
    }
    let iv = cbc_iv(key)?;
    Ok(Aes128CbcEnc::new(key.into(), iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data))
}

/// AES-CBC-128 decrypt (key must be 16 bytes).
pub fn decrypt_aes_cbc_128(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 16 {
        return Err(anyhow!("aes-128 key must be 16 bytes, got {}", key.len()));
    }
    let iv = cbc_iv(key)?;
    Aes128CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(data)
        .map_err(|e| anyhow!("aes-cbc-128 decrypt error: {e:?}"))
}

/// AES-CBC-256 encrypt (key must be 32 bytes).
pub fn encrypt_aes_cbc_256(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(anyhow!("aes-256 key must be 32 bytes, got {}", key.len()));
    }
    let iv = cbc_iv(key)?;
    Ok(Aes256CbcEnc::new(key.into(), iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data))
}

/// AES-CBC-256 decrypt (key must be 32 bytes).
pub fn decrypt_aes_cbc_256(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(anyhow!("aes-256 key must be 32 bytes, got {}", key.len()));
    }
    let iv = cbc_iv(key)?;
    Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(data)
        .map_err(|e| anyhow!("aes-cbc-256 decrypt error: {e:?}"))
}

pub fn aes_gcm_key_128() -> Vec<u8> {
    Aes128Gcm::generate_key(OsRng).to_vec()
}

pub fn aes_gcm_nonce_128() -> Vec<u8> {
    Aes128Gcm::generate_nonce(OsRng).to_vec()
}

pub fn aes_gcm_key_256() -> Vec<u8> {
    Aes256Gcm::generate_key(OsRng).to_vec()
}

pub fn aes_gcm_nonce_256() -> Vec<u8> {
    Aes256Gcm::generate_nonce(OsRng).to_vec()
}

/// AES-GCM-128 encrypt (key 16 bytes, nonce 12 bytes).
pub fn encrypt_aes_gcm_128(secret: &[u8], nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    if secret.len() != 16 {
        return Err(anyhow!("gcm-128 key must be 16 bytes"));
    }
    if nonce.len() != 12 {
        return Err(anyhow!("gcm nonce must be 12 bytes"));
    }
    let key = Key::<Aes128Gcm>::from_slice(secret);
    let nonce = Nonce::from_slice(nonce);
    let cipher = Aes128Gcm::new(key);
    cipher
        .encrypt(nonce, data)
        .map_err(|e| anyhow!("aes-gcm-128 encrypt error: {e:?}"))
}

/// AES-GCM-128 decrypt (key 16 bytes, nonce 12 bytes).
pub fn decrypt_aes_gcm_128(secret: &[u8], nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    if secret.len() != 16 {
        return Err(anyhow!("gcm-128 key must be 16 bytes"));
    }
    if nonce.len() != 12 {
        return Err(anyhow!("gcm nonce must be 12 bytes"));
    }
    let key = Key::<Aes128Gcm>::from_slice(secret);
    let nonce = Nonce::from_slice(nonce);
    let cipher = Aes128Gcm::new(key);
    cipher
        .decrypt(nonce, data)
        .map_err(|e| anyhow!("aes-gcm-128 decrypt error: {e:?}"))
}

/// AES-GCM-256 encrypt (key 32 bytes, nonce 12 bytes).
pub fn encrypt_aes_gcm_256(secret: &[u8], nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    if secret.len() != 32 {
        return Err(anyhow!("gcm-256 key must be 32 bytes"));
    }
    if nonce.len() != 12 {
        return Err(anyhow!("gcm nonce must be 12 bytes"));
    }
    let key = Key::<Aes256Gcm>::from_slice(secret);
    let nonce = Nonce::from_slice(nonce);
    let cipher = Aes256Gcm::new(key);
    cipher
        .encrypt(nonce, data)
        .map_err(|e| anyhow!("aes-gcm-256 encrypt error: {e:?}"))
}

/// AES-GCM-256 decrypt (key 32 bytes, nonce 12 bytes).
pub fn decrypt_aes_gcm_256(secret: &[u8], nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    if secret.len() != 32 {
        return Err(anyhow!("gcm-256 key must be 32 bytes"));
    }
    if nonce.len() != 12 {
        return Err(anyhow!("gcm nonce must be 12 bytes"));
    }
    let key = Key::<Aes256Gcm>::from_slice(secret);
    let nonce = Nonce::from_slice(nonce);
    let cipher = Aes256Gcm::new(key);
    cipher
        .decrypt(nonce, data)
        .map_err(|e| anyhow!("aes-gcm-256 decrypt error: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: usize) -> Vec<u8> {
        gen_rand_string(Some(n)).into_bytes()
    }

    #[test]
    fn gen_rand_string_default_len() {
        assert_eq!(gen_rand_string(None).len(), 16);
        assert_eq!(gen_rand_string(Some(32)).len(), 32);
    }

    #[test]
    fn gen_rand_bytes_len() {
        assert_eq!(gen_rand_bytes(8).len(), 8);
    }

    #[test]
    fn aes_cbc_128_roundtrip() {
        let k = key(16);
        let enc = encrypt_aes_cbc_128(&k, b"hello world").unwrap();
        let dec = decrypt_aes_cbc_128(&k, &enc).unwrap();
        assert_eq!(dec, b"hello world");
    }

    #[test]
    fn aes_cbc_256_roundtrip() {
        let k = key(32);
        let enc = encrypt_aes_cbc_256(&k, b"data").unwrap();
        assert_eq!(decrypt_aes_cbc_256(&k, &enc).unwrap(), b"data");
    }

    #[test]
    fn aes_gcm_128_roundtrip() {
        let k = aes_gcm_key_128();
        let n = aes_gcm_nonce_128();
        let enc = encrypt_aes_gcm_128(&k, &n, b"secret").unwrap();
        assert_eq!(decrypt_aes_gcm_128(&k, &n, &enc).unwrap(), b"secret");
    }

    #[test]
    fn aes_gcm_256_roundtrip() {
        let k = aes_gcm_key_256();
        let n = aes_gcm_nonce_256();
        let enc = encrypt_aes_gcm_256(&k, &n, b"secret").unwrap();
        assert_eq!(decrypt_aes_gcm_256(&k, &n, &enc).unwrap(), b"secret");
    }

    #[test]
    fn wrong_key_length_errors() {
        assert!(encrypt_aes_cbc_128(b"short", b"x").is_err());
        assert!(encrypt_aes_cbc_256(b"short", b"x").is_err());
        assert!(encrypt_aes_gcm_128(b"short", &[0u8; 12], b"x").is_err());
    }

    #[test]
    fn wrong_nonce_length_errors() {
        let k = aes_gcm_key_128();
        assert!(encrypt_aes_gcm_128(&k, b"short", b"x").is_err());
    }

    #[test]
    fn gcm_tampered_ciphertext_fails() {
        let k = aes_gcm_key_128();
        let n = aes_gcm_nonce_128();
        let mut enc = encrypt_aes_gcm_128(&k, &n, b"secret").unwrap();
        if let Some(b) = enc.last_mut() {
            *b ^= 0xff;
        }
        assert!(decrypt_aes_gcm_128(&k, &n, &enc).is_err());
    }
}
