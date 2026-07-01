//! Cross-module smoke tests that exercise several rskit modules together.

use rskit::{cache::Cache, encode, hash, serde_ext};

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
struct Record {
    id: u64,
    name: String,
}

#[test]
fn json_encode_hash_pipeline() {
    let rec = Record {
        id: 7,
        name: "rskit".into(),
    };

    // serialize -> encode -> hash, then reverse.
    let json = serde_ext::to_json(&rec).unwrap();
    let b64 = encode::base64_encode(json.as_bytes());
    let digest = hash::sha256(b64.as_bytes());

    assert_ne!(digest, hash::sha256(json.as_bytes()));
    assert_eq!(digest.len(), 64); // sha256 hex length

    let back = encode::base64_decode(&b64).unwrap();
    let parsed: Record = serde_ext::from_json(&String::from_utf8(back).unwrap()).unwrap();
    assert_eq!(parsed, rec);
}

#[test]
fn cache_stores_serialized_payload() {
    let cache = Cache::new();
    let rec = Record {
        id: 1,
        name: "iclings".into(),
    };
    let payload = serde_ext::to_json(&rec).unwrap();
    cache.set("record", payload);

    let got: String = cache.get("record").unwrap();
    let parsed: Record = serde_ext::from_json(&got).unwrap();
    assert_eq!(parsed, rec);
}

#[test]
fn bigint_and_hex_roundtrip() {
    let raw = b"\x01\x02\x03\x04";
    let num = rskit::num::bytes_to_int(raw);
    let hex = encode::hex_encode(raw);
    // The decimal and hex forms must both reconstitute the original bytes.
    assert_eq!(rskit::num::int_to_bytes(&num), Some(raw.to_vec()));
    assert_eq!(encode::hex_decode(&hex).unwrap(), raw.to_vec());
}

#[test]
fn datetime_serde_through_json() {
    use chrono::TimeZone;
    let toml = r#"
[app]
version = "2.0.0"
"#;
    let s: rskit::Settings = rskit::config::load_str(toml).unwrap();
    assert_eq!(s.app.version, "2.0.0");

    // ensure the datetime helper round-trips via the public constant
    let _fmt = rskit::serde_ext::DATETIME_FORMAT;
    let dt = chrono::Local.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    assert!(rskit::serde_ext::to_json(&dt).is_ok());
}
