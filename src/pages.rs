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
    let username = get_current_username(&**pool, cookies).await;
    
    Template::render("index", context! { 
        rows: articles, 
        login: login, 
        username: username 
    })
}

#[get("/new")]
pub fn new_page(_user: AuthenticatedUser) -> Template {
    Template::render("editor", context! {
        is_author: true
    })
}

#[get("/edit?<id>")]
pub async fn edit(id: i64, pool: &State<DbPool>, user: AuthenticatedUser) -> UserResponse {
    let user_id = user.id;
    let username: Option<String> = sqlx::query("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("username"));
    let author: Option<String> = sqlx::query("SELECT author FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("author"));
    let db_editable: i32 = sqlx::query("SELECT editable_for_all FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("editable_for_all"))
        .unwrap_or(0);
    if username != author && db_editable != 1 {
        return UserResponse::Status(Status::Forbidden);
    }
    let row = sqlx::query("SELECT id, title, content, visibility, share_link FROM articles WHERE id = ?")
        .bind(id)
        .fetch_one(&**pool)
        .await
        .expect("Error when getting article");
    let article = Article {
        id: row.get("id"),
        title: row.get("title"),
        content: row.get("content"),
    };
    let visibility: String = row.get("visibility");
    let share_link: Option<String> = row.get("share_link");
    UserResponse::Template(
        Template::render("editor", context! { 
            edit: true, 
            id: article.id, 
            title: article.title, 
            content: article.content,
            is_author: username == author,
            editable: db_editable,
            visibility: visibility,
            share_link: share_link
        })
    )
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
pub async fn user(pool: &State<DbPool>, username: String, cookies: &CookieJar<'_>) -> UserResponse {
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
    let current_user_id: Option<i64> = cookies.get_private("user_id")
        .and_then(|cookie| cookie.value().parse().ok());
    let login = current_user_id.is_some();
    let current_username = get_current_username(&**pool, cookies).await;
    
    UserResponse::Template(
        Template::render("user", context! {
            rows: articles,
            login: login,
            username: username,
            created_at: created_at,
            is_my_profile: Some(user_id) == current_user_id,
            current_username: current_username
        })
    )
}

#[get("/account-settings")]
pub async fn account_settings(pool: &State<DbPool>, user: AuthenticatedUser) -> UserResponse {
    let id = user.id;
    let row = sqlx::query("SELECT created_at, username FROM users WHERE id = ?")
        .bind(&id)
        .fetch_optional(&**pool)
        .await;
    let user_row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return UserResponse::Status(Status::NotFound),
        Err(_) => return UserResponse::Status(Status::InternalServerError)
    };
    let username: String = user_row.get("username");
    let created_at: String = user_row.get("created_at");
    UserResponse::Template(
        Template::render("account_settings", context! {
            username: username,
            created_at: created_at
        })
    )
}

#[get("/account-settings/change-password")]
pub fn change_password_page(_user: AuthenticatedUser) -> Template {
    Template::render("change_password", context! {})
}