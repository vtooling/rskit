//! Redis helpers. Requires the `redis` feature (async, tokio runtime).
//!
//! The original implementation called `.get()` directly on a `Client`, which is
//! not how the redis crate works (commands run on a connection). This module
//! opens a multiplexed async connection per operation; for high-throughput use,
//! prefer `connect()` once and reuse the connection.

use anyhow::Result;
use redis::{Client, FromRedisValue, ToRedisArgs, aio::MultiplexedConnection};
/// Default URL used when neither the argument nor `REDIS_URL` is provided.
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

/// Build a [`Client`] from an explicit URL.
pub fn client(url: &str) -> Result<Client> {
    Ok(Client::open(url)?)
}

/// Build a [`Client`] from `REDIS_URL` (or the default if unset).
pub fn client_from_env() -> Result<Client> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    client(&url)
}

/// Open a fresh multiplexed async connection from the env-configured client.
pub async fn connect() -> Result<MultiplexedConnection> {
    Ok(client_from_env()?
        .get_multiplexed_async_connection()
        .await?)
}

/// Set `key` to `val`.
pub async fn set<V: ToRedisArgs + Send + Sync>(key: &str, val: V) -> Result<()> {
    let mut conn = connect().await?;
    let (): () = redis::AsyncCommands::set(&mut conn, key, val).await?;
    Ok(())
}

/// Get `key`.
pub async fn get<V: FromRedisValue + Send + Sync>(key: &str) -> Result<V> {
    let mut conn = connect().await?;
    Ok(redis::AsyncCommands::get(&mut conn, key).await?)
}

/// Delete `key`. Returns the number of keys removed.
pub async fn del(key: &str) -> Result<i64> {
    let mut conn = connect().await?;
    Ok(redis::AsyncCommands::del::<&str, i64>(&mut conn, key).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_from_explicit_url() {
        assert!(client("redis://127.0.0.1:6379").is_ok());
    }

    #[test]
    fn client_from_env_succeeds() {
        // Either a real REDIS_URL or the default; both parse.
        assert!(client_from_env().is_ok());
    }

    #[test]
    fn client_invalid_url_errors() {
        assert!(client("not a url").is_err());
    }
}
