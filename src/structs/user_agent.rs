pub type UserAgentExtension = String;

pub struct UserAgentPlatform {
    pub name: String,
    pub version: String,
    pub details: String,
}

pub struct ParsedUserAgent {
    pub platforms: Vec<UserAgentPlatform>,
    pub extensions: Vec<UserAgentExtension>,
}
