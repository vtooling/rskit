//! Crypto integration: hybrid encrypt-then-sign flow across modules.

use rskit::crypto::{aes, ecc, ecdh};
use rskit::{encode, hash};

#[test]
fn ecdh_derives_shared_aes_key() {
    // Two parties agree on a shared secret via ECDH, then use it as an AES key.
    let (sk_a, pk_a) = ecdh::generate_keypair(ecdh::Curve::Secp256k1);
    let (sk_b, pk_b) = ecdh::generate_keypair(ecdh::Curve::Secp256k1);

    let shared_a = ecdh::generate_shared(ecdh::Curve::Secp256k1, &sk_a, &pk_b).unwrap();
    let shared_b = ecdh::generate_shared(ecdh::Curve::Secp256k1, &sk_b, &pk_a).unwrap();
    assert_eq!(shared_a, shared_b);

    // Derive a 32-byte AES-256 key by hashing the shared secret.
    let key = hash::sha256_bytes(shared_a.as_bytes());
    assert_eq!(key.len(), 32);

    let nonce = aes::aes_gcm_nonce_256();
    let msg = b"top secret payload";
    let ciphertext = aes::encrypt_aes_gcm_256(&key, &nonce, msg).unwrap();
    assert_eq!(
        aes::decrypt_aes_gcm_256(&key, &nonce, &ciphertext).unwrap(),
        msg
    );
}

#[test]
fn sign_then_encrypt_envelope() {
    // Sign a digest with Ed25519, then encrypt the message + signature with AES.
    let (mut sk, vk) = ecc::generate_keypair();
    let msg = b"hello envelope";

    let sig = ecc::sign(&mut sk, msg);
    assert!(ecc::verify(&vk, &sig, msg));

    let key = aes::aes_gcm_key_128();
    let nonce = aes::aes_gcm_nonce_128();
    let payload = encode::base64_encode(msg); // pretend payload
    let ct = aes::encrypt_aes_gcm_128(&key, &nonce, payload.as_bytes()).unwrap();
    let pt = aes::decrypt_aes_gcm_128(&key, &nonce, &ct).unwrap();
    assert_eq!(pt, payload.as_bytes());
}
