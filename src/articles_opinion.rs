use rocket::form::Form;
use rocket::post;
use rocket::State;
use sqlx::Row;
use crate::db::DbPool;
use rocket::http::Status;
use serde_json::json;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use crate::user::AuthenticatedUser;

#[derive(FromForm)]
pub struct Vote {
    pub option: String,
    pub id: i64
}

#[derive(FromForm)]
pub struct Comment<'r> {
    pub article_id: i64,
    pub content: &'r str,
}

#[derive(Serialize)]
pub struct ApiResponse<'r> {
    pub status: &'r str,
    pub message: &'r str,
}

#[post("/vote", data="<vote_form>")]
pub async fn vote(pool: &State<DbPool>, vote_form: Form<Vote>, user: AuthenticatedUser) -> Result<Json<serde_json::Value>, Status> {
    let id = vote_form.id;
    let option = &vote_form.option;
    let user_id = user.id;
    let vote: i32 = match option.as_str() {
        "up" => 1,
        "down" => -1,
        _ => return Err(Status::BadRequest),
    };
    let insert_result = sqlx::query("INSERT INTO votes (user_id, article_id, value) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(id)
        .bind(vote)
        .execute(&**pool)
        .await;
    if let Err(e) = insert_result {
        println!("User already voted or invalid article: {}", e);
        return Err(Status::BadRequest);
    }
    let row = sqlx::query("SELECT score FROM articles WHERE id = ?")
        .bind(id)
        .fetch_one(&**pool)
        .await
        .map_err(|_| Status::NotFound)?;
    let mut score: i32 = row.get::<i32, _>("score");
    score += vote;
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

#[get("/vote_status?<id>")]
pub async fn vote_status(pool: &State<DbPool>, user: AuthenticatedUser, id: i64) -> Result<String, Status> {
    let user_id = user.id;
    let row = sqlx::query("SELECT value FROM votes WHERE user_id = ? AND article_id = ?")
        .bind(user_id)
        .bind(id)
        .fetch_optional(&**pool)
        .await;
    let status_text = match row {
        Ok(Some(r)) => match r.get::<i32, _>("value") {
            1 => "up",
            -1 => "down",
            _ => "none",
        },
        Ok(None) => "none",
        Err(_) => "error"
    };
    Ok(status_text.to_string())
}

#[post("/new-comment", data="<comment_form>")]
pub async fn comment(pool: &State<DbPool>, user: AuthenticatedUser, comment_form: Form<Comment<'_>>) -> Json<ApiResponse<'static>> {
    let article_id = comment_form.article_id;
    let content = comment_form.content;
    let user_id = user.id;
    if content.is_empty() {
        return Json(ApiResponse{
            status: "error",
            message: "Comment cannot be empty"
        });
    }
    if content.chars().count() > 1000 {
        return Json(ApiResponse{
            status: "error",
            message: "Comment is too long: 1000 chars max"
        });
    }
    let username: Option<String> = sqlx::query("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("username"));
    let result = sqlx::query("INSERT INTO comments (author, article_id, content) VALUES (?, ?, ?)")
        .bind(username)
        .bind(article_id)
        .bind(content)
        .execute(&**pool)
        .await;
    match result {
        Ok(_) => {
            Json(ApiResponse {
                status: "success",
                message: "OK",
            })
        }
        Err(_) => {
            Json(ApiResponse {
                status: "error",
                message: "Internal server error",
            })
        }
    }
}