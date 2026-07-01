use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use rsa::{
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey},
};

/// Generate an RSA key pair, returning `(private_b64, public_b64)` (PKCS#8 DER,
/// base64-encoded). `bits = None` defaults to 1024.
pub fn generate_rsa_pair(bits: Option<usize>) -> Result<(String, String)> {
    let bits = bits.unwrap_or(1024);
    let mut rng = rsa::rand_core::OsRng;
    let pri =
        RsaPrivateKey::new(&mut rng, bits).map_err(|e| anyhow!("generate rsa key error: {e}"))?;
    let pub_key = RsaPublicKey::from(&pri);
    let pri_der = EncodePrivateKey::to_pkcs8_der(&pri)
        .map_err(|e| anyhow!("encode private key error: {e}"))?;
    let pub_der = EncodePublicKey::to_public_key_der(&pub_key)
        .map_err(|e| anyhow!("encode public key error: {e}"))?;
    Ok((
        BASE64_STANDARD.encode(pri_der.as_bytes()),
        BASE64_STANDARD.encode(pub_der.as_bytes()),
    ))
}

/// Encrypt with the public key (raw DER bytes).
pub fn encrypt_rsa_byte(pub_key_der: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let pk = RsaPublicKey::from_public_key_der(pub_key_der)
        .map_err(|e| anyhow!("invalid public key: {e}"))?;
    let mut rng = rsa::rand_core::OsRng;
    Ok(pk.encrypt(&mut rng, Pkcs1v15Encrypt, data)?)
}

/// Decrypt with the private key (raw PKCS#8 DER bytes).
pub fn decrypt_rsa_byte(pri_key_der: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let pk = RsaPrivateKey::from_pkcs8_der(pri_key_der)
        .map_err(|e| anyhow!("invalid private key: {e}"))?;
    Ok(pk.decrypt(Pkcs1v15Encrypt, data)?)
}

/// Encrypt with a base64-encoded public key (see [`generate_rsa_pair`]).
pub fn encrypt_rsa_base(pub_key_b64: &str, data: &[u8]) -> Result<Vec<u8>> {
    let der = BASE64_STANDARD
        .decode(pub_key_b64)
        .map_err(|e| anyhow!("base64 decode error: {e}"))?;
    encrypt_rsa_byte(&der, data)
}

/// Decrypt with a base64-encoded private key (see [`generate_rsa_pair`]).
pub fn decrypt_rsa_base(pri_key_b64: &str, data: &[u8]) -> Result<Vec<u8>> {
    let der = BASE64_STANDARD
        .decode(pri_key_b64)
        .map_err(|e| anyhow!("base64 decode error: {e}"))?;
    decrypt_rsa_byte(&der, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_pair_sizes() {
        let (pri, pub_b64) = generate_rsa_pair(Some(2048)).unwrap();
        assert!(!pri.is_empty());
        assert!(!pub_b64.is_empty());
    }

    #[test]
    fn rsa_roundtrip_base() {
        let (pri, pubk) = generate_rsa_pair(Some(2048)).unwrap();
        let enc = encrypt_rsa_base(&pubk, b"hello world").unwrap();
        let dec = decrypt_rsa_base(&pri, &enc).unwrap();
        assert_eq!(dec, b"hello world");
    }

    #[test]
    fn rsa_roundtrip_der() {
        let (pri, pubk) = generate_rsa_pair(Some(2048)).unwrap();
        let pri_der = BASE64_STANDARD.decode(pri).unwrap();
        let pub_der = BASE64_STANDARD.decode(pubk).unwrap();
        let enc = encrypt_rsa_byte(&pub_der, b"hi").unwrap();
        let dec = decrypt_rsa_byte(&pri_der, &enc).unwrap();
        assert_eq!(dec, b"hi");
    }

    #[test]
    fn invalid_key_errors() {
        assert!(encrypt_rsa_byte(b"not a key", b"x").is_err());
        assert!(decrypt_rsa_byte(b"not a key", b"x").is_err());
        assert!(encrypt_rsa_base("!!!notbase64", b"x").is_err());
    }
}
