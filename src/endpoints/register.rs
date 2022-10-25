use std::collections::HashMap;

use actix_web::HttpRequest;
use chrono::{Duration, Utc};
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
    types::FullDatabase,
    util::{parse::parse_user_agent, sessions::generate_browser_session},
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
) -> Result<CreatedJson<UserRegistrationResponse>, ClientError> {
    let created_at = Duration::seconds(Utc::now().timestamp());
    let id = Uuid::new_v4();
    let full_user = FullUser {
        id,
        username: body.username.clone(),
        email: body.email.clone(),
        created_at,
        roles: 0,
        authentication: HashMap::new(),
        // TODO: Generate verification token
        verification_token: None,
    };

    let mut persistent = db.persistent.lock().unwrap();
    let res = persistent.register_user(full_user.clone()).await;
    drop(persistent);
    if let Err(e) = res {
        return Err(ClientError::BadRequest(Status { message: e }));
    }

    let user_agent = match data.headers().get("User-Agent") {
        Some(agent) => agent,
        None => {
            return Err(ClientError::BadRequest(Status {
                message: "No user agent present".to_string(),
            }))
        }
    };
    let ip = data.peer_addr().unwrap().ip().to_string();
    let parsed_user_agent = parse_user_agent(user_agent.to_str().unwrap().to_string());

    let token = generate_browser_session(ip, parsed_user_agent);
    let mut temporary = db.temporary.lock().unwrap();
    temporary.set(token.clone(), id.to_string()).await;

    Ok(CreatedJson(UserRegistrationResponse {
        user: full_user.to_user(),
        session: Session { token, ttl: 0 },
    }))
}
