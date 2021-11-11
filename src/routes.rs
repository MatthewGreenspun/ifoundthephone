use serde_json::json;
use rocket::form::{FromForm, Form};
use rocket_dyn_templates::Template;

#[derive(FromForm)]
pub struct NewUserRequest {
	pub email: String,
	pub password: String,
	#[field(name = "confirm-password")]	
	pub confirm_password: String
}

#[derive(FromForm)]
pub struct ReturningUserRequest {
	pub email: String,
	pub password: String,
}

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
pub fn sign_up(new_user: Form<NewUserRequest>) -> Template {
    if new_user.email.len() > 0 {
        if new_user.password.len() > 0 {
            if new_user.password == new_user.confirm_password {
                let context = json!({"isSignedIn": true, "email": new_user.email.clone()});
                Template::render("profile", &context)
            } else {
                let context = json!({
                    "email": new_user.email.clone(),
                    "password": new_user.password.clone(),
                    "confirmPasswordError": "passwords don't match"
                });
                Template::render("signup", &context)
            }
        } else {
            let context = json!({
                "email": new_user.email.clone(),
                "passwordError": "password is required",
            });
            Template::render("signup", &context)
        }
    } else {
        let context = json!({
            "emailError": "email is required",
            "password": new_user.password.clone()
        });
        Template::render("signup", &context)
    }
}

#[get("/login")]
pub fn login_page() -> Template {
    let context = json!({});
    Template::render("login", &context)
}

#[post("/login", data = "<user>")]
pub fn login(user: Form<ReturningUserRequest>) -> Template {
    if user.email.len() > 0 {
        if user.password.len() > 0 {
            //TODO add actual validation for password
            let context = json!({"isSignedIn": true, "email": user.email.clone()});
            Template::render("profile", &context)
        } else {
            let context = json!({
                "email": user.email.clone(),
                "passwordError": "password is required",
            });
            Template::render("login", &context)
        }
    } else {
        let context = json!({
            "emailError": "email is required",
            "password": user.password.clone()
        });
        Template::render("login", &context)
    }
}