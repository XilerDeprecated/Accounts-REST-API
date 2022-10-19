// Represents a user in memory

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type UserAuthenticationMap = HashMap<u8, String>;

#[derive(Serialize, Apiv2Schema)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: usize,
    /// A small int that contains the user roles, can be parsed by using bitwize operations.
    pub roles: u8,
}

/// Contains the minimum data for a user to register.
#[derive(Deserialize, Apiv2Schema)]
pub struct UserRegistration {
    pub username: String,
    pub email: String,
    /// A hashed version of the password.
    pub password: String,
}

#[derive(Clone)]
pub struct FullUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    /// A small int that contains the user roles, can be parsed by using bitwize operations.
    pub roles: u8,
    /// A map of authentication tokens, the key is the authentication type.
    /// For example: 0 = password, 1 = google, 2 = GitHub, etc.
    /// The value is the hash of the password or the ID of the account on the provider
    pub authentication: UserAuthenticationMap,
    pub verification_token: String,
}
