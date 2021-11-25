#[macro_use]
extern crate rocket;
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
mod routes;

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().expect("failed to load environment variables");
    rocket::build()
        .mount("/static", FileServer::from(relative!("/static")))
        .mount(
            "/",
            routes![
                routes::index,
                routes::sign_up,
                routes::sign_up_page,
                routes::login,
                routes::login_page,
                routes::device_found_page,
                routes::device_found,
                routes::profile_page
            ],
        )
        .attach(Template::fairing())
}
