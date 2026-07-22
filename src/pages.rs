use rocket_dyn_templates::{Template, context};
use crate::db::DbPool;
use rocket::State;
use sqlx::Row;
use serde::Serialize;
use crate::user::AuthenticatedUser;
use rocket::http::CookieJar;

#[derive(Debug, Serialize)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub content: String,
}

#[get("/")]
pub async fn index(pool: &State<DbPool>, cookies: &CookieJar<'_>) -> Template {
    let rows = sqlx::query("SELECT id, title, content FROM articles ORDER BY created_at DESC")
        .fetch_all(&**pool)
        .await
        .expect("Error while getting articles");

    let articles: Vec<Article> = rows
        .into_iter()
        .map(|row| {
            let full_content: String = row.get(2);
            let preview = if full_content.len() > 100 {
                full_content.chars().take(100).collect::<String>() + "..."
            } else {
                full_content
            };
            Article {
                id: row.get(0),
                title: row.get(1),
                content: preview,
            }
        })
        .collect();
    let login = cookies.get_private("user_id").is_some();
    Template::render("index", context! { rows: articles, login: login })
}

#[get("/new")]
pub fn new_page(_user: AuthenticatedUser) -> Template {
    Template::render("editor", context! {})
}

#[get("/edit?<id>")]
pub async fn edit(id: i64, pool: &State<DbPool>, _user: AuthenticatedUser) -> Template {
    let row = sqlx::query("SELECT id, title, content FROM articles WHERE id = ?")
        .bind(id)
        .fetch_one(&**pool)
        .await
        .expect("Error when getting article");

    let article = Article {
        id: row.get("id"),
        title: row.get("title"),
        content: row.get("content"),
    };

    Template::render("editor", context! { 
        edit: true, 
        id: article.id, 
        title: article.title, 
        content: article.content 
    })
}

#[get("/upload")]
pub fn upload_page(_user: AuthenticatedUser) -> Template {
    Template::render("upload", context! {})
}

#[get("/login")]
pub fn login_page() -> Template {
    Template::render("login", context! {})
}

#[get("/register")]
pub fn register_page() -> Template {
    Template::render("register", context! {})
}