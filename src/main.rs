#[macro_use]
extern crate rocket;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use rocket::fs::{FileServer, relative};

#[get("/")]
fn index() -> Template {
    let context: HashMap<String, String> = HashMap::new();
    Template::render("index", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/static", FileServer::from(relative!("/static")))
        .mount("/", routes![index])
        .attach(Template::fairing())
}