use paperclip::actix::{api_v2_operation, web::Json, Apiv2Schema};
use serde::Serialize;

use crate::{
    errors::ClientError,
    structs::{
        session::Session,
        user::{FullUser, User},
        Status,
    },
    traits::PersistentStorageProvider,
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
pub async fn delete_account(db: FullDatabase, user: FullUser) -> Result<Json<Status>, ClientError> {
    let mut db = db.persistent.lock().unwrap();
    let res = db.delete_user(user.id.to_string()).await;
    drop(db);

    match res {
        Ok(_) => Ok(Json(Status {
            message: "success".to_string(),
        })),
        Err(e) => Err(ClientError::BadRequest(Status {
            message: e.to_string(),
        })),
    }
}
