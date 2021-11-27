use super::db::{email_exists, get_hash_and_salt};
use hex::{FromHex, ToHex};
use rand::{distributions::Alphanumeric, Rng};
use ring::digest::SHA512_OUTPUT_LEN;
use ring::{pbkdf2, rand::SecureRandom};
use rocket::http::{Cookie, CookieJar};
use std::num::NonZeroU32;
use time::{Duration, OffsetDateTime};
use tokio_postgres::Client;

pub fn gen_session_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .map(char::from)
        .collect()
}

pub fn gen_salt() -> [u8; SHA512_OUTPUT_LEN] {
    let mut salt = [0u8; SHA512_OUTPUT_LEN];
    let rng = ring::rand::SystemRandom::new();
    rng.fill(&mut salt).expect("failed to generate salt");
    salt
}

pub fn gen_hash(salt: &[u8], password: &String) -> [u8; SHA512_OUTPUT_LEN] {
    let mut hash = [0u8; SHA512_OUTPUT_LEN];
    let n_iter = NonZeroU32::new(100_000).unwrap();
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        salt,
        password.as_bytes(),
        &mut hash,
    );
    hash
}

pub async fn user_is_valid(client: &Client, email: &String, password: &String) -> bool {
    if !email_exists(&client, email).await {
        return false;
    }

    let (true_hash, salt) = match get_hash_and_salt(&client, email).await {
        Ok(hash_and_salt) => hash_and_salt,
        Err(e) => {
            eprintln!("error fetching hash and salt: {:?}", e);
            return false;
        }
    };
    let salt_buf = <[u8; SHA512_OUTPUT_LEN]>::from_hex(salt).expect("decoding salt failed");
    let hash = gen_hash(&salt_buf, password).encode_hex::<String>();

    true_hash == hash
}

pub fn set_session_cookie(cookies: &CookieJar<'_>, session_id: String, max_age: Duration) {
    let expiration_date = OffsetDateTime::now_utc() + max_age;
    let cookie = Cookie::build("session_id", session_id.clone())
        .expires(expiration_date)
        .finish();
    cookies.add(cookie);
}
