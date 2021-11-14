use ring::digest::SHA512_OUTPUT_LEN;
use ring::{pbkdf2, rand::SecureRandom};
use std::num::NonZeroU32;

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
