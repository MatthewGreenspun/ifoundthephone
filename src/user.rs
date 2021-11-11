use rocket::serde::Serialize;

#[derive(Serialize)]
pub struct User {
	pub is_signed_in: bool,
	pub user_name: String,
}
