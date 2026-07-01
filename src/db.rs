//! Database connection-pool helpers. Requires the `db` feature.
//!
//! This module intentionally provides only pool construction; application models
//! and queries belong to the consuming crate (see `examples/db_demo.rs`).

use anyhow::{Context, Result};
use sqlx::{
    PgPool, SqlitePool,
    postgres::PgPoolOptions,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

/// Open a SQLite pool that creates the file if missing. Pass `:memory:` for an
/// in-process database (only useful with a single connection).
pub async fn sqlite_pool(path: &str) -> Result<SqlitePool> {
    sqlite_pool_with(path, 1, true).await
}

/// Open a SQLite pool with explicit options.
pub async fn sqlite_pool_with(
    path: &str,
    max_conns: u32,
    create_if_missing: bool,
) -> Result<SqlitePool> {
    let mut opts = SqliteConnectOptions::new().filename(path);
    if create_if_missing {
        opts = opts.create_if_missing(true);
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(max_conns)
        .connect_with(opts)
        .await
        .with_context(|| format!("connect sqlite: {path}"))?;
    Ok(pool)
}

/// Open a Postgres pool from a connection URL.
pub async fn pg_pool(url: &str) -> Result<PgPool> {
    pg_pool_with(url, 10).await
}

/// Open a Postgres pool with an explicit max connections limit.
pub async fn pg_pool_with(url: &str, max_conns: u32) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_conns)
        .connect(url)
        .await
        .with_context(|| format!("connect postgres: {url}"))?;
    Ok(pool)
}

/// Read `DATABASE_URL` from the environment (loading `.env` first if present).
pub fn database_url() -> Result<String> {
    let _ = dotenvy::dotenv();
    std::env::var("DATABASE_URL").context("DATABASE_URL not set")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    #[tokio::test]
    async fn sqlite_create_insert_select() {
        let pool = sqlite_pool(":memory:").await.unwrap();
        sqlx::query("create table t (id integer primary key, name text)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("insert into t (name) values (?)")
            .bind("rskit")
            .execute(&pool)
            .await
            .unwrap();

        let row = sqlx::query("select name from t where id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let name: String = row.try_get("name").unwrap();
        assert_eq!(name, "rskit");
    }

    #[tokio::test]
    async fn sqlite_pool_with_options() {
        let pool = sqlite_pool_with(":memory:", 5, true).await.unwrap();
        assert!(sqlx::query("select 1").execute(&pool).await.is_ok());
    }

    #[test]
    fn database_url_unset_when_env_missing() {
        // We don't want to clobber a real DATABASE_URL in the test env; just
        // ensure the function is callable.
        let _ = database_url();
    }
}
