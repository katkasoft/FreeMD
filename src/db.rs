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
            score INTEGER NOT NULL DEFAULT 0,
            author TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .expect("Error while creating articles table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(&pool)
    .await
    .expect("Error while creating users table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS votes (
            user_id INTEGER NOT NULL,
            article_id INTEGER NOT NULL,
            value INTEGER NOT NULL,
            PRIMARY KEY (user_id, article_id),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE
        );"
    )
    .execute(&pool)
    .await
    .expect("Error while creating votes table");


    pool
}