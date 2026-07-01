//! Unified ECDH key agreement over secp256k1 (k256) and secp256r1 (p256).
//!
//! Keys are exchanged as hex strings: the secret key is the raw scalar bytes,
//! the public key is the SEC1-encoded point.

use anyhow::{Result, anyhow};

/// Supported ECDH curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    /// NIST P-256 / secp256r1.
    P256,
    /// SECG secp256k1.
    Secp256k1,
}

/// Generate a key pair, returning `(secret_hex, public_hex)`.
pub fn generate_keypair(curve: Curve) -> (String, String) {
    match curve {
        Curve::P256 => {
            let sk = p256::SecretKey::random(&mut rand::thread_rng());
            let pk = sk.public_key();
            (hex::encode(sk.to_bytes()), hex::encode(pk.to_sec1_bytes()))
        }
        Curve::Secp256k1 => {
            let sk = k256::SecretKey::random(&mut rand::thread_rng());
            let pk = sk.public_key();
            (hex::encode(sk.to_bytes()), hex::encode(pk.to_sec1_bytes()))
        }
    }
}

/// Derive the shared secret hex string from a local secret key and a peer's
/// public key.
pub fn generate_shared(curve: Curve, sk_hex: &str, pk_hex: &str) -> Result<String> {
    let sk_bytes = hex::decode(sk_hex).map_err(|e| anyhow!("decode sk: {e}"))?;
    let pk_bytes = hex::decode(pk_hex).map_err(|e| anyhow!("decode pk: {e}"))?;
    match curve {
        Curve::P256 => {
            let sk = p256::SecretKey::from_slice(&sk_bytes)
                .map_err(|e| anyhow!("invalid p256 secret key: {e}"))?;
            let pk = p256::PublicKey::from_sec1_bytes(&pk_bytes)
                .map_err(|e| anyhow!("invalid p256 public key: {e}"))?;
            let shared = p256::ecdh::diffie_hellman(sk.to_nonzero_scalar(), pk.as_affine());
            Ok(hex::encode(shared.raw_secret_bytes()))
        }
        Curve::Secp256k1 => {
            let sk = k256::SecretKey::from_slice(&sk_bytes)
                .map_err(|e| anyhow!("invalid k256 secret key: {e}"))?;
            let pk = k256::PublicKey::from_sec1_bytes(&pk_bytes)
                .map_err(|e| anyhow!("invalid k256 public key: {e}"))?;
            let shared = k256::ecdh::diffie_hellman(sk.to_nonzero_scalar(), pk.as_affine());
            Ok(hex::encode(shared.raw_secret_bytes()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_curve(curve: Curve) {
        let (sk1, pk1) = generate_keypair(curve);
        let (sk2, pk2) = generate_keypair(curve);

        // Valid hex.
        assert!(sk1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(pk1.chars().all(|c| c.is_ascii_hexdigit()));

        // ECDH symmetry: both parties derive the same shared secret.
        let shared1 = generate_shared(curve, &sk1, &pk2).unwrap();
        let shared2 = generate_shared(curve, &sk2, &pk1).unwrap();
        assert_eq!(shared1, shared2);
        assert!(!shared1.is_empty());
    }

    #[test]
    fn p256_pair_and_shared() {
        assert_curve(Curve::P256);
    }

    #[test]
    fn secp256k1_pair_and_shared() {
        assert_curve(Curve::Secp256k1);
    }

    #[test]
    fn invalid_secret_hex_errors() {
        assert!(generate_shared(Curve::P256, "nothex", "00").is_err());
    }

    #[test]
    fn invalid_key_bytes_errors() {
        // Valid hex but not a valid scalar/point.
        assert!(generate_shared(Curve::P256, "00", "00").is_err());
        assert!(generate_shared(Curve::Secp256k1, "00", "00").is_err());
    }

    #[test]
    fn cross_curve_keys_differ() {
        let (sk_p, _) = generate_keypair(Curve::P256);
        let (sk_k, _) = generate_keypair(Curve::Secp256k1);
        assert_ne!(sk_p, sk_k);
    }
}
