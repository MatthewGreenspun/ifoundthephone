use super::auth;
use hex::ToHex;
use rand::{distributions::Alphanumeric, Rng};
use tokio_postgres::{Error, NoTls};

pub fn gen_user_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

pub async fn save_user(user_id: &String, email: &String, password: &String) -> Result<(), Error> {
    let postgres_uri = std::env::var("DB_URI").expect("environment variable not found");
    let (client, connection) = tokio_postgres::connect(&postgres_uri, NoTls).await?;

    rocket::tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let salt = auth::gen_salt();
    let hash = auth::gen_hash(&salt, password);
    let salt_str = &salt.encode_hex::<String>();
    let hash_str = &hash.encode_hex::<String>();

    client
        .execute(
            "INSERT INTO users (user_id, email, pw_hash, pw_salt) VALUES ($1, $2, $3, $4)",
            &[&user_id, email, hash_str, salt_str],
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

pub async fn get_id(email: &String) -> Result<String, Error> {
    let postgres_uri = std::env::var("DB_URI").expect("environment variable not found");
    let (client, connection) = tokio_postgres::connect(&postgres_uri, NoTls).await?;

    rocket::tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT user_id FROM users WHERE email = $1", &[email])
        .await?;
    let email: &str = rows[0].get(0);
    Ok(email.to_string())
}

pub async fn id_exists(id: &String) -> bool {
    let postgres_uri = std::env::var("DB_URI").expect("environment variable not found");
    let (client, connection) = tokio_postgres::connect(&postgres_uri, NoTls).await.expect("failed to connect to database");

    rocket::tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = match client
        .query("SELECT COUNT(*) FROM users WHERE user_id = $1", &[id])
        .await {
            Ok(rows) => rows,
            Err(_) => return false
        };

    let count: i64 = rows[0].get(0);
    if count == 0 {false} else {true}
}

pub async fn email_exists(email: &String) -> bool {
    let postgres_uri = std::env::var("DB_URI").expect("environment variable not found");
    let (client, connection) = tokio_postgres::connect(&postgres_uri, NoTls).await.expect("failed to connect to database");

    rocket::tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = match client
        .query("SELECT COUNT(*) FROM users WHERE email = $1", &[email])
        .await {
            Ok(rows) => rows, 
            Err(_) => return false
        };

    let count: i64 = rows[0].get(0);
    if count == 0 {false} else {true}
}

pub async fn get_hash_and_salt(email: &String) -> Result<(String, String), Error> {
    let postgres_uri = std::env::var("DB_URI").expect("environment variable not found");
    let (client, connection) = tokio_postgres::connect(&postgres_uri, NoTls).await?;

    rocket::tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT pw_hash, pw_salt FROM users WHERE email = $1", &[email])
        .await?;
    let hash: String = rows[0].get("pw_hash");
    let salt: String = rows[0].get("pw_salt");
    Ok((hash, salt))
}