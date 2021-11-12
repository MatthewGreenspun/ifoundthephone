#[macro_use]
extern crate rocket;
use rocket_dyn_templates::Template;
use rocket::fs::{FileServer, relative};
mod routes;
use routes::*;

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().expect("failed to load environment variables");
    rocket::build()
        .mount("/static", FileServer::from(relative!("/static")))
        .mount("/", routes![index, sign_up, sign_up_page, login, login_page, device_found_page])
        .attach(Template::fairing())
}