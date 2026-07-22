use rocket::fs::TempFile;
use rocket::form::Form;
use rocket_dyn_templates::{Template, context};
use rand::{distributions::Alphanumeric, Rng};
use std::path::Path;
use crate::user::AuthenticatedUser;

#[derive(FromForm)]
pub struct Upload<'r> {
    file: TempFile<'r>,
}

#[post("/upload", data = "<form>")]
pub async fn upload(form: Form<Upload<'_>>, _user: AuthenticatedUser) -> Template {
    let mut file = form.into_inner().file;
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    
    let ext_str = if let Some(original_name) = file.name() {
        let path = Path::new(original_name);
        if let Some(ext) = path.extension() {
            ext.to_string_lossy().to_lowercase()
        } else {
            if let Some(content_type) = file.content_type() {
                match content_type.to_string().as_str() {
                    "image/png" => "png".to_string(),
                    "image/jpeg" => "jpg".to_string(),
                    "image/gif" => "gif".to_string(),
                    "application/pdf" => "pdf".to_string(),
                    "text/plain" => "txt".to_string(),
                    _ => return Template::render("upload", context! {error: format!("Unsupported MIME: {}", content_type)})
                }
            } else {
                return Template::render("upload", context! {error: "No extension and no MIME"})
            }
        }
    } else {
        return Template::render("upload", context! {error: "No filename!"})
    };
    
    let file_link = format!("static/upload/{}.{}", s, ext_str);
    let link = format!("/{}", file_link);
    match file.persist_to(file_link).await {
        Ok(()) => Template::render("upload", context! {link: link}),
        Err(e) => Template::render("upload", context! {error: format!("Save fail: {}", e)})
    }
}   