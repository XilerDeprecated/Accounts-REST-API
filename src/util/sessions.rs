use super::{hashing::xx_hash128, random::random_string};

pub fn generate_browser_session(user_agent: String) -> String {
    let random = random_string(63);
    let hashed_user_agent = xx_hash128(user_agent);
    format!("s1.{}.{}", hashed_user_agent, random)
}
