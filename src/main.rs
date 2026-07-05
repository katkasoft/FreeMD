#[macro_use] extern crate rocket;
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

mod pages;
mod articles_edit;
mod db;
mod articles_display;
mod search;

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
            pages::edit
        ])
        .mount("/api", routes![
            articles_edit::create_page
        ])
        .mount("/static", FileServer::from(relative!("static")))
}