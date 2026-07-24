use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::Status;
use rocket::post;
use rocket::response::Redirect;
use rocket::http::CookieJar;

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

#[post("/logout")]
pub async fn logout(_user: AuthenticatedUser, cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private("user_id");
    Redirect::to(uri!("/"))
}