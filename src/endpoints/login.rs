use actix_web::{web::Json, HttpRequest};
use paperclip::actix::api_v2_operation;

use crate::{
    errors::HttpError,
    structs::{
        session::Session,
        user::{FullUser, UserLogin},
        Status,
    },
    traits::{PersistentStorageProvider, TemporaryStorageProvider},
    types::FullDatabase,
    util::sessions::create_browser_session,
};

type LoginResult = Result<Json<Session>, HttpError>;
static PASSWORD_AUTHENTICATION: i16 = 0;
static TTL: usize = 60 * 60 * 24 * 30; // 30 days

#[api_v2_operation]
pub async fn add_login(db: FullDatabase, body: Json<UserLogin>, data: HttpRequest) -> LoginResult {
    fn no_match() -> LoginResult {
        Err(HttpError::Unauthorized(Status {
            message: "Could not find match.".to_string(),
        }))
    }
    let persistent = db.persistent.lock().unwrap();
    let mut user: Option<FullUser> = persistent.get_user_by_username(body.username.clone()).await;

    if user.is_none() {
        user = persistent.get_user_by_email(body.username.clone()).await;
    }
    drop(persistent);

    if user.is_none() {
        return no_match();
    }

    let user = user.unwrap();
    let password = match user.authentication.get(&PASSWORD_AUTHENTICATION) {
        Some(password) => password,
        None => {
            return Err(HttpError::BadRequest(Status {
                message: "Password authentication is not a viable authentication for this user."
                    .to_string(),
            }))
        }
    };
    // TODO: Hash passwd in check
    if password != &body.password {
        return no_match();
    }

    let token = create_browser_session(data)?;

    let mut temporary = db.temporary.lock().unwrap();
    temporary.set(token.clone(), user.id.to_string()).await;

    Ok(Json(Session { token, ttl: TTL }))
}
