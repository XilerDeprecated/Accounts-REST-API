use crate::structs::{
    cookie::ParsedCookie,
    user_agent::{ParsedUserAgent, UserAgentPlatform},
};

fn parse_platform(platform: &str) -> UserAgentPlatform {
    let mut name: String = "".to_string();
    let mut version: String = "".to_string();
    let mut details: String = "".to_string();

    let mut is_parsing_platform = true;
    let mut is_parsing_version = false;
    let mut is_parsing_details = false;

    for c in platform.chars() {
        if is_parsing_platform {
            if c == '/' {
                is_parsing_platform = false;
                is_parsing_version = true;
            } else {
                name.push(c);
            }
        } else if is_parsing_version {
            if c == ' ' {
                is_parsing_version = false;
                is_parsing_details = true;
            } else {
                version.push(c);
            }
        } else if is_parsing_details {
            details.push(c);
        }
    }

    UserAgentPlatform {
        name,
        version,
        details,
    }
}

fn parse_extensions(platforms: &mut Vec<&str>) -> Vec<String> {
    match !platforms.last().unwrap().ends_with(")") {
        true => platforms
            .pop()
            .unwrap()
            .split(' ')
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        false => vec![],
    }
}

pub fn parse_user_agent(user_agent: String) -> ParsedUserAgent {
    let mut platforms: Vec<&str> = user_agent.split_inclusive(')').collect();

    if platforms.len() == 0 {
        return ParsedUserAgent {
            platforms: vec![],
            extensions: vec![],
        };
    }

    let extensions = parse_extensions(&mut platforms);
    let platforms = platforms.iter().map(|p| parse_platform(p)).collect();

    ParsedUserAgent {
        platforms,
        extensions,
    }
}

// s1.2426094911.1768803836-3566326825-3912814204||3967086928-2746952293-707505781.3788223220_59786466.Jmtl9LJ3bAYkoymfnCdHCjYjE00hJhdJ
// TODO: Parse expected (hashed) values from cookie
pub fn parse_browser_cookie(cookie: &str) -> Result<ParsedCookie, String> {
    let splitted: Vec<&str> = cookie.split(".").collect();
    if splitted.len() != 5 {
        return Err("Invalid cookie".to_string());
    }

    let prefix = splitted[0].to_string();
    let ip = splitted[1].to_string();
    let platforms = splitted[2]
        .split("||")
        .filter_map(|platform| {
            let platform_details = platform.split("-").collect::<Vec<&str>>();

            if platform_details.len() != 3 {
                return None;
            }

            Some(UserAgentPlatform {
                name: platform_details[0].to_string(),
                version: platform_details[1].to_string(),
                details: platform_details[2].to_string(),
            })
        })
        .collect();

    let extensions = splitted[3].split("_").map(|s| s.to_string()).collect();

    Ok(ParsedCookie {
        prefix,
        ip,
        platforms,
        extensions,
        random: splitted[4].to_string(),
    })
}
