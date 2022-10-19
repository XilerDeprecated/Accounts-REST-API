use std::{collections::HashMap, sync::Mutex};

use actix_web::{web::Data, HttpRequest};
use chrono::{DateTime, Utc};
use paperclip::actix::{api_v2_operation, web::Json, Apiv2Schema, CreatedJson};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    errors::ClientError,
    structs::{
        session::Session,
        user::{FullUser, User, UserRegistration},
        Status,
    },
    traits::{PersistentStorageProvider, TemporaryStorageProvider},
    util::{
        data::{PersistentStorage, TemporaryStorage},
        sessions::generate_browser_session,
    },
};

/// Merge the user with the session details
#[derive(Serialize, Apiv2Schema)]
pub struct UserRegistrationResponse {
    pub user: User,
    pub session: Session,
}

/// Register a new user, if none exists yet.
/// If a user already exists, a bad request is returned with some more information.
#[api_v2_operation]
pub async fn register(
    persistent: Data<Mutex<PersistentStorage>>,
    temporary: Data<Mutex<TemporaryStorage>>,
    body: Json<UserRegistration>,
    data: HttpRequest,
) -> Result<CreatedJson<UserRegistrationResponse>, ClientError> {
    let created_at = Utc::now();
    let id = Uuid::new_v4();
    let full_user = FullUser {
        id,
        username: body.username.clone(),
        email: body.email.clone(),
        created_at,
        roles: 0,
        authentication: HashMap::new(),
        verification_token: "".to_string(),
    };

    let mut persistent = persistent.lock().unwrap();
    if let Err(e) = persistent.register_user(full_user).await {
        return Err(ClientError::BadRequest(Status { message: e }));
    }
    drop(persistent);

    let user_agent = match data.headers().get("User-Agent") {
        Some(agent) => agent,
        None => {
            return Err(ClientError::BadRequest(Status {
                message: "No user agent present".to_string(),
            }))
        }
    };

    let token = generate_browser_session(user_agent.to_str().unwrap().to_string());
    let mut temporary = temporary.lock().unwrap();
    temporary.set(token.clone(), id.to_string()).await;

    Ok(CreatedJson(UserRegistrationResponse {
        user: User {
            id: id.to_string(),
            username: body.username.clone(),
            email: body.email.clone(),
            created_at: seconds_since_epoch(&created_at),
            roles: 0,
        },
        session: Session { token, ttl: 0 },
    }))
}

fn seconds_since_epoch(created_at: &DateTime<Utc>) -> usize {
    created_at.timestamp() as usize
}
