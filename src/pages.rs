use rocket_dyn_templates::{Template, context};
use crate::db::DbPool;
use rocket::State;
use sqlx::Row;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub content: String,
}

#[get("/")]
pub async fn index(pool: &State<DbPool>) -> Template {
    let rows = sqlx::query("SELECT id, title, content FROM articles")
        .fetch_all(&**pool)
        .await
        .expect("Ошибка при получении статей, ты криворукий");

    let articles: Vec<Article> = rows
        .into_iter()
        .map(|row| Article {
            id: row.get(0),
            title: row.get(1),
            content: row.get(2),
        })
        .collect();

    Template::render("index", context! { rows: articles })
}

#[get("/new")]
pub fn new_page() -> Template {
    Template::render("new", context! {})
}