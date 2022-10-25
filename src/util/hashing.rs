use argon2::{hash_encoded, Config, ThreadMode, Variant, Version};
use std::hash::Hasher;
use twox_hash::XxHash32;

pub fn xx_hash(data: &str) -> String {
    let mut hasher = XxHash32::with_seed(0);
    hasher.write(data.as_bytes());
    hasher.finish().to_string()
}

static ARGON2_CONFIG: Config = Config {
    ad: &[],
    hash_length: 128,
    lanes: 1,
    mem_cost: 32,
    secret: &[],
    thread_mode: ThreadMode::Parallel,
    time_cost: 3,
    variant: Variant::Argon2i,
    version: Version::Version13,
};

pub fn argon2_hash(data: &str) -> String {
    // TODO: Get salt from env
    let salt = b"ajsldAJKHDLAKJDjsna/AZ";
    hash_encoded(data.as_bytes(), salt, &ARGON2_CONFIG).unwrap()
}
