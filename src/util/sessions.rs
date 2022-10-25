use actix_web::HttpRequest;

use crate::{
    errors::HttpError,
    structs::{user_agent::ParsedUserAgent, Status},
};

use super::{hashing::xx_hash, parse::parse_user_agent, random::random_string};

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

pub fn create_browser_session(data: HttpRequest) -> Result<String, HttpError> {
    let user_agent = match data.headers().get("User-Agent") {
        Some(agent) => agent,
        None => {
            return Err(HttpError::BadRequest(Status {
                message: "No user agent present".to_string(),
            }))
        }
    };
    let ip = data.peer_addr().unwrap().ip().to_string();
    let parsed_user_agent = parse_user_agent(user_agent.to_str().unwrap().to_string());

    Ok(generate_browser_session(ip, parsed_user_agent))
}
