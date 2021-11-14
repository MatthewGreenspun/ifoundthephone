use super::auth;
use hex::ToHex;
use rand::{distributions::Alphanumeric, Rng};
use tokio_postgres::{Error, NoTls};

fn gen_user_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

pub async fn save_user(email: &String, password: &String) -> Result<(), Error> {
    let postgres_uri = std::env::var("DB_URI").expect("environment variable not found");
    let (client, connection) = tokio_postgres::connect(&postgres_uri, NoTls).await?;

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

    client
        .execute(
            "INSERT INTO users (user_id, email, pw_hash, pw_salt) VALUES ($1, $2, $3, $4)",
            &[&id, email, hash_str, salt_str],
        )
        .await?;
    Ok(())
}

pub async fn get_email(id: &String) -> Result<String, Error> {
    let postgres_uri = std::env::var("DB_URI").expect("environment variable not found");
    let (client, connection) = tokio_postgres::connect(&postgres_uri, NoTls).await?;

    rocket::tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT email FROM users WHERE user_id = $1", &[id])
        .await?;
    let email: &str = rows[0].get("email");
    Ok(email.to_string())
}
