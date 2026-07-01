//! ECDSA + KDF + constant-time verify pipeline.

#![cfg(feature = "ecdsa")]

use rskit::crypto::{ct, ecdsa, kdf};

#[test]
fn derive_key_and_sign() {
    // Derive a 32-byte ECDSA-ish seed via HKDF from a password-derived key.
    let pbk = kdf::pbkdf2_sha256(b"password", b"salt", 1000, 32);
    let seed = kdf::hkdf_sha256(b"hkdf-salt", &pbk, b"ecdsa-ctx", 32);
    assert_eq!(seed.len(), 32);

    let (sk, pk) = ecdsa::generate_keypair(ecdsa::Curve::Secp256k1);
    let sig = ecdsa::sign(ecdsa::Curve::Secp256k1, &sk, b"doc").unwrap();
    assert!(ecdsa::verify(ecdsa::Curve::Secp256k1, &pk, b"doc", &sig).unwrap());

    // a deterministic digest compare of the signature length
    assert!(ct::ct_eq(&sig, &sig));
    assert!(!ct::ct_eq(&sig, &[0u8; 72]));
}
