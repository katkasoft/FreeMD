use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::Status;

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
