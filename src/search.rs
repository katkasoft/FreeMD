use rocket_dyn_templates::{Template, context};
use crate::db::DbPool;
use rocket::State;
use sqlx::Row;
use serde::Serialize;
use rocket::http::CookieJar;

#[derive(Debug, Serialize)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub content: String,
}

async fn get_current_username(pool: &DbPool, cookies: &CookieJar<'_>) -> Option<String> {
    let current_user_id: i64 = cookies.get_private("user_id")
        .and_then(|cookie| cookie.value().parse().ok())?;
    let row = sqlx::query("SELECT username FROM users WHERE id = ?")
        .bind(current_user_id)
        .fetch_optional(pool)
        .await
        .ok()?;
    row.map(|r| r.get("username"))
}

#[get("/search?<q>")]
pub async fn search(q: String, pool: &State<DbPool>, cookies: &CookieJar<'_>) -> Template {
    let login = cookies.get_private("user_id").is_some();
    let username = get_current_username(&**pool, cookies).await;
    let current_user = username.clone().unwrap_or_default();
    let search_term = format!("%{}%", q.to_lowercase());
    let rows = sqlx::query(
        "SELECT id, title, content FROM articles \
        WHERE (LOWER(title) LIKE ? OR LOWER(content) LIKE ?) \
        AND (visibility = 'public' OR (visibility IN ('private', 'link') AND author = ?)) \
        ORDER BY created_at DESC"
    )
        .bind(&search_term)
        .bind(&search_term)
        .bind(current_user)
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
    Template::render("index", context! { 
        rows: articles, 
        query: q,
        login: login,
        username: username
    })
}

#[get("/tag/<tag>")]
pub async fn search_by_tag(tag: String, pool: &State<DbPool>, cookies: &CookieJar<'_>) -> Template {
    let login = cookies.get_private("user_id").is_some();
    let username = get_current_username(&**pool, cookies).await;
    let current_user = username.clone().unwrap_or_default();
    let search_term = format!("%{}%", tag.to_lowercase());
    let rows = sqlx::query(
        "SELECT id, title, content FROM articles \
        WHERE (LOWER(tags) = ? OR LOWER(tags) LIKE ?) \
        AND (visibility = 'public' OR (visibility IN ('private', 'link') AND author = ?)) \
        ORDER BY created_at DESC"
    )
        .bind(&search_term)
        .bind(&search_term)
        .bind(current_user)
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
    Template::render("index", context! { 
        rows: articles, 
        tag: tag,
        login: login,
        username: username
    })
}