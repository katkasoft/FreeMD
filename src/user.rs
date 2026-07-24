use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::Status;
use rocket::post;
use rocket::response::{Redirect, Responder};
use rocket::http::CookieJar;
use rocket::State;
use crate::db::DbPool;

pub struct AuthenticatedUser {
    pub id: i64,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        if let Some(cookie) = cookies.get_private("user_id") {
            if let Ok(user_id) = cookie.value().parse::<i64>() {
                return Outcome::Success(AuthenticatedUser { id: user_id });
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}

#[derive(Responder)]
pub enum RedirectOrStatus {
    Redirect(Redirect),
    Status(Status),
}

#[post("/logout")]
pub async fn logout(_user: AuthenticatedUser, cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private("user_id");
    Redirect::to(uri!("/"))
}

#[post("/delete")]
pub async fn delete(user: AuthenticatedUser, cookies: &CookieJar<'_>, pool: &State<DbPool>) -> RedirectOrStatus {
    let id = user.id;
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&**pool)
        .await;
    match result {
        Ok(_) => {
            cookies.remove_private("user_id");
            RedirectOrStatus::Redirect(Redirect::to(uri!("/")))
        },
        Err(_) => {
            RedirectOrStatus::Status(Status::InternalServerError)
        }
    }
}