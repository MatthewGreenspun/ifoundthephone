#[macro_use]
extern crate rocket;
use rocket_dyn_templates::Template;
mod user;
use rocket::fs::{FileServer, relative};

#[get("/")]
fn index() -> Template {
    let context = user::User{
        is_signed_in: false, 
        user_name: String::from("")
    };
    Template::render("index", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/static", FileServer::from(relative!("/static")))
        .mount("/", routes![index])
        .attach(Template::fairing())
}