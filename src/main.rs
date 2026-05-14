#[macro_use] extern crate rocket;
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

mod pages;
mod articles_edit;
mod db;

#[launch]
async fn rocket() -> _ {
    let pool = db::init_db().await;
    rocket::build()
        .attach(Template::fairing())
        .manage(pool)
        .mount("/", routes![
            pages::index,
            pages::new_page
        ])
        .mount("/api", routes![
            articles_edit::create_page
        ])
        .mount("/static", FileServer::from(relative!("static")))
}