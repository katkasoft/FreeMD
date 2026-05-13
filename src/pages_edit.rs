use rocket::form::Form;
use rocket::post;
use rocket::response::{Redirect, Responder};
use rocket_dyn_templates::{Template, context};
use rocket::State;
use crate::db::DbPool;

#[derive(FromForm)]
pub struct NewPage<'r> {
    pub title: &'r str,
    pub content: &'r str,
}

#[derive(Responder)]
pub enum CreatePageResponse {
    Template(Template),
    Redirect(Redirect),
}

#[post("/new", data = "<page_form>")]
pub async fn create_page(
    page_form: Form<NewPage<'_>>, 
    pool: &State<DbPool> 
) -> CreatePageResponse {
    let title = page_form.title.trim();
    let content = page_form.content.trim();
    if title.is_empty() || content.is_empty() {
        return CreatePageResponse::Template(Template::render("new", context! { 
            error: "Not all fields are filled in" 
        }));
    }
    if title.len() > 100 || content.len() > 10000 {
        return CreatePageResponse::Template(Template::render("new", context! { 
            error: "Too many text! Limits: 100 chars max for title and 10000 for content" 
        }));
    }
    let result = sqlx::query("INSERT INTO pages (title, content) VALUES (?, ?)")
        .bind(title)
        .bind(content)
        .execute(&**pool)
        .await;
    match result {
        Ok(_) => CreatePageResponse::Redirect(Redirect::to(uri!("/"))),
        Err(e) => {
            CreatePageResponse::Template(Template::render("new", context! { 
                error: format!("Internal server error: {}", e) 
            }))
        }
    }
}