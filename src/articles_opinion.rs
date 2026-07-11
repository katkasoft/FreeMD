use rocket::form::Form;
use rocket::post;
use rocket::State;
use sqlx::Row;
use crate::db::DbPool;
use rocket::http::Status;
use serde_json::json;
use rocket::serde::json::Json;

#[derive(FromForm)]
pub struct Vote {
    pub option: String,
    pub id: i64
}

#[post("/vote", data="<vote_form>")]
pub async fn vote(pool: &State<DbPool>, vote_form: Form<Vote>) -> Result<Json<serde_json::Value>, Status> {
    let id = vote_form.id;
    let option = &vote_form.option;
    
    let row = sqlx::query("SELECT score FROM articles WHERE id = ?")
        .bind(id)
        .fetch_one(&**pool)
        .await
        .map_err(|_| Status::NotFound)?;
    let mut score: i32 = row.get::<i32, _>("score");
    if option == "up" {
        score += 1;
    } else if option == "down" {
        score -= 1;
    }
    sqlx::query("UPDATE articles SET score = ? WHERE id = ?")
        .bind(score)
        .bind(id)
        .execute(&**pool)
        .await
        .map_err(|e| {
            println!("Error while voting: {}", e);
            Status::InternalServerError
        })?;
    Ok(Json(json!({ "score": score })))
}