use rand::{distributions::Alphanumeric, Rng};
use tokio_postgres::{NoTls, Error};
use super::auth;
use hex::ToHex;

fn gen_user_id() -> String {
	rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(7)
		.map(char::from)
		.collect()
}

pub async fn save_user(email: &String, password: &String) -> Result<(), Error> {
	let postgres_uri = std::env::var("DB_URI").expect("environment variable not found");
	let (client, connection) =
			tokio_postgres::connect(&postgres_uri, NoTls).await?;

	// The connection object performs the actual communication with the database,
	// so spawn it off to run on its own.
	rocket::tokio::spawn(async move {
			if let Err(e) = connection.await {
					eprintln!("connection error: {}", e);
			}
	});

	let id: String = gen_user_id();
	let salt = auth::gen_salt();
	let hash = auth::gen_hash(&salt, password);
	let salt_str = &salt.encode_hex::<String>();
	let hash_str = &hash.encode_hex::<String>();

	client.execute("INSERT INTO users (user_id, email, pw_hash, pw_salt) VALUES ($1, $2, $3, $4)",&[&id, email, hash_str, salt_str]).await?;
	Ok(())
}