//! HMAC-SHA256 / HMAC-SHA512 message authentication.

use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512};

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

/// Compute HMAC-SHA256, returning raw bytes.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("any key length is valid for HMAC-SHA256");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Compute HMAC-SHA256 as a lowercase hex string.
pub fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    hex::encode(hmac_sha256(key, data))
}

/// Compute HMAC-SHA512, returning raw bytes.
pub fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha512::new_from_slice(key).expect("any key length is valid for HMAC-SHA512");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Compute HMAC-SHA512 as a lowercase hex string.
pub fn hmac_sha512_hex(key: &[u8], data: &[u8]) -> String {
    hex::encode(hmac_sha512(key, data))
}

/// Constant-time verification of an HMAC-SHA256 tag.
pub fn verify_sha256(key: &[u8], expected: &[u8], data: &[u8]) -> bool {
    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(data);
    mac.verify_slice(expected).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 4231 Test Case 2: key="Jefe", data="what do ya want for nothing?"
    const RFC4231_KEY: &[u8] = b"Jefe";
    const RFC4231_DATA: &[u8] = b"what do ya want for nothing?";
    const RFC4231_SHA256: &str = "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843";

    #[test]
    fn sha256_rfc_vector() {
        assert_eq!(hmac_sha256_hex(RFC4231_KEY, RFC4231_DATA), RFC4231_SHA256);
    }

    #[test]
    fn sha256_hex_matches_bytes() {
        let bytes = hmac_sha256(b"k", b"d");
        assert_eq!(hex::encode(&bytes), hmac_sha256_hex(b"k", b"d"));
    }

    #[test]
    fn sha256_length() {
        assert_eq!(hmac_sha256(b"k", b"d").len(), 32);
    }

    #[test]
    fn sha512_length() {
        assert_eq!(hmac_sha512(b"k", b"d").len(), 64);
    }

    #[test]
    fn deterministic() {
        assert_eq!(
            hmac_sha256_hex(b"key", b"msg"),
            hmac_sha256_hex(b"key", b"msg")
        );
        assert_ne!(
            hmac_sha256_hex(b"key", b"msg"),
            hmac_sha256_hex(b"key", b"tampered")
        );
    }

    #[test]
    fn verify_roundtrip() {
        let tag = hmac_sha256(b"secret", b"payload");
        assert!(verify_sha256(b"secret", &tag, b"payload"));
        assert!(!verify_sha256(b"wrong", &tag, b"payload"));
        assert!(!verify_sha256(b"secret", &tag, b"tampered"));
    }

    #[test]
    fn empty_inputs_ok() {
        assert_eq!(hmac_sha256(b"", b"").len(), 32);
    }
}
