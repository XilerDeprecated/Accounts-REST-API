use argon2::{hash_encoded, Config};
use std::hash::Hasher;
use twox_hash::XxHash32;

pub fn xx_hash(data: &str) -> String {
    let mut hasher = XxHash32::with_seed(0);
    hasher.write(data.as_bytes());
    hasher.finish().to_string()
}

pub fn argon2_hash(data: &str) -> String {
    // TODO: Get salt from env
    let salt = b"some salt";
    hash_encoded(data.as_bytes(), salt, &Config::default()).unwrap()
}
