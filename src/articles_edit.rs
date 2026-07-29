use rocket::form::Form;
use rocket::http::Status;
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
    pub editable_for_all: Option<bool>
}

#[derive(FromForm)]
pub struct EditPage<'r> {
    pub title: &'r str,
    pub content: &'r str,
    pub id: i64,
    pub editable_for_all: Option<bool>
}

#[derive(FromForm)]
pub struct DeleteArticle {
    pub id: i64
}

#[derive(Responder)]
pub enum CreatePageResponse {
    Template(Template),
    Redirect(Redirect),
    Status(Status)
}

#[post("/new", data = "<page_form>")]
pub async fn create_page(
    page_form: Form<NewPage<'_>>, 
    pool: &State<DbPool>,
    user: AuthenticatedUser
) -> CreatePageResponse {
    let title = page_form.title.trim();
    let content = page_form.content.trim();
    let editable =  if page_form.editable_for_all.unwrap_or(false) { 1 } else { 0 };
    if title.is_empty() || content.is_empty() {
        return CreatePageResponse::Template(Template::render("editor", context! { 
            error: "Not all fields are filled in" 
        }));
    }
    if title.len() > 200 || content.len() > 100000 {
        return CreatePageResponse::Template(Template::render("editor", context! { 
            error: "Too much text! Limits: 200 chars max for title and 100000 for content" 
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
    let result = sqlx::query("INSERT INTO articles (title, content, author, editable_for_all) VALUES (?, ?, ?, ?)")
        .bind(title)
        .bind(content)
        .bind(&username)
        .bind(editable)
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
    user: AuthenticatedUser
) -> CreatePageResponse  {
    let user_id = user.id;
    let id = edit_form.id;
    let editable =  if edit_form.editable_for_all.unwrap_or(false) { 1 } else { 0 };
    let username: Option<String> = sqlx::query("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("username"));
    let author: Option<String> = sqlx::query("SELECT author FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("author"));
    let db_editable: i32 = sqlx::query("SELECT editable_for_all FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("editable_for_all"))
        .unwrap_or(0);

    if username != author && db_editable != 1 {
        return CreatePageResponse::Status(Status::Forbidden);
    }
    let title = edit_form.title.trim();
    let content = edit_form.content.trim();
    if title.is_empty() || content.is_empty() {
        return CreatePageResponse::Template(Template::render("editor", context! { 
            error: "Not all fields are filled in",
            edit: true,
            title: title,
            content: content,
            id: id
        }));
    }
    if title.len() > 200 || content.len() > 100000 {
        return CreatePageResponse::Template(Template::render("editor", context! { 
            error: "Too much text! Limits: 200 chars max for title and 100000 for content",
            edit: true,
            title: title,
            content: content,
            id: id
        }));
    }
    let result = sqlx::query("UPDATE articles SET title = ?, content = ?, editable_for_all = ?    WHERE id = ?")
        .bind(title)
        .bind(content)
        .bind(id)
        .bind(editable)
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

#[post("/delete-article", data = "<delete_form>")]
pub async fn delete_article(
    delete_form: Form<DeleteArticle>,
    pool: &State<DbPool>,
    user: AuthenticatedUser
) -> CreatePageResponse {
    let id = delete_form.id;
    let user_id = user.id;
    let username: Option<String> = sqlx::query("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("username"));
    let author: Option<String> = sqlx::query("SELECT author FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await
        .unwrap_or(None)
        .map(|row| row.get("author"));
    if username != author {
        return CreatePageResponse::Status(Status::Forbidden)
    }
    let result = sqlx::query("DELETE FROM articles WHERE id = ?")
        .bind(id)
        .execute(&**pool)
        .await;
    match result {
        Ok(_) => CreatePageResponse::Redirect(Redirect::to(uri!("/"))),
        Err(_) => CreatePageResponse::Status(Status::InternalServerError)
    }
}