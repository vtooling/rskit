//! Key derivation: HKDF-SHA256 (RFC 5869) and PBKDF2-SHA256 (RFC 8018).

use hkdf::Hkdf;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

/// HKDF-SHA256: expand `ikm` (input keying material) into `len` output bytes.
///
/// `salt` mixes the key; `info` binds the output to a context. Output length
/// must not exceed 255 × 32 (8160 bytes).
pub fn hkdf_sha256(salt: &[u8], ikm: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    assert!(len <= 255 * 32, "hkdf output too long");
    let mut okm = vec![0u8; len];
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    hk.expand(info, &mut okm).expect("length asserted above");
    okm
}

/// PBKDF2-HMAC-SHA256: derive `len` bytes from a password and salt over
/// `rounds` iterations.
pub fn pbkdf2_sha256(password: &[u8], salt: &[u8], rounds: u32, len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    pbkdf2_hmac::<Sha256>(password, salt, rounds, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    // RFC 5869 Test Case 1 (SHA-256).
    #[test]
    fn hkdf_rfc5869_case_1() {
        let ikm = hex!("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
        let salt = hex!("000102030405060708090a0b0c");
        let info = hex!("f0f1f2f3f4f5f6f7f8f9");
        let expected = hex!(
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
        );
        assert_eq!(hkdf_sha256(&salt, &ikm, &info, 42), expected.to_vec());
    }

    #[test]
    fn hkdf_deterministic() {
        let a = hkdf_sha256(b"salt", b"ikm", b"ctx", 32);
        let b = hkdf_sha256(b"salt", b"ikm", b"ctx", 32);
        assert_eq!(a, b);
        assert_ne!(a, hkdf_sha256(b"salt", b"ikm", b"different", 32));
    }

    #[test]
    fn hkdf_length_varies() {
        assert_eq!(hkdf_sha256(b"s", b"k", b"i", 16).len(), 16);
        assert_eq!(hkdf_sha256(b"s", b"k", b"i", 64).len(), 64);
    }

    #[test]
    fn pbkdf2_deterministic() {
        let a = pbkdf2_sha256(b"password", b"salt", 1000, 32);
        let b = pbkdf2_sha256(b"password", b"salt", 1000, 32);
        assert_eq!(a, b);
        assert_ne!(a, pbkdf2_sha256(b"different", b"salt", 1000, 32));
    }

    #[test]
    fn pbkdf2_length() {
        let out = pbkdf2_sha256(b"pw", b"nacl", 1, 40);
        assert_eq!(out.len(), 40);
    }

    #[test]
    fn pbkdf2_rounds_change_output() {
        let a = pbkdf2_sha256(b"pw", b"salt", 1, 32);
        let b = pbkdf2_sha256(b"pw", b"salt", 2, 32);
        assert_ne!(a, b);
    }
}
