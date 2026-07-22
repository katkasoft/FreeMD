use rocket::form::Form;
use rocket::post;
use rocket::State;
use sqlx::Row;
use crate::db::DbPool;
use rocket::http::Status;
use serde_json::json;
use rocket::serde::json::Json;
use crate::user::AuthenticatedUser;

#[derive(FromForm)]
pub struct Vote {
    pub option: String,
    pub id: i64
}

#[post("/vote", data="<vote_form>")]
pub async fn vote(pool: &State<DbPool>, vote_form: Form<Vote>, user: AuthenticatedUser) -> Result<Json<serde_json::Value>, Status> {
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
    } else {
        return Err(Status::BadRequest)
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
    let user_id = user.id;
    let vote: i32 = match option.as_str() {
        "up" => 1,
        "down" => -1,
        _ => unreachable!()
    };
    sqlx::query("INSERT INTO votes (user_id, article_id, value) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(id)
        .bind(vote)
        .execute(&**pool)
        .await
        .map_err(|e| {
            println!("Error while voting: {}", e);
            Status::InternalServerError
        })?;
    Ok(Json(json!({ "score": score })))
}