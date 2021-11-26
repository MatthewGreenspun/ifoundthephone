use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
    State,
};
use rocket_dyn_templates::Template;
use serde_json::json;
use time::{Duration, OffsetDateTime};
mod email;
mod route_structs;
use route_structs::*;
mod auth;
pub mod db;
use db::DbClient;

#[get("/")]
pub fn index() -> Template {
    let context = json!({"isSignedIn": false});
    Template::render("index", &context)
}

#[catch(401)]
pub fn unauthorized() -> Redirect {
    Redirect::to(uri!(login_page))
}

#[get("/signup")]
pub fn sign_up_page() -> Template {
    let context = json!({});
    Template::render("signup", &context)
}

#[post("/signup", data = "<new_user>")]
pub async fn sign_up(
    new_user: Form<NewUserRequest>,
    cookies: &CookieJar<'_>,
    db_client: &State<DbClient>,
) -> Result<Redirect, Template> {
    if new_user.email.len() <= 0 {
        let context = json!({
            "emailError": "email is required",
            "password": &new_user.password
        });
        return Err(Template::render("signup", &context));
    }
    if new_user.password.len() <= 0 {
        let context = json!({
            "email": &new_user.email,
            "passwordError": "password is required",
        });
        return Err(Template::render("signup", &context));
    }
    if new_user.password == new_user.confirm_password {
        let user_id = db::gen_user_id();
        let _ = db::save_user(
            &db_client.client,
            &user_id,
            &new_user.email,
            &new_user.password,
        )
        .await;
        email::email_new_user(&user_id, &new_user.email);
        let session_id = auth::gen_session_id();
        auth::set_session_cookie(&cookies, session_id.clone(), Duration::days(1));
        match db::save_session(
            &db_client.client,
            &session_id,
            &user_id,
            OffsetDateTime::now_utc() + Duration::days(1)
        )
        .await
        {
            Ok(()) => (),
            Err(e) => eprintln!("error saving session: {:?}", e),
        };
        return Ok(Redirect::to(uri!(profile_page(user_id))));
    }
    let context = json!({
        "email": &new_user.email,
        "password": &new_user.password,
        "confirmPasswordError": "passwords don't match"
    });
    Err(Template::render("signup", &context))
}
#[get("/login", rank = 1)]
pub fn login_page_authenticated(user: AuthenticatedUser) -> Redirect {
    Redirect::to(uri!(profile_page(user.user_id)))
}

#[get("/login", rank = 2)]
pub fn login_page() -> Template {
    let context = json!({});
    Template::render("login", &context)
}

#[post("/login", data = "<user>")]
pub async fn login(
    user: Form<ReturningUserRequest>,
    cookies: &CookieJar<'_>,
    db_client: &State<DbClient>,
) -> Result<Redirect, Template> {
    if user.email.len() <= 0 {
        let context = json!({
            "emailError": "email is required",
            "password": &user.password
        });
        return Err(Template::render("login", &context));
    }
    if user.password.len() <= 0 {
        let context = json!({
            "email": &user.email,
            "passwordError": "password is required",
        });
        return Err(Template::render("login", &context));
    }
    if !auth::user_is_valid(&db_client.client, &user.email, &user.password).await {
        let context =
            json!({"email": &user.email,"passwordError": "email or password is incorrect"});
        return Err(Template::render("login", &context));
    }
    let user_id = db::get_id(&db_client.client, &user.email).await;
    match user_id {
        Ok(id) => {
            let session_id = auth::gen_session_id();
            auth::set_session_cookie(&cookies, session_id.clone(), Duration::days(1));
            match db::save_session(&db_client.client, &session_id, &id, OffsetDateTime::now_utc() + Duration::days(1))
                .await
            {
                Ok(()) => (),
                Err(e) => eprintln!("error saving session: {:?}", e),
            };
            return Ok(Redirect::to(uri!(profile_page(id))));
        }
        Err(e) => {
            eprint!("error getting user id: {:?}", e);
            let context =
                json!({"email": &user.email,"passwordError": "email or password is incorrect"});
            return Err(Template::render("login", &context));
        }
    };
}

#[get("/device/<id>")]
pub async fn device_found_page(id: String, db_client: &State<DbClient>) -> Template {
    if !db::id_exists(&db_client.client, &id).await {
        let context = json!({ "idError": format!("id {} does not exist", id) });
        return Template::render("index", &context);
    }
    let context = json!({ "deviceId": &id });
    Template::render("device_found", &context)
}

#[post("/device/<id>", data = "<email_info>")]
pub async fn device_found(
    id: String,
    email_info: Form<DeviceFoundRequest>,
    db_client: &State<DbClient>,
) -> Template {
    if email_info.message.len() > 0 {
        let owner_email = match db::get_email(&db_client.client, &id).await {
            Ok(email) => email,
            Err(e) => {
                eprintln!("failed to retrieve email. Error: {:?}", e);
                String::new()
            }
        };
        if owner_email.len() == 0 {
            let context = json!({"deviceId": &id, "error": "failed to send email"});
            return Template::render("device_found", &context);
        }
        email::email_device_owner(&owner_email, email_info.into_inner());
        let context = json!({});
        return Template::render("index", &context);
    }
    let context = json!({"deviceId": &id, "messageError": "message is required"});
    return Template::render("device_found", &context);
}

#[get("/profile/<id>", rank = 1)]
pub async fn profile_page(
    id: String,
    user: AuthenticatedUser
) -> Result<Template, Redirect> {
    if user.user_id != id {
        return Err(Redirect::to(uri!(login_page())));
    }
    let context = json!({"email": user.email, "isSignedIn": true, "userId": &id});
    Ok(Template::render("profile", &context))
}
#[get("/profile/<_id>", rank = 2)]
pub async fn profile_page_failure(_id: String) -> Redirect {
    Redirect::to(uri!(login_page()))
}
#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>, db_client: &State<DbClient>) -> Redirect {
    let session_id = match cookies.get("session_id") {
        Some(session_id) => session_id.value(),
        None => return Redirect::to(uri!(index())),
    };
    cookies.remove(Cookie::named("session_id"));
    match db::terminate_session(&db_client.client, &session_id.to_string()).await {
        Ok(()) => (),
        Err(e) => eprintln!(
            "failed to terminate session with id {}. error: {:?}",
            &session_id, e
        ),
    };

    Redirect::to(uri!(index()))
}
