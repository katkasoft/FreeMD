#[macro_use] extern crate rocket;
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

mod pages;
mod articles_edit;
mod db;
mod articles_display;
mod search;
mod upload;
mod articles_opinion;
mod auth;
pub mod user;

#[launch]
async fn rocket() -> _ {
    let pool = db::init_db().await;
    rocket::build()
        .attach(Template::fairing())
        .manage(pool)
        .mount("/", routes![
            pages::index,
            pages::new_page,
            articles_display::article,
            search::search,
            pages::edit,
            pages::upload_page,
            pages::register_page,
            pages::login_page
        ])
        .mount("/api", routes![
            articles_edit::create_page,
            articles_edit::edit_page,
            upload::upload,
            articles_opinion::vote,
            auth::register,
            auth::login,
            articles_opinion::vote_status
        ])
        .mount("/static", FileServer::from(relative!("static")))
}