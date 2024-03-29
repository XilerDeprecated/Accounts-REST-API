use std::collections::HashMap;

use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use paperclip::actix::{api_v2_operation, web::Json, Apiv2Schema, CreatedJson};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    constants::TTL,
    errors::HttpError,
    structs::{
        session::Session,
        user::{FullUser, User, UserRegistration},
        Status,
    },
    traits::{PersistentStorageProvider, TemporaryStorageProvider},
    types::FullDatabase,
    util::{hashing::argon2_hash, random::random_string, sessions::create_browser_session},
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
    db: FullDatabase,
    body: Json<UserRegistration>,
    data: HttpRequest,
) -> Result<CreatedJson<UserRegistrationResponse>, HttpError> {
    if body.username.is_empty() || body.email.is_empty() || body.password.len() < 8 {
        return Err(HttpError::BadRequest(Status {
            message: "Username and email are required. Your password length must also be more than 8. (are you messing with the API? The checks should be handled by the frontend and a basic SHA hash should also be performed there?)".to_string(),
        }));
    } else if body.username.len() > 64 || body.email.len() > 64 || body.password.len() > 128 {
        return Err(HttpError::BadRequest(Status {
            message: "Username, email and password must be less than 64, 64 and 128 characters respectively.".to_string(),
        }));
    }

    let created_at = Duration::seconds(Utc::now().timestamp());
    let id = Uuid::new_v4();
    let mut authentication = HashMap::new();
    let password = argon2_hash(&body.password);
    authentication.insert(0, password);

    let full_user = FullUser {
        id,
        username: body.username.clone(),
        email: body.email.clone(),
        created_at,
        roles: 0,
        authentication,
        verification_token: Some(random_string(64)),
    };

    let persistent = db.persistent.lock().unwrap();
    let res = persistent.register_user(full_user.clone()).await;
    drop(persistent);
    if let Err(e) = res {
        if e.contains("Failed") {
            return Err(HttpError::InternalServerError(Status { message: e }));
        }
        return Err(HttpError::BadRequest(Status { message: e }));
    }

    let token = create_browser_session(data)?;
    let temporary = db.temporary.lock().unwrap();
    temporary.set(token.clone(), id.to_string()).await;

    Ok(CreatedJson(UserRegistrationResponse {
        user: full_user.to_user(),
        session: Session { token, ttl: TTL },
    }))
}
