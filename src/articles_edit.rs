use rocket::form::Form;
use rocket::post;
use rocket::response::{Redirect, Responder};
use rocket_dyn_templates::{Template, context};
use rocket::State;
use crate::db::DbPool;
use crate::user::AuthenticatedUser;
use sqlx::Row;

#[derive(FromForm)]
pub struct NewPage<'r> {
    pub title: &'r str,
    pub content: &'r str,
}

#[derive(FromForm)]
pub struct EditPage<'r> {
    pub title: &'r str,
    pub content: &'r str,
    pub id: i64
}

#[derive(Responder)]
pub enum CreatePageResponse {
    Template(Template),
    Redirect(Redirect),
}

#[post("/new", data = "<page_form>")]
pub async fn create_page(
    page_form: Form<NewPage<'_>>, 
    pool: &State<DbPool>,
    user: AuthenticatedUser
) -> CreatePageResponse {
    let title = page_form.title.trim();
    let content = page_form.content.trim();
    if title.is_empty() || content.is_empty() {
        return CreatePageResponse::Template(Template::render("editor", context! { 
            error: "Not all fields are filled in" 
        }));
    }
    if title.len() > 100 || content.len() > 10000 {
        return CreatePageResponse::Template(Template::render("editor", context! { 
            error: "Too many text! Limits: 100 chars max for title and 10000 for content" 
        }));
    }
    let id: i64 = user.id;
    let user_row = sqlx::query("SELECT username FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await;
    let username: String = match user_row {
        Ok(Some(row)) => row.get("username"),
        _ => return CreatePageResponse::Template(Template::render("editor", context! {
            error: "Internal server error"
        })),
    };
    let result = sqlx::query("INSERT INTO articles (title, content, author) VALUES (?, ?, ?)")
        .bind(title)
        .bind(content)
        .bind(&username)
        .execute(&**pool)
        .await;
    match result {
        Ok(_) => CreatePageResponse::Redirect(Redirect::to(uri!("/"))),
        Err(e) => {
            CreatePageResponse::Template(Template::render("editor", context! { 
                error: format!("Internal server error: {}", e) 
            }))
        }
    }
}

#[post("/edit", data = "<edit_form>")]
pub async fn edit_page(
    edit_form: Form<EditPage<'_>>, 
    pool: &State<DbPool>,
    _user: AuthenticatedUser
) -> CreatePageResponse  {
    let title = edit_form.title.trim();
    let content = edit_form.content.trim();
    let id = edit_form.id;
    if title.is_empty() || content.is_empty() {
        return CreatePageResponse::Template(Template::render("editor", context! { 
            error: "Not all fields are filled in",
            edit: true,
            title: title,
            content: content,
            id: id
        }));
    }
    if title.len() > 100 || content.len() > 10000 {
        return CreatePageResponse::Template(Template::render("editor", context! { 
            error: "Too many text! Limits: 100 chars max for title and 10000 for content",
            edit: true,
            title: title,
            content: content,
            id: id
        }));
    }
    let result = sqlx::query("UPDATE articles SET title = ?, content = ? WHERE id = ?")
        .bind(title)
        .bind(content)
        .bind(id)
        .execute(&**pool)
        .await;
    match result {
        Ok(_) => CreatePageResponse::Redirect(Redirect::to(uri!("/"))),
        Err(e) => {
            CreatePageResponse::Template(Template::render("editor", context! { 
                error: format!("Internal server error: {}", e),
                edit: true,
                title: title,
                content: content,
                id: id
            }))
        }
    }
}