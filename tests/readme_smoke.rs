//! Verifies the code snippets shown in README.md actually compile against the
//! public API. Gated on `full` so every referenced module is present.

#![cfg(all(
    feature = "id",
    feature = "pass",
    feature = "jwt",
    feature = "totp",
    feature = "ecdsa",
    feature = "decimal",
    feature = "yaml",
    feature = "csv",
    feature = "host"
))]

#[allow(dead_code, unused_imports, unused_variables, unused_mut)]
#[test]
fn readme_snippets_compile() {
    use std::time::Duration;

    // Encoding & hashing
    let raw = b"hello world";
    let b64 = rskit::encode::base64_encode(raw);
    let _ = rskit::encode::base64_decode(&b64).unwrap();
    let _ = rskit::hash::sha256(raw);

    // Cache
    let cache = rskit::cache::Cache::new();
    cache.set("name", "iclings");
    let _ = cache.get::<&str>("name");

    // Serde
    let json = rskit::serde_ext::to_json(&vec![1, 2]).unwrap();
    let _back: Vec<i32> = rskit::serde_ext::from_json(&json).unwrap();

    // AES
    let key = rskit::crypto::aes::gen_rand_string(Some(32));
    let ct = rskit::crypto::aes::encrypt_aes_cbc_256(key.as_bytes(), b"hello").unwrap();
    let _ = rskit::crypto::aes::decrypt_aes_cbc_256(key.as_bytes(), &ct).unwrap();

    // RSA
    let (pri, pubk) = rskit::crypto::rsa::generate_rsa_pair(Some(2048)).unwrap();
    let ct = rskit::crypto::rsa::encrypt_rsa_base(&pubk, b"hello").unwrap();
    let _ = rskit::crypto::rsa::decrypt_rsa_base(&pri, &ct).unwrap();

    // ECDH
    let (sk_a, pk_a) = rskit::crypto::ecdh::generate_keypair(rskit::crypto::ecdh::Curve::Secp256k1);
    let (sk_b, pk_b) = rskit::crypto::ecdh::generate_keypair(rskit::crypto::ecdh::Curve::Secp256k1);
    let s1 =
        rskit::crypto::ecdh::generate_shared(rskit::crypto::ecdh::Curve::Secp256k1, &sk_a, &pk_b)
            .unwrap();
    let s2 =
        rskit::crypto::ecdh::generate_shared(rskit::crypto::ecdh::Curve::Secp256k1, &sk_b, &pk_a)
            .unwrap();
    assert_eq!(s1, s2);

    // String/time/valid/retry
    assert_eq!(rskit::str_util::to_snake_case("HTTPServer"), "http_server");
    assert_eq!(rskit::str_util::slugify("Hello, World!"), "hello-world");
    assert_eq!(
        rskit::str_util::mask_email("alice@example.com"),
        "a****@example.com"
    );
    assert_eq!(rskit::str_util::levenshtein("kitten", "sitting"), 3);
    assert_eq!(
        rskit::datetime::parse_duration("1h30m").unwrap(),
        Duration::from_secs(5400)
    );
    assert!(rskit::valid::is_email("a@b.com"));
    let _: anyhow::Result<()> = rskit::retry::Retry::new()
        .max_attempts(5)
        .run_sync(|| Ok(()));

    // Rate limit / circuit / lru
    let tb = rskit::rate_limit::TokenBucket::new(10, 5.0);
    assert!(tb.try_acquire(1));
    let cb = rskit::circuit::CircuitBreaker::new(5, Duration::from_secs(30));
    let _: anyhow::Result<()> = cb.call_sync(|| Ok(()));
    let lru: rskit::lru::LruCache<&str, i32> = rskit::lru::LruCache::new(100);
    lru.put("k", 1);
    assert_eq!(lru.get(&"k"), Some(1));

    // Stats / kdf / ecdsa / ct
    assert_eq!(rskit::stats::mean(&[1.0, 2.0, 3.0]), Some(2.0));
    assert_eq!(
        rskit::stats::percentile(&[1.0, 2.0, 3.0, 4.0], 50.0),
        Some(2.5)
    );
    let _okm = rskit::crypto::kdf::hkdf_sha256(b"salt", b"ikm", b"info", 32);
    let _pwk = rskit::crypto::kdf::pbkdf2_sha256(b"password", b"nacl", 100_000, 32);
    let (sk, pk) = rskit::crypto::ecdsa::generate_keypair(rskit::crypto::ecdsa::Curve::P256);
    let sig = rskit::crypto::ecdsa::sign(rskit::crypto::ecdsa::Curve::P256, &sk, b"doc").unwrap();
    assert!(
        rskit::crypto::ecdsa::verify(rskit::crypto::ecdsa::Curve::P256, &pk, b"doc", &sig).unwrap()
    );
    assert!(rskit::crypto::ct::ct_eq(&sig, &sig));

    // Decimal / yaml / csv / host
    let price = rskit::decimal::parse("19.99").unwrap();
    assert_eq!(
        (price + rskit::decimal::parse("0.01").unwrap()).to_string(),
        "20.00"
    );
    let _yaml = rskit::serde_ext::to_yaml(&[1, 2, 3]).unwrap();
    let _csv = rskit::serde_ext::to_csv(&[("a", 1)]).unwrap();
    let _host = rskit::host::hostname();
}
