use sha2::{Digest, Sha256, Sha512};

/// Compute SHA-256 and return the digest as a lowercase hex string.
pub fn sha256(input: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(input);
    format!("{:x}", h.finalize())
}

/// Compute SHA-256 and return the raw digest bytes.
pub fn sha256_bytes(input: &[u8]) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(input);
    h.finalize().to_vec()
}

/// Compute SHA-512 and return the digest as a lowercase hex string.
pub fn sha512(input: &[u8]) -> String {
    let mut h = Sha512::new();
    h.update(input);
    format!("{:x}", h.finalize())
}

/// Compute SHA-512 and return the raw digest bytes.
pub fn sha512_bytes(input: &[u8]) -> Vec<u8> {
    let mut h = Sha512::new();
    h.update(input);
    h.finalize().to_vec()
}

/// Compute MD5 and return the digest as a lowercase hex string.
///
/// MD5 is cryptographically broken and must not be used for security;
/// it is provided for legacy interoperability and checksums only.
pub fn md5(input: &[u8]) -> String {
    format!("{:x}", md5::compute(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            sha256(b"hello world"),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn sha256_bytes_matches_hex() {
        let bytes = sha256_bytes(b"abc");
        assert_eq!(hex::encode(bytes), sha256(b"abc"));
    }

    #[test]
    fn sha256_empty() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha512_known_vector() {
        assert_eq!(
            sha512(b"hello world"),
            "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
        );
    }

    #[test]
    fn sha512_bytes_len() {
        assert_eq!(sha512_bytes(b"x").len(), 64);
    }

    #[test]
    fn md5_known_vector() {
        assert_eq!(md5(b"hello world"), "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn md5_empty() {
        assert_eq!(md5(b""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn deterministic_output() {
        assert_eq!(sha256(b"rskit"), sha256(b"rskit"));
        assert_ne!(sha256(b"a"), sha256(b"b"));
    }
}
