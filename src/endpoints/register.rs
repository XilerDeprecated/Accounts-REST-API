use std::{collections::HashMap, sync::Mutex};

use actix_web::web::Data;
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
    traits::PersistentStorageProvider,
    util::data::PersistentStorage,
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
    db: Data<Mutex<PersistentStorage>>,
    body: Json<UserRegistration>,
) -> Result<CreatedJson<UserRegistrationResponse>, ClientError> {
    let created_at = Utc::now();
    let full_user = FullUser {
        id: Uuid::new_v4(),
        username: body.username.clone(),
        email: body.email.clone(),
        created_at,
        roles: 0,
        authentication: HashMap::new(),
        verification_token: "".to_string(),
    };

    let mut db = db.lock().unwrap();
    if let Err(e) = db.register_user(full_user).await {
        return Err(ClientError::BadRequest(Status {
            message: e,
            code: 400,
        }));
    }
    drop(db);

    Ok(CreatedJson(UserRegistrationResponse {
        user: User {
            id: body.username.clone(),
            username: body.username.clone(),
            email: body.email.clone(),
            created_at: seconds_since_epoch(&created_at),
            roles: 0,
        },
        session: Session {
            token: "".to_string(),
            ttl: 0,
        },
    }))
}

fn seconds_since_epoch(created_at: &DateTime<Utc>) -> usize {
    created_at.timestamp() as usize
}
