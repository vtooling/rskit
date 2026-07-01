//! # rskit
//!
//! A collection of practical Rust utilities: encoding, hashing, crypto (incl.
//! JWT, TOTP, ECDSA, password hashing), serde helpers, cache (incl. LRU),
//! scheduling, IDs, rate limiting, circuit breaking, stats, validation,
//! time/date, filesystem, logging, config loading, and optional HTTP / DB /
//! Redis / compression / host integrations.
//!
//! ## Feature flags
//!
//! - `full` (default) — enables every optional integration below.
//! - `http` — `reqwest`-based HTTP helpers.
//! - `db` — `sqlx` SQLite/Postgres pool helpers.
//! - `redis` — async Redis helpers.
//! - `sys` — process introspection via `sysinfo`.
//! - `id` — UUID v4/v7, NanoID, ULID, Snowflake.
//! - `pass` — Argon2/bcrypt password hashing.
//! - `jwt` — HS256 JSON Web Tokens.
//! - `totp` — HOTP/TOTP one-time passwords.
//! - `ecdsa` — ECDSA sign/verify (P-256 / secp256k1).
//! - `compress` — gzip/zstd compression.
//! - `decimal` — decimal/money via `rust_decimal`.
//! - `yaml` / `csv` — extra serde formats.
//! - `host` — hostname and local IP addresses.

pub mod cache;
pub mod crypto;
pub mod datetime;
pub mod encode;
pub mod envutil;
pub mod fs;
pub mod hash;
pub mod log;
pub mod lru;
pub mod num;
pub mod rate_limit;
pub mod retry;
pub mod serde_ext;
pub mod stats;
pub mod str_util;
pub mod throttle;
pub mod timer;
pub mod valid;

pub mod circuit;
pub mod config;
pub mod sys;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "db")]
pub mod db;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "id")]
pub mod id;

#[cfg(feature = "compress")]
pub mod compress;

#[cfg(feature = "decimal")]
pub mod decimal;

#[cfg(feature = "host")]
pub mod host;

#[cfg(feature = "sys")]
pub use sys::is_running_current;

pub use log::Log;

// Convenience re-exports.
pub use config::{App, Configs, Settings};
