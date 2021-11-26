use rocket::{form::Form, http::{CookieJar, Cookie}, response::Redirect};
use time::{OffsetDateTime, Duration};
use chrono::NaiveDate;
use rocket_dyn_templates::Template;
use serde_json::json;
mod email;
mod route_structs;
use route_structs::*;
mod auth;
mod db;
use db::AuthError;

#[get("/")]
pub fn index() -> Template {
    let context = json!({"isSignedIn": false});
    Template::render("index", &context)
}

#[get("/signup")]
pub fn sign_up_page() -> Template {
    let context = json!({});
    Template::render("signup", &context)
}

#[post("/signup", data = "<new_user>")]
pub async fn sign_up(new_user: Form<NewUserRequest>, cookies: &CookieJar<'_>) -> Result<Redirect, Template> {
    if new_user.email.len() <= 0 {
        let context = json!({
            "emailError": "email is required",
            "password": &new_user.password
        });
        return Err(Template::render("signup", &context))
    }
    if new_user.password.len() <= 0 {
        let context = json!({
            "email": &new_user.email,
            "passwordError": "password is required",
        });
        return Err(Template::render("signup", &context))
    }
    if new_user.password == new_user.confirm_password {
        let user_id = db::gen_user_id();
        let _ = db::save_user(&user_id, &new_user.email, &new_user.password).await;
        email::email_new_user(&user_id, &new_user.email);
        let session_id = auth::gen_session_id();
        let expiration_date = OffsetDateTime::now_utc() + Duration::days(1);
        let chrono_expiration_date = NaiveDate::from_ymd(expiration_date.year(), expiration_date.month().into(), expiration_date.day().into());
        let cookie = Cookie::build("session_id", session_id.clone())
            .expires(expiration_date)
            .finish();
        cookies.add(cookie);
        match db::save_session(&session_id,&user_id, chrono_expiration_date).await {
            Ok(()) => println!("saved session. id: {}, expiration date: {}", &user_id, &expiration_date),
            Err(e) => eprintln!("error saving session: {:?}", e)
        };
        return Ok(Redirect::to(uri!(profile_page(user_id))))
    }
    let context = json!({
        "email": &new_user.email,
        "password": &new_user.password,
        "confirmPasswordError": "passwords don't match"
    });
    Err(Template::render("signup", &context))
}

#[get("/login")]
pub fn login_page() -> Template {
    let context = json!({});
    Template::render("login", &context)
}

#[post("/login", data = "<user>")]
pub async fn login(user: Form<ReturningUserRequest>, cookies: &CookieJar<'_>) -> Result<Redirect, Template> {
    if user.email.len() <= 0 {
        let context = json!({
            "emailError": "email is required",
            "password": &user.password
        });
        return Err(Template::render("login", &context))
    }
    if user.password.len() <= 0 {
        let context = json!({
            "email": &user.email,
            "passwordError": "password is required",
        });
        return Err(Template::render("login", &context))
    }
    if !auth::user_is_valid(&user.email, &user.password).await {
        let context = json!({"email": &user.email,"passwordError": "email or password is incorrect"});
        return Err(Template::render("login", &context))
    }
    let user_id = db::get_id(&user.email).await;
    match user_id {
        Ok(id) => {
            let session_id = auth::gen_session_id();
            let expiration_date = OffsetDateTime::now_utc() + Duration::days(1);
            let chrono_expiration_date = NaiveDate::from_ymd(expiration_date.year(), expiration_date.month().into(), expiration_date.day().into());
            let cookie = Cookie::build("session_id", session_id.clone())
                .expires(expiration_date)
                .finish();
            cookies.add(cookie);
            match db::save_session(&session_id,&id, chrono_expiration_date).await {
                Ok(()) => println!("saved session. id: {}, expiration date: {}", &id, &expiration_date),
                Err(e) => eprintln!("error saving session: {:?}", e)
            };
            return Ok(Redirect::to(uri!(profile_page(id))))
        },
        Err(e) => { 
            eprint!("error getting user id: {:?}", e);
            let context = json!({"email": &user.email,"passwordError": "email or password is incorrect"});
            return Err(Template::render("login", &context))
        }
    };
}

#[get("/device/<id>")]
pub async fn device_found_page(id: String) -> Template {
    if !db::id_exists(&id).await {
        let context = json!({"idError": format!("id {} does not exist", id)});
        return Template::render("index", &context);
    }
    let context = json!({ "deviceId": &id });
    Template::render("device_found", &context)
}

#[post("/device/<id>", data = "<email_info>")]
pub async fn device_found(id: String, email_info: Form<DeviceFoundRequest>) -> Template {
    if email_info.message.len() > 0 {
        let owner_email = match db::get_email(&id).await {
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

#[get("/profile/<id>")]
pub async fn profile_page(id: String, cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    if !db::id_exists(&id).await { return Err(Redirect::to(uri!(login_page()))) }
    let session_id = match cookies.get("session_id") {
        Some(session_id) => session_id,
        None => return Err(Redirect::to(uri!(login_page())))
    };
    let session_user_id = match db::get_session_user(&session_id.value().to_string()).await { 
        Ok(user_id) => {
            if user_id != id {return Err(Redirect::to(uri!(login_page())))}
            user_id
        },
        Err(err) => {
            match err {
                AuthError::DbError(e) => eprintln!("error retriving session: {:?}", e),
                _ => ()
            };
            return Err(Redirect::to(uri!(login_page()))) 
        }
    };

    let email = match db::get_email(&session_user_id).await {
        Ok(email) => email,
        Err(err) => {
            eprintln!("error getting email from id of {}.\n error: {:?}", &session_user_id, err);
            return Err(Redirect::to(uri!(login_page())))
        }
    };

    let context = json!({"email": email, "isSignedIn": true, "userId": &id});
    Ok(Template::render("profile", &context))
}
 
#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    let session_id = match cookies.get("session_id") {
        Some(session_id) => session_id.value(),
        None => return Redirect::to(uri!(index()))
    };
    cookies.remove(Cookie::named("session_id"));
    match db::terminate_session(&session_id.to_string()).await {
        Ok(()) => (),
        Err(e) => eprintln!("failed to terminate session with id {}. error: {:?}", &session_id, e)
    };

    Redirect::to(uri!(index()))
}