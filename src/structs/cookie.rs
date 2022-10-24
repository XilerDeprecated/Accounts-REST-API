use super::user_agent::{UserAgentExtension, UserAgentPlatform};

// s1.2426094911.1768803836-3566326825-3912814204||3967086928-2746952293-707505781.3788223220_59786466.Jmtl9LJ3bAYkoymfnCdHCjYjE00hJhdJ
pub struct ParsedCookie {
    pub prefix: String,
    pub ip: String,
    pub platforms: Vec<UserAgentPlatform>,
    pub extensions: Vec<UserAgentExtension>,
    pub random: String,
}
