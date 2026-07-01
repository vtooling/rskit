//! Demonstrates using `rskit::db` to build a SQLite pool and run queries with
//! application-defined models. (Run with `cargo run --example db_demo --features db`.)

use rskit::db;
use sqlx::{FromRow, Row};

#[derive(Debug, Default, FromRow)]
#[allow(dead_code)]
struct User {
    id: Option<i64>,
    username: Option<String>,
    age: Option<u32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = db::sqlite_pool(":memory:").await?;

    sqlx::query(
        "create table user (id integer primary key autoincrement, username text, age integer)",
    )
    .execute(&pool)
    .await?;

    let user = User {
        id: None,
        username: Some("zhangsan".into()),
        age: Some(18),
    };
    sqlx::query("insert into user (username, age) values (?, ?)")
        .bind(user.username.clone().unwrap())
        .bind(user.age.unwrap())
        .execute(&pool)
        .await?;

    let users: Vec<User> = sqlx::query_as::<_, User>("select * from user")
        .fetch_all(&pool)
        .await?;
    println!("users: {users:?}");

    let id: i64 = sqlx::query("select id from user where username = ?")
        .bind("zhangsan")
        .fetch_one(&pool)
        .await?
        .try_get(0)?;
    println!("zhangsan id = {id}");

    Ok(())
}
