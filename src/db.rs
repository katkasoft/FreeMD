use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
use std::str::FromStr;

pub type DbPool = SqlitePool;

pub async fn init_db() -> DbPool {
    let database_url = "freemd.db";

    let options = SqliteConnectOptions::from_str(database_url)
        .expect("Error while opening database file")
        .create_if_missing(true); 

    let pool = SqlitePool::connect_with(options)
        .await
        .expect("Error while initialising database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            score INTEGER NOT NULL DEFAULT 0
        );"
    )
    .execute(&pool)
    .await
    .expect("Error while creating articles table");

    pool
}