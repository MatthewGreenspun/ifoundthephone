use rocket::form::FromForm;

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
