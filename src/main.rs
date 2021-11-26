#[macro_use]
extern crate rocket;
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
mod routes;
use routes::db::DbClient;
use tokio_postgres::NoTls;

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().expect("failed to load environment variables");

    let postgres_uri = std::env::var("DB_URI").expect("environment variable 'DB_URI' not found");
    let (client, connection) = tokio_postgres::connect(&postgres_uri, NoTls)
        .await
        .expect("failed to connnect to database");
    rocket::tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("database connection error: {}", e);
        }
    });

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
                routes::login_page_authenticated,
                routes::device_found_page,
                routes::device_found,
                routes::profile_page,
                routes::profile_page_failure,
                routes::logout
            ]
        )
        .register("/", catchers![routes::unauthorized])
        .manage(DbClient { client })
        .attach(Template::fairing())
}
