use rocket_dyn_templates::{Template, context};
use crate::db::DbPool;
use rocket::State;
use sqlx::Row;
use serde::Serialize;
use crate::user::AuthenticatedUser;
use rocket::http::CookieJar;
use rocket::http::Status;

#[derive(Debug, Serialize)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub content: String,
}

#[derive(Responder)]
pub enum UserResponse {
    Template(Template),
    Status(Status),
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

#[get("/user/<username>")]
pub async fn user(pool: &State<DbPool>, username: String, cookies: &CookieJar<'_>, user: AuthenticatedUser) -> UserResponse {
    let row = sqlx::query("SELECT created_at, id FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&**pool)
        .await;
    let user_row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return UserResponse::Status(Status::NotFound),
        Err(_) => return UserResponse::Status(Status::InternalServerError)
    };
    let created_at: String = user_row.get("created_at");
    let user_id: i64 = user_row.get("id");
    let rows_articles = sqlx::query("SELECT id, title, content FROM articles WHERE author = ? ORDER BY created_at DESC")
        .bind(&username)
        .fetch_all(&**pool)
        .await
        .expect("Error while getting articles");
    let articles: Vec<Article> = rows_articles
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
    UserResponse::Template(
        Template::render("user", context! {
            rows: articles,
            login: login,
            username: username,
            created_at: created_at,
            is_my_profile: user_id == user.id
        })
    )
}

#[get("/account-settings")]
pub fn account_settings() -> Template {
    Template::render("account_settings", context! {})
}