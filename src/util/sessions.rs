use crate::structs::user_agent::ParsedUserAgent;

use super::{hashing::xx_hash, random::random_string};

pub fn generate_browser_session(ip: String, user_agent: ParsedUserAgent) -> String {
    let random = random_string(32);
    let hashed_ip = xx_hash(&ip);
    let hashed_user_agent_platforms = user_agent
        .platforms
        .iter()
        .map(|platform| {
            format!(
                "{}-{}-{}",
                xx_hash(&platform.name.clone()),
                xx_hash(&platform.version.clone()),
                xx_hash(&platform.details.clone())
            )
        })
        .collect::<Vec<String>>();

    let hashed_user_agent_extensions = user_agent
        .extensions
        .iter()
        .map(|extension| xx_hash(&extension.to_string()))
        .collect::<Vec<String>>();

    vec![
        "s1".to_string(),
        hashed_ip,
        hashed_user_agent_platforms.join("||"),
        hashed_user_agent_extensions.join("_"),
        random,
    ]
    .join(".")
}
