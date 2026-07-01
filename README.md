# rskit

A collection of practical Rust utilities: encoding, hashing, crypto (incl. JWT,
TOTP, password hashing), serde helpers, an in-memory cache, scheduling, IDs,
retry, validation, time/date, filesystem, logging, config loading, and optional
HTTP / DB / Redis / compression integrations.

## Features

| Module          | Description                                          | Feature    |
| --------------- | ---------------------------------------------------- | ---------- |
| `encode`        | base58 / base64 / hex / url encode & decode          | always     |
| `hash`          | sha256 / sha512 / md5                                | always     |
| `str_util`      | case conversion, slugify, random strings             | always     |
| `datetime`      | duration parse/humanize, timestamps, "time ago"      | always     |
| `valid`         | email / url / ip / credit-card validators            | always     |
| `fs`            | read/write/append/atomic_write/ensure_dir            | always     |
| `retry`         | retry with exponential backoff (sync + async)        | always     |
| `serde_ext`     | json/bin helpers, datetime serde, query flattener    | always     |
| `cache`         | thread-safe in-memory key/value cache                | always     |
| `num`           | big-integer / byte conversions                       | always     |
| `timer`         | cron & interval scheduling on tokio                  | always     |
| `config`        | TOML config loader + auto file-watch reload          | always     |
| `log`           | `fast_log` wrapper                                   | always     |
| `crypto`        | AES (CBC/GCM), RSA, Ed25519, ECDH                    | always     |
| `crypto::hmac`  | HMAC-SHA256 / SHA512                                 | always     |
| `sys`           | process info, raw pointer / Windows helpers          | partial    |
| `id`            | UUID v4/v7, NanoID, ULID, Snowflake                  | `id`       |
| `crypto::pass`  | Argon2 + bcrypt password hashing                     | `pass`     |
| `crypto::jwt`   | HS256 JSON Web Tokens                                | `jwt`      |
| `crypto::totp`  | HOTP / TOTP one-time passwords                       | `totp`     |
| `compress`      | gzip / zstd compression                              | `compress` |
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

assert_eq!(datetime::parse_duration("1h30m").unwrap(), Duration::from_secs(5400));

assert!(valid::is_email("a@b.com"));
assert!(!valid::is_email("bad"));

let n = Retry::new().max_attempts(5).run_sync(|| Ok::<_, anyhow::Error>(())).unwrap();
```

## License

MIT
