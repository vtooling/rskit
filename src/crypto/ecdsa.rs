//! ECDSA signing / verification over P-256 and secp256k1.
//!
//! Requires the `ecdsa` feature (enables `k256`/`p256` ECDSA + SHA-256).

use anyhow::Result;
use k256::ecdsa::signature::{Signer, Verifier};

/// Supported ECDSA curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    /// NIST P-256 / secp256r1.
    P256,
    /// SECG secp256k1.
    Secp256k1,
}

/// Generate a key pair, returning `(secret_hex, public_hex)` where the public
/// key is SEC1-encoded and the secret key is the raw scalar.
pub fn generate_keypair(curve: Curve) -> (String, String) {
    match curve {
        Curve::P256 => {
            let sk = p256::ecdsa::SigningKey::random(&mut rand::thread_rng());
            let pk = sk.verifying_key();
            (hex::encode(sk.to_bytes()), hex::encode(pk.to_sec1_bytes()))
        }
        Curve::Secp256k1 => {
            let sk = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
            let pk = sk.verifying_key();
            (hex::encode(sk.to_bytes()), hex::encode(pk.to_sec1_bytes()))
        }
    }
}

/// Sign `msg` (hashed internally with SHA-256). Returns DER-encoded signature bytes.
pub fn sign(curve: Curve, sk_hex: &str, msg: &[u8]) -> Result<Vec<u8>> {
    let sk_bytes = hex::decode(sk_hex)?;
    Ok(match curve {
        Curve::P256 => {
            let sk = p256::ecdsa::SigningKey::from_slice(&sk_bytes)?;
            let sig: p256::ecdsa::Signature = sk.sign(msg);
            sig.to_der().to_bytes().to_vec()
        }
        Curve::Secp256k1 => {
            let sk = k256::ecdsa::SigningKey::from_slice(&sk_bytes)?;
            let sig: k256::ecdsa::Signature = sk.sign(msg);
            sig.to_der().to_bytes().to_vec()
        }
    })
}

/// Verify a DER-encoded signature. Returns `Ok(false)` on a bad signature or
/// key rather than erroring, except for malformed inputs.
pub fn verify(curve: Curve, pk_hex: &str, msg: &[u8], sig: &[u8]) -> Result<bool> {
    let pk_bytes = hex::decode(pk_hex)?;
    Ok(match curve {
        Curve::P256 => {
            let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(&pk_bytes)?;
            let sig = p256::ecdsa::Signature::from_der(sig)?;
            vk.verify(msg, &sig).is_ok()
        }
        Curve::Secp256k1 => {
            let vk = k256::ecdsa::VerifyingKey::from_sec1_bytes(&pk_bytes)?;
            let sig = k256::ecdsa::Signature::from_der(sig)?;
            vk.verify(msg, &sig).is_ok()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(curve: Curve) {
        let (sk, pk) = generate_keypair(curve);
        let msg = b"sign me";
        let sig = sign(curve, &sk, msg).unwrap();
        assert!(verify(curve, &pk, msg, &sig).unwrap());
        // tampered message fails
        assert!(!verify(curve, &pk, b"other", &sig).unwrap());
        // different keypair
        let (_, pk2) = generate_keypair(curve);
        assert!(!verify(curve, &pk2, msg, &sig).unwrap());
    }

    #[test]
    fn p256_roundtrip() {
        roundtrip(Curve::P256);
    }

    #[test]
    fn secp256k1_roundtrip() {
        roundtrip(Curve::Secp256k1);
    }

    #[test]
    fn invalid_secret_hex_errors() {
        assert!(sign(Curve::P256, "nothex", b"x").is_err());
    }

    #[test]
    fn cross_curve_keys_differ() {
        let (sk1, _) = generate_keypair(Curve::P256);
        let (sk2, _) = generate_keypair(Curve::Secp256k1);
        assert_ne!(sk1, sk2);
    }
}
