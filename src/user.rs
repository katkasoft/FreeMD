use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::Status;
use rocket::post;
use rocket::response::{Redirect, Responder};
use rocket::http::CookieJar;
use rocket::State;
use crate::db::DbPool;
use rocket::form::Form;
use rocket_dyn_templates::{Template, context};
use sqlx::Row;

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

#[derive(FromForm)]
pub struct ChangeUsername<'r> {
    pub username: &'r str
}

#[derive(Responder)]
pub enum UserResponse {
    Template(Template),
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

#[post("/change-username", data="<change_username_form>")]
pub async fn change_username(user: AuthenticatedUser, pool: &State<DbPool>, change_username_form: Form<ChangeUsername<'_>>) -> UserResponse {
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
    let username_old: String = user_row.get("username");
    let created_at: String = user_row.get("created_at");
    let username_new = change_username_form.username;
    if username_new == username_old || username_new.is_empty() {
        return UserResponse::Template(
            Template::render("account_settings", context! {
                username: username_old,
                created_at: created_at
            })
        );
    }
    let username_length = username_new.chars().count();
    if username_length < 2 || username_length > 50 {
        return UserResponse::Template(
            Template::render("account_settings", context! {
                username: username_old,
                created_at: created_at,
                error: "Username must be from 2 to 50 chars"
            })
        );
    }
    let existing = sqlx::query("SELECT id FROM users WHERE username = ?")
        .bind(&username_new)
        .fetch_optional(&**pool)
        .await;
    if let Ok(Some(_)) = existing {
        return UserResponse::Template(
            Template::render("account_settings", context! {
                username: username_old,
                created_at: created_at,
                error: "This nickname is already taken"
            })
        );
    }
    let result = sqlx::query("UPDATE users SET username = ? WHERE id = ?")
        .bind(&username_new)
        .bind(id)
        .execute(&**pool)
        .await;
    match result {
        Ok(_) => {
            let _ = sqlx::query("UPDATE articles SET author = ? WHERE author = ?")
                .bind(&username_new)
                .bind(&username_old)
                .execute(&**pool)
                .await;
            UserResponse::Template(
                Template::render("account_settings", context! {
                    username: username_new,
                    created_at: created_at
                })
            )
        },
        Err(_) => {
            UserResponse::Template(
                Template::render("account_settings", context! {
                    username: username_old,
                    created_at: created_at,
                    error: "Internal server error"
                })
            )
        }
    }
}