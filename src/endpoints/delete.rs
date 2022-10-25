use paperclip::actix::{api_v2_operation, web::Json, Apiv2Schema};
use serde::Serialize;

use crate::{
    errors::HttpError,
    structs::{
        session::Session,
        user::{FullUser, User},
        Status,
    },
    traits::{PersistentStorageProvider, TemporaryStorageProvider},
    types::FullDatabase,
};

/// Merge the user with the session details
#[derive(Serialize, Apiv2Schema)]
pub struct UserRegistrationResponse {
    pub user: User,
    pub session: Session,
}

/// Delete your account
#[api_v2_operation]
pub async fn delete_account(db: FullDatabase, user: FullUser) -> Result<Json<Status>, HttpError> {
    let persistent = db.persistent.lock().unwrap();
    let res = persistent.delete_user(user.id).await;
    drop(persistent);

    let temporary = db.temporary.lock().unwrap();
    temporary.drop_all(user.id.to_string()).await;
    drop(temporary);

    match res {
        Ok(_) => Ok(Json(Status {
            message: "success".to_string(),
        })),
        Err(e) => Err(HttpError::BadRequest(Status {
            message: e.to_string(),
        })),
    }
}
