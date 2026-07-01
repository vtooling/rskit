//! Typed environment-variable helpers (uses `std::env`).

use std::fmt::Display;
use std::str::FromStr;

use anyhow::{Result, anyhow};

/// Read and parse an env var into `T`. Returns `None` if the var is unset.
pub fn get<T: FromStr>(key: &str) -> Option<T>
where
    <T as FromStr>::Err: Display,
{
    std::env::var(key).ok().and_then(|v| v.parse::<T>().ok())
}

/// Read and parse an env var, falling back to `default` if unset or invalid.
pub fn get_or<T: FromStr>(key: &str, default: T) -> T
where
    <T as FromStr>::Err: Display,
{
    get(key).unwrap_or(default)
}

/// Read and parse an env var, returning an error if unset or unparseable.
pub fn require<T: FromStr>(key: &str) -> Result<T>
where
    <T as FromStr>::Err: Display,
{
    let raw = std::env::var(key).map_err(|e| anyhow!("{key}: {e}"))?;
    raw.parse::<T>()
        .map_err(|e| anyhow!("{key} parse error: {e}"))
}

/// Read a raw string env var, or `default` if unset.
pub fn get_string_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_key(suffix: &str) -> String {
        format!("RSKIT_ENVUTIL_TEST_{}_{}", std::process::id(), suffix)
    }

    #[test]
    fn get_unset_returns_none() {
        let key = unique_key("unset");
        assert!(get::<i32>(&key).is_none());
    }

    #[test]
    fn get_or_default() {
        let key = unique_key("or");
        assert_eq!(get_or(&key, 99), 99);
    }

    #[test]
    fn require_unset_errors() {
        let key = unique_key("req");
        assert!(require::<i32>(&key).is_err());
    }

    #[test]
    fn set_then_get() {
        let key = unique_key("set");
        // set_var/remove_var are unsafe in edition 2024 (env mutation isn't thread-safe).
        unsafe {
            std::env::set_var(&key, "123");
        }
        assert_eq!(get::<i32>(&key), Some(123));
        assert_eq!(require::<i32>(&key).unwrap(), 123);
        unsafe {
            std::env::remove_var(&key);
        }
    }

    #[test]
    fn invalid_value_returns_none_or_err() {
        let key = unique_key("invalid");
        unsafe {
            std::env::set_var(&key, "not-a-number");
        }
        assert_eq!(get::<i32>(&key), None);
        assert!(require::<i32>(&key).is_err());
        unsafe {
            std::env::remove_var(&key);
        }
    }

    #[test]
    fn parse_bool() {
        let key = unique_key("bool");
        unsafe {
            std::env::set_var(&key, "true");
        }
        assert_eq!(get::<bool>(&key), Some(true));
        unsafe {
            std::env::remove_var(&key);
        }
    }

    #[test]
    fn string_or_default() {
        let key = unique_key("str");
        assert_eq!(get_string_or(&key, "fallback"), "fallback");
        unsafe {
            std::env::set_var(&key, "real");
        }
        assert_eq!(get_string_or(&key, "fallback"), "real");
        unsafe {
            std::env::remove_var(&key);
        }
    }
}
