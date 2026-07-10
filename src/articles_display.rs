use rocket::State;
use rocket::serde::Serialize;
use rocket_dyn_templates::{Template, context};
use sqlx::FromRow;
use crate::db::DbPool;
use rocket::http::Status;

#[derive(Serialize, FromRow)]
#[serde(crate = "rocket::serde")]
struct Article {
    title: String,
    content: String,
    score: i32
}

#[get("/article/<id>")]
pub async fn article(id: u32, pool: &State<DbPool>) -> Result<Template, Status> {
    let result = sqlx::query_as::<_, Article>("SELECT title, content, score FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.inner())
        .await;
    match result {
        Ok(Some(article)) => {
            Ok(Template::render("article", context! {
                title: article.title,
                content: article.content,
                score: article.score
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