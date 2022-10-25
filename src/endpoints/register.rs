use std::collections::HashMap;

use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use paperclip::actix::{api_v2_operation, web::Json, Apiv2Schema, CreatedJson};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    errors::HttpError,
    structs::{
        session::Session,
        user::{FullUser, User, UserRegistration},
        Status,
    },
    traits::{PersistentStorageProvider, TemporaryStorageProvider},
    types::FullDatabase,
    util::{hashing::argon2_hash, sessions::create_browser_session},
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
        // TODO: Generate verification token
        verification_token: None,
    };

    let mut persistent = db.persistent.lock().unwrap();
    let res = persistent.register_user(full_user.clone()).await;
    drop(persistent);
    if let Err(e) = res {
        return Err(HttpError::InternalServerError(Status { message: e }));
    }

    let token = create_browser_session(data)?;
    let mut temporary = db.temporary.lock().unwrap();
    temporary.set(token.clone(), id.to_string()).await;

    Ok(CreatedJson(UserRegistrationResponse {
        user: full_user.to_user(),
        session: Session { token, ttl: 0 },
    }))
}
