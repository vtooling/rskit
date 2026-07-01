//! Ed25519 sign/verify helpers.

use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rsa::signature::SignerMut;

/// Generate an Ed25519 signing/verifying key pair.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut rng = OsRng;
    let sk = SigningKey::generate(&mut rng);
    let vk = sk.verifying_key();
    (sk, vk)
}

/// Sign `msg` with `sk`.
pub fn sign(sk: &mut SigningKey, msg: &[u8]) -> Signature {
    sk.sign(msg)
}

/// Verify a signature. Returns `true` on a valid match.
pub fn verify(vk: &VerifyingKey, sig: &Signature, msg: &[u8]) -> bool {
    vk.verify_strict(msg, sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify_roundtrip() {
        let (mut sk, vk) = generate_keypair();
        let msg = b"hello ed25519";
        let sig = sign(&mut sk, msg);
        assert!(verify(&vk, &sig, msg));
    }

    #[test]
    fn verify_rejects_tampered_message() {
        let (mut sk, vk) = generate_keypair();
        let sig = sign(&mut sk, b"original");
        assert!(!verify(&vk, &sig, b"tampered"));
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let (mut sk1, _) = generate_keypair();
        let (_, vk2) = generate_keypair();
        let sig = sign(&mut sk1, b"msg");
        assert!(!verify(&vk2, &sig, b"msg"));
    }

    #[test]
    fn keypairs_differ() {
        let (sk1, _) = generate_keypair();
        let (sk2, _) = generate_keypair();
        assert_ne!(sk1.to_bytes(), sk2.to_bytes());
    }
}
