use rocket::State;
use rocket::serde::Serialize;
use rocket_dyn_templates::{Template, context};
use sqlx::FromRow;
use crate::db::DbPool;
use rocket::http::Status;
use rocket::http::CookieJar;
use sqlx::Row;

#[derive(Serialize, FromRow)]
#[serde(crate = "rocket::serde")]
struct Article {
    title: String,
    content: String,
    score: i32,
    author: String,
    editable_for_all: i32,
    visibility: String
}

#[derive(Debug, Serialize)]
pub struct Comment {
    pub author: String,
    pub content: String,
    pub created_at: String
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

#[get("/article/<id>")]
pub async fn article(id: u32, pool: &State<DbPool>, cookies: &CookieJar<'_>) -> Result<Template, Status> {
    let result = sqlx::query_as::<_, Article>("SELECT title, content, score, author, editable_for_all, visibility FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.inner())
        .await;
    match result {
        Ok(Some(article)) => {
            let rows = sqlx::query("SELECT author, content, created_at FROM comments WHERE article_id = ?")
                .bind(id)
                .fetch_all(pool.inner())
                .await
                .expect("Error while getting comment");
            let comments: Vec<Comment> = rows
                .into_iter()
                .map(|row| {
                    Comment {
                        author: row.get(0),
                        content: row.get(1),
                        created_at: row.get(2)
                    }
                })
                .collect();
            let login = cookies.get_private("user_id").is_some();
            let username = get_current_username(pool.inner(), cookies).await;
            let is_author = username.as_deref() == Some(article.author.as_str());
            if article.visibility == "private" && !is_author {
                return Err(Status::Forbidden)
            }
            Ok(Template::render("article", context! {
                title: article.title,
                content: article.content,
                score: article.score,
                login: login,
                author: article.author.clone(),
                username: username.clone(),
                is_author: is_author,
                id: id,
                comments: comments,
                is_editable: article.editable_for_all == 1 || is_author
            }))
        }
        Ok(None) => {
            Err(Status::NotFound)
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}