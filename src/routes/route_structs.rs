use rocket::form::FromForm;
use rocket::request::{ Request, FromRequest, Outcome};
use rocket::http::{Status};
use super::db;

#[derive(FromForm)]
pub struct NewUserRequest {
    pub email: String,
    pub password: String,
    #[field(name = "confirm-password")]
    pub confirm_password: String,
}

#[derive(FromForm)]
pub struct ReturningUserRequest {
    pub email: String,
    pub password: String,
}

#[derive(FromForm)]
pub struct DeviceFoundRequest {
    pub message: String,
    pub email: String,
    #[field(name = "phone-number")]
    pub phone_number: String,
}

pub struct AuthenticatedUser {
    pub user_id: String, 
    pub session_id: String, 
    pub email: String,
}

#[derive(Debug)]
pub enum AuthError {
    DbClientNotFound,
    DbQueryFailure
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, AuthError> {
        let session_id = match req.cookies().get("session_id") {
            Some(cookie) => cookie.value().to_string(),
            None => return Outcome::Forward(())
        };
        let db_client = match req.rocket().state::<db::DbClient>(){
            Some(client) => client,
            None => return Outcome::Failure((Status::InternalServerError, AuthError::DbClientNotFound))
        };
        let user_id = match db::get_session_user(&db_client.client, &session_id).await {
            Ok(user_id) => user_id,
            Err(e) => match e {
                db::AuthError::DbError(e) => {
                    eprintln!("error retrieving user id from database: {}", e);
                    return Outcome::Failure((Status::InternalServerError, AuthError::DbQueryFailure))
                },
                db::AuthError::SessionInvalidError => return Outcome::Forward(())
            }
        };
        let email = match db::get_email(&db_client.client, &user_id).await {
            Ok(email) => email,
            Err(e) => {
                eprintln!("error retrieving email from database: {}", e);
                return Outcome::Failure((Status::InternalServerError, AuthError::DbQueryFailure))
            }
        };

        Outcome::Success(AuthenticatedUser {user_id, session_id, email})
    }
}