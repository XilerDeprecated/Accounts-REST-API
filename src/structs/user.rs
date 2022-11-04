// Represents a user in memory

use std::collections::HashMap;

use chrono::Duration;
use paperclip::actix::{Apiv2Schema, Apiv2Security};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserAuthenticationMap = HashMap<i16, String>;

#[derive(Serialize, Apiv2Schema)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: usize,
    /// A int that contains the user roles, can be parsed by using bitwize operations.
    pub roles: usize,
    /// An int that contains the linked platforms, can be parsed by using bitwize operations.
    pub authentication: i16,
    pub verified: bool,
}

/// Contains the minimum data for a user to register.
#[derive(Deserialize, Apiv2Schema)]
pub struct UserRegistration {
    pub username: String,
    pub email: String,
    /// A hashed version of the password.
    pub password: String,
}

#[derive(Deserialize, Apiv2Schema)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Apiv2Security)]
#[openapi(
    apiKey,
    in = "cookie",
    name = "xiler-session",
    description = "The session cookie."
)]
pub struct FullUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: Duration,
    /// A int that contains the user roles, can be parsed by using bitwize operations.
    pub roles: usize,
    /// A map of authentication tokens, the key is the authentication type.
    /// For example: 0 = password, 1 = google, 2 = GitHub, etc.
    /// The value is the hash of the password or the ID of the account on the provider
    pub authentication: UserAuthenticationMap,
    pub verification_token: Option<String>,
}

impl FullUser {
    pub fn to_user(self) -> User {
        User {
            id: self.id.to_string(),
            username: self.username,
            email: self.email,
            created_at: self.created_at.num_seconds() as usize,
            roles: self.roles,
            authentication: self.authentication.keys().fold(0, |acc, x| acc | x),
            verified: self.verification_token.is_none(),
        }
    }
}
