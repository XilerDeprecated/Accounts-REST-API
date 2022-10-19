use std::hash::Hasher;
use twox_hash::XxHash64;

pub fn xx_hash128(data: String) -> String {
    let mut hasher = XxHash64::with_seed(0);
    hasher.write(data.as_bytes());
    hasher.finish().to_string()
}
