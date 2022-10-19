use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn random_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn xx_hash128(data: String) -> String {
    // TODO: Implement xxHash128
    data
}

pub fn generate_browser_session(user_agent: String) -> String {
    let random = random_string(63);
    let hashed_user_agent = xx_hash128(user_agent);
    format!("s1.{}.{}", hashed_user_agent, random)
}
