use rocket::State;
use rocket::serde::Serialize;
use rocket_dyn_templates::{Template, context};
use sqlx::FromRow;
use crate::db::DbPool;

#[derive(Serialize, FromRow)]
#[serde(crate = "rocket::serde")]
struct Article {
    title: String,
    content: String,
}

#[get("/article/<id>")]
pub async fn article(id: u32, pool: &State<DbPool>) -> Option<Template> {
    let result = sqlx::query_as::<_, Article>("SELECT title, content FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.inner())
        .await;
    match result {
        Ok(Some(article)) => Some(Template::render("article", context! {
            title: article.title,
            content: article.content
        })),
        Ok(None) => {
            None //TODO: сделать тут отображение ошибки
        },
        Err(e) => {
            eprintln!("database error: {}", e); //TODO: сделать тут отображение ошибки
            None
        }
    }
}