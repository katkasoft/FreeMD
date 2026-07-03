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

#[get("/search?<q>")]
pub async fn search(q: String, pool: &State<DbPool>) -> Template {
    let search_term = format!("%{}%", q.to_lowercase());
    let rows = sqlx::query(
        "SELECT id, title, content FROM articles \
        WHERE LOWER(title) LIKE ? OR LOWER(content) LIKE ? \
        ORDER BY created_at DESC"
    )
        .bind(&search_term)
        .bind(&search_term)
        .fetch_all(&**pool)
        .await
        .expect("Database error");
    let articles: Vec<Article> = rows
        .into_iter()
        .map(|row| {
            let full_content: String = row.get("content");
            let preview = if full_content.len() > 100 {
                full_content.chars().take(100).collect::<String>() + "..."
            } else {
                full_content
            };
            Article {
                id: row.get("id"),
                title: row.get("title"),
                content: preview,
            }
        })
        .collect();
    Template::render("index", context! { rows: articles })
}