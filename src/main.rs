#[macro_use]
extern crate rocket;
use rocket_dyn_templates::Template;
use rocket::fs::{FileServer, relative};
mod routes;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/static", FileServer::from(relative!("/static")))
        .mount("/", routes![routes::index, routes::sign_up_page, routes::sign_up, routes::login, routes::login_page])
        .attach(Template::fairing())
}