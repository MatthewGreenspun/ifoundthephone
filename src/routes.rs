use rocket::{form::Form, response::Redirect};
use rocket_dyn_templates::Template;
use serde_json::json;
mod email;
mod form_structs;
use form_structs::*;
mod auth;
mod db;

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
pub async fn sign_up(new_user: Form<NewUserRequest>) -> Result<Redirect, Template> {
    if new_user.email.len() > 0 {
        if new_user.password.len() > 0 {
            if new_user.password == new_user.confirm_password {
                let user_id = db::gen_user_id();
                let _ = db::save_user(&user_id, &new_user.email, &new_user.password).await;
                email::email_new_user(&user_id, &new_user.email);
                return Ok(Redirect::to(uri!(profile_page(user_id))));
            }
            let context = json!({
                "email": &new_user.email,
                "password": &new_user.password,
                "confirmPasswordError": "passwords don't match"
            });
            return Err(Template::render("signup", &context));
        }
        let context = json!({
            "email": &new_user.email,
            "passwordError": "password is required",
        });
        return Err(Template::render("signup", &context));
    }
    let context = json!({
        "emailError": "email is required",
        "password": &new_user.password
    });
    Err(Template::render("signup", &context))
}

#[get("/login")]
pub fn login_page() -> Template {
    let context = json!({});
    Template::render("login", &context)
}

#[post("/login", data = "<user>")]
pub async fn login(user: Form<ReturningUserRequest>) -> Result<Redirect, Template> {
    if user.email.len() > 0 {
        if user.password.len() > 0 {
            if auth::user_is_valid(&user.email, &user.password).await {
                let user_id = db::get_id(&user.email).await;
                match user_id {
                    Ok(id) => return Ok(Redirect::to(uri!(profile_page(id)))),
                    Err(e) => eprint!("error getting user id: {:?}", e),
                };
            }
            let context = json!({"email": &user.email,"passwordError": "email or password is incorrect"});
            return Err(Template::render("login", &context))
        }
        let context = json!({
            "email": &user.email,
            "passwordError": "password is required",
        });
        return Err(Template::render("login", &context))
    }
    let context = json!({
        "emailError": "email is required",
        "password": &user.password
    });
    Err(Template::render("login", &context))
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
pub async fn profile_page(id: String) -> Template {
    let email = match db::get_email(&id).await {
        Ok(email) => email,
        Err(err) => {
            eprintln!("error getting email from id of {}.\n error: {:?}", &id, err);
            let context = json!({});
            return Template::render("index", &context)
        }
    };
    let context = json!({"email": email, "isSignedIn": true, "userId": &id});
    Template::render("profile", &context)
}
