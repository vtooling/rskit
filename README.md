# rskit

A collection of practical Rust utilities: encoding, hashing, crypto (incl. JWT,
TOTP, ECDSA, password hashing), serde helpers, cache (incl. LRU), scheduling,
IDs, rate limiting, circuit breaking, stats, validation, time/date, filesystem,
logging, config loading, and optional HTTP / DB / Redis / compression / host
integrations.

## Features

| Module          | Description                                          | Feature    |
| --------------- | ---------------------------------------------------- | ---------- |
| `encode`        | base58 / base64 / hex / url encode & decode          | always     |
| `hash`          | sha256 / sha512 / md5                                | always     |
| `str_util`      | case conversion, slugify, masking, levenshtein, template, random strings | always |
| `datetime`      | duration parse/humanize, timestamps, "time ago"      | always     |
| `valid`         | email / url / ip / credit-card / hex validators      | always     |
| `fs`            | read/write/append/atomic_write/ensure_dir            | always     |
| `retry`         | retry with exponential backoff (sync + async)        | always     |
| `rate_limit`    | token bucket & leaky bucket rate limiters            | always     |
| `circuit`       | circuit breaker (Closed/Open/HalfOpen)               | always     |
| `throttle`      | throttle gate & debounce                             | always     |
| `stats`         | min/max/mean/median/percentile/stddev                | always     |
| `lru`           | thread-safe LRU cache                                | always     |
| `envutil`       | typed environment variables                          | always     |
| `serde_ext`     | json/bin helpers, datetime serde, query flattener    | always     |
| `cache`         | thread-safe in-memory key/value cache                | always     |
| `num`           | big-integer / byte conversions                       | always     |
| `timer`         | cron & interval scheduling on tokio                  | always     |
| `config`        | TOML config loader + auto file-watch reload          | always     |
| `log`           | `fast_log` wrapper                                   | always     |
| `crypto`        | AES (CBC/GCM), RSA, Ed25519, ECDH                    | always     |
| `crypto::hmac`  | HMAC-SHA256 / SHA512                                 | always     |
| `crypto::kdf`   | HKDF-SHA256, PBKDF2-SHA256                           | always     |
| `crypto::ct`    | constant-time comparison                             | always     |
| `sys`           | process info, raw pointer / Windows helpers          | partial    |
| `id`            | UUID v4/v7, NanoID, ULID, Snowflake                  | `id`       |
| `crypto::pass`  | Argon2 + bcrypt password hashing                     | `pass`     |
| `crypto::jwt`   | HS256 JSON Web Tokens                                | `jwt`      |
| `crypto::totp`  | HOTP / TOTP one-time passwords                       | `totp`     |
| `crypto::ecdsa` | ECDSA sign/verify (P-256 / secp256k1)                | `ecdsa`    |
| `compress`      | gzip / zstd compression                              | `compress` |
| `decimal`       | decimal / money via `rust_decimal`                   | `decimal`  |
| `host`          | hostname and local IP addresses                      | `host`     |
| `serde_ext` +yaml| `to_yaml` / `from_yaml`                             | `yaml`     |
| `serde_ext` +csv | `to_csv` / `from_csv`                               | `csv`      |
| `http`          | `reqwest`-based HTTP client                          | `http`     |
| `db`            | `sqlx` SQLite/Postgres pool builders                 | `db`       |
| `redis`         | async Redis helpers                                  | `redis`    |

All optional integrations are enabled by the **`full`** feature, which is on by
default. To slim down the dependency tree, disable default features and opt in:

```toml
[dependencies]
# Core only (encode/hash/crypto/serde/cache/str_util/datetime/...), no DB/HTTP/Redis:
rskit = { version = "0.1", default-features = false }
# Or pick specific integrations:
# rskit = { version = "0.1", default-features = false, features = ["db", "redis", "id", "jwt"] }
```

## Quick examples

### Encoding & hashing

```rust
use rskit::{encode, hash};

let raw = b"hello world";
let b64 = encode::base64_encode(raw);          // "aGVsbG8gd29ybGQ="
assert_eq!(encode::base64_decode(&b64).unwrap(), raw);

assert_eq!(
    hash::sha256(raw),
    "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
);
```

### Cache

```rust
use rskit::cache::Cache;

let cache = Cache::new();
cache.set("name", "iclings");
assert_eq!(cache.get::<&str>("name"), Some("iclings"));
```

### Serde helpers

```rust
use rskit::serde_ext;

#[derive(serde::Serialize, serde::Deserialize)]
struct Aoo { name: String, age: i32 }

let a = Aoo { name: "ok".into(), age: 18 };
let json = serde_ext::to_json(&a).unwrap();
let back: Aoo = serde_ext::from_json(&json).unwrap();
```

### AES

```rust
use rskit::crypto::aes;

let key = aes::gen_rand_string(Some(32));      // 32-byte key
let ct = aes::encrypt_aes_cbc_256(key.as_bytes(), b"hello").unwrap();
let pt = aes::decrypt_aes_cbc_256(key.as_bytes(), &ct).unwrap();
assert_eq!(pt, b"hello");
```

### RSA

```rust
use rskit::crypto::rsa;

let (pri, pubk) = rsa::generate_rsa_pair(Some(2048)).unwrap();
let ct = rsa::encrypt_rsa_base(&pubk, b"hello").unwrap();
let pt = rsa::decrypt_rsa_base(&pri, &ct).unwrap();
assert_eq!(pt, b"hello");
```

### ECDH (shared secret)

```rust
use rskit::crypto::ecdh;

let (sk_a, pk_a) = ecdh::generate_keypair(ecdh::Curve::Secp256k1);
let (sk_b, pk_b) = ecdh::generate_keypair(ecdh::Curve::Secp256k1);

let s1 = ecdh::generate_shared(ecdh::Curve::Secp256k1, &sk_a, &pk_b).unwrap();
let s2 = ecdh::generate_shared(ecdh::Curve::Secp256k1, &sk_b, &pk_a).unwrap();
assert_eq!(s1, s2);
```

### Config + log

```rust
use rskit::{config, Log};

Log::new().init().unwrap();            // console logger
rskit::log::info!("booting rskit");

let s: config::Settings = config::load_str(r#"[app]
version = "1.0.0""#).unwrap();
assert_eq!(s.app.version, "1.0.0");

// Auto-reload: spawn a watcher that re-reads ./app.toml on change.
config::init_auto_watch();
```

### DB pool (feature `db`)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = rskit::db::sqlite_pool(":memory:").await?;
    sqlx::query("create table t (id integer primary key)")
        .execute(&pool).await?;
    Ok(())
}
```

See [`examples/db_demo.rs`](examples/db_demo.rs) for a full CRUD example.

### IDs (feature `id`)

```rust
use rskit::id;

println!("{}", id::uuid_v4());
println!("{}", id::uuid_v7());        // time-ordered
println!("{}", id::nanoid_default()); // 21-char
println!("{}", id::ulid_str());       // 26-char sortable
let sf = id::Snowflake::new(1);
println!("{}", sf.next_id());         // 64-bit monotonic
```

### Password hashing (feature `pass`)

```rust
use rskit::crypto::pass;

let phc = pass::argon2_hash("hunter2").unwrap();
assert!(pass::verify_password("hunter2", &phc).unwrap());

let phc2 = pass::bcrypt_hash("hunter2", 10).unwrap();
// verify_password auto-detects argon2 vs bcrypt by prefix
assert!(pass::verify_password("hunter2", &phc2).unwrap());
```

### JWT (feature `jwt`)

```rust
use rskit::crypto::jwt;

let claims = jwt::claims_for("user-42", 3600); // 1h TTL
let token = jwt::encode_jwt(&claims, "secret").unwrap();
let back: jwt::Claims = jwt::decode_jwt(&token, "secret").unwrap();
assert_eq!(back.sub, "user-42");
```

### HMAC & TOTP

```rust
use rskit::crypto::{hmac, totp};

let sig = hmac::hmac_sha256_hex(b"key", b"payload");

// RFC 6238 TOTP (30s step, 6 digits)
let code = totp::totp_now(b"my-secret", 30, 6);
assert!(totp::totp_verify(b"my-secret", rskit::datetime::now_ts() as u64, 30, 6, &code, 1));
```

### String, time, validation, retry

```rust
use rskit::{str_util, datetime, valid, retry::Retry};
use std::time::Duration;

assert_eq!(str_util::to_snake_case("HTTPServer"), "http_server");
assert_eq!(str_util::slugify("Hello, World!"), "hello-world");
assert_eq!(str_util::mask_email("alice@example.com"), "a****@example.com");
assert_eq!(str_util::levenshtein("kitten", "sitting"), 3);

assert_eq!(datetime::parse_duration("1h30m").unwrap(), Duration::from_secs(5400));

assert!(valid::is_email("a@b.com"));
assert!(!valid::is_email("bad"));

let n = Retry::new().max_attempts(5).run_sync(|| Ok::<_, anyhow::Error>(())).unwrap();
```

### Rate limiting, circuit breaker, LRU

```rust
use std::time::Duration;
use rskit::{rate_limit::TokenBucket, circuit::CircuitBreaker, lru::LruCache};

let tb = TokenBucket::new(10, 5.0);      // capacity 10, refill 5/s
assert!(tb.try_acquire(1));

let cb = CircuitBreaker::new(5, Duration::from_secs(30)); // open after 5 failures
let _: anyhow::Result<()> = cb.call_sync(|| Ok(()));

let lru: LruCache<&str, i32> = LruCache::new(100);
lru.put("k", 1);
assert_eq!(lru.get(&"k"), Some(1));
```

### Stats, KDF, ECDSA, constant-time compare

```rust
use rskit::{stats, crypto::{kdf, ecdsa, ct}};

assert_eq!(stats::mean(&[1.0, 2.0, 3.0]), Some(2.0));
assert_eq!(stats::percentile(&[1.0, 2.0, 3.0, 4.0], 50.0), Some(2.5));

let okm = kdf::hkdf_sha256(b"salt", b"ikm", b"info", 32);
let pw_key = kdf::pbkdf2_sha256(b"password", b"nacl", 100_000, 32);

let (sk, pk) = ecdsa::generate_keypair(ecdsa::Curve::P256);
let sig = ecdsa::sign(ecdsa::Curve::P256, &sk, b"doc").unwrap();
assert!(ecdsa::verify(ecdsa::Curve::P256, &pk, b"doc", &sig).unwrap());
assert!(ct::ct_eq(&sig, &sig));
```

### Decimal, YAML/CSV, host (features `decimal` / `yaml` / `csv` / `host`)

```rust
use rskit::{decimal, serde_ext, host};

let price = decimal::parse("19.99").unwrap();
assert_eq!((price + decimal::parse("0.01").unwrap()).to_string(), "20.00");

let yaml = serde_ext::to_yaml(&[1, 2, 3]).unwrap();    // feature "yaml"
let csv  = serde_ext::to_csv(&[("a", 1)]).unwrap();    // feature "csv"

println!("host: {}", host::hostname());                // feature "host"
println!("ip: {:?}", host::local_ip().unwrap());
```

## License

MIT
