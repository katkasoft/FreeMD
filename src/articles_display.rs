use rocket::State;
use rocket::serde::Serialize;
use rocket_dyn_templates::{Template, context};
use sqlx::FromRow;
use crate::db::DbPool;
use rocket::http::Status;
use rocket::http::CookieJar;

#[derive(Serialize, FromRow)]
#[serde(crate = "rocket::serde")]
struct Article {
    title: String,
    content: String,
    score: i32,
    author: String
}

#[get("/article/<id>")]
pub async fn article(id: u32, pool: &State<DbPool>, cookies: &CookieJar<'_>) -> Result<Template, Status> {
    let result = sqlx::query_as::<_, Article>("SELECT title, content, score, author FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.inner())
        .await;
    match result {
        Ok(Some(article)) => {
            let login = cookies.get_private("user_id").is_some();
            Ok(Template::render("article", context! {
                title: article.title,
                content: article.content,
                score: article.score,
                login: login,
                author: article.author
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