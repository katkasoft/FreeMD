use rocket_dyn_templates::{Template, context};

#[get("/")]
pub fn index() -> Template {
    Template::render("index", context! {})
}

#[get("/new")]
pub fn new_page() -> Template {
    Template::render("new", context! {})
}