use super::auth;
use chrono::naive::NaiveDate;
use hex::ToHex;
use rand::{distributions::Alphanumeric, Rng};
use time::OffsetDateTime;
use tokio_postgres::{Client, Error};

pub enum AuthError {
    DbError(Error),
    SessionInvalidError,
}

pub struct DbClient {
    pub client: Client,
}

pub fn gen_user_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

pub async fn save_user(
    client: &Client,
    user_id: &String,
    email: &String,
    password: &String,
) -> Result<(), Error> {
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

pub async fn save_session(
    client: &Client,
    session_id: &String,
    user_id: &String,
    expiration_date_offset: OffsetDateTime,
) -> Result<(), Error> {
    let expiration_date = NaiveDate::from_ymd(
        expiration_date_offset.year(),
        expiration_date_offset.month().into(),
        expiration_date_offset.day().into(),
    );
    client
        .execute(
            "INSERT INTO sessions (session_id, user_id, expires) VALUES ($1, $2, $3)",
            &[session_id, user_id, &expiration_date],
        )
        .await?;
    Ok(())
}

pub async fn get_email(client: &Client, id: &String) -> Result<String, Error> {
    let rows = client
        .query("SELECT email FROM users WHERE user_id = $1", &[id])
        .await?;
    let email: &str = rows[0].get("email");
    Ok(email.to_string())
}

pub async fn get_id(client: &Client, email: &String) -> Result<String, Error> {
    let rows = client
        .query("SELECT user_id FROM users WHERE email = $1", &[email])
        .await?;
    let email: &str = rows[0].get(0);
    Ok(email.to_string())
}

pub async fn id_exists(client: &Client, id: &String) -> bool {
    let rows = match client
        .query("SELECT COUNT(*) FROM users WHERE user_id = $1", &[id])
        .await
    {
        Ok(rows) => rows,
        Err(_) => return false,
    };

    let count: i64 = rows[0].get(0);
    if count == 0 {
        false
    } else {
        true
    }
}

pub async fn email_exists(client: &Client, email: &String) -> bool {
    let rows = match client
        .query("SELECT COUNT(*) FROM users WHERE email = $1", &[email])
        .await
    {
        Ok(rows) => rows,
        Err(_) => return false,
    };

    let count: i64 = rows[0].get(0);
    if count == 0 {
        false
    } else {
        true
    }
}

pub async fn get_hash_and_salt(client: &Client, email: &String) -> Result<(String, String), Error> {
    let rows = client
        .query(
            "SELECT pw_hash, pw_salt FROM users WHERE email = $1",
            &[email],
        )
        .await?;
    let hash: String = rows[0].get("pw_hash");
    let salt: String = rows[0].get("pw_salt");
    Ok((hash, salt))
}

pub async fn get_session_user(client: &Client, session_id: &String) -> Result<String, AuthError> {
    let _ = client
        .execute("DELETE FROM sessions WHERE NOW() > expires", &[])
        .await;
    let rows = match client
        .query(
            "SELECT user_id FROM sessions WHERE session_id = $1",
            &[session_id],
        )
        .await
    {
        Ok(rows) => {
            if rows.len() == 0 {
                return Err(AuthError::SessionInvalidError);
            }
            rows
        }
        Err(e) => return Err(AuthError::DbError(e)),
    };

    let user_id: String = rows[0].get(0);
    Ok(user_id)
}

pub async fn terminate_session(client: &Client, session_id: &String) -> Result<(), Error> {
    client
        .execute("DELETE FROM sessions WHERE session_id = $1", &[session_id])
        .await?;
    Ok(())
}
