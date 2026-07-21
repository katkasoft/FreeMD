use rocket::form::Form;
use rocket::post;
use rocket::response::{Redirect, Responder};
use rocket_dyn_templates::{Template, context};
use rocket::State;
use crate::db::DbPool;
use rocket::http::{Cookie, CookieJar};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

#[derive(FromForm)]
pub struct Login<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

#[derive(FromForm)]
pub struct Register<'r> {
    pub username: &'r str,
    pub password: &'r str,
    pub confirm: &'r str,
}

#[derive(Responder)]
pub enum Response {
    Template(Template),
    Redirect(Redirect),
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Hash error: {}", e))
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}

#[post("/register", data="<register_form>")]
pub async fn register(
    register_form: Form<Register<'_>>, 
    pool: &State<DbPool>,
    cookies: &CookieJar<'_>
) -> Response {
    let username = register_form.username;
    let password = register_form.password;
    let confirm = register_form.confirm;
    if username.is_empty() || password.is_empty() || confirm.is_empty() {
        return Response::Template(Template::render("register", context! {
            error: "Not all fields are filled in"
        }));
    }
    if password != confirm {
        return Response::Template(Template::render("register", context! {
            error: "Passwords not matching"
        }));
    }
    let username_length = username.chars().count();
    if username_length < 2 || username_length > 50 {
        return Response::Template(Template::render("register", context! {
            error: "Username must be from 2 to 50 chars"
        }));
    }
    let password_length = password.chars().count();
    if password_length < 4 || password_length > 50 {
        return Response::Template(Template::render("register", context! {
            error: "Password must be from 4 to 50 chars"
        }));
    }
    let password_hash = match hash_password(password) {
        Ok(hash) => hash,
        Err(_) => {
            return Response::Template(Template::render("register", context! {
                error: "Internal server error. Try again later"
            }));
        }
    };
    let result = sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
        .bind(username)
        .bind(password_hash)
        .execute(&**pool)
        .await;
    match result {
        Ok(res) => {
            let user_id = res.last_insert_rowid();
            cookies.add_private(Cookie::build(("user_id", user_id.to_string()))
                .path("/")
                .same_site(rocket::http::SameSite::Lax)
                .build());
            Response::Redirect(Redirect::to(uri!("/")))
        },
        Err(e) => {
            let err_msg = if e.to_string().contains("UNIQUE") {
                "Username already taken".to_string()
            } else {
                "Internal server error".to_string()
            };
            Response::Template(Template::render("register", context! { 
                error: err_msg 
            }))
        }
    }
}