use actix_web::web::{Json, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::Deserialize;

use crate::{
    errors::HttpError,
    structs::{user::FullUser, Status},
    traits::PersistentStorageProvider,
    types::FullDatabase,
};

#[derive(Deserialize, Apiv2Schema)]
pub struct VerifyUser {
    /// The verification code
    code: String,
}

#[api_v2_operation]
pub async fn verify_user(
    db: FullDatabase,
    full_user: FullUser,
    query: Query<VerifyUser>,
) -> Result<Json<Status>, HttpError> {
    if full_user.verification_token.is_none() {
        return Err(HttpError::BadRequest(Status {
            message: "User is already verified.".to_string(),
        }));
    }

    if query.code != full_user.verification_token.unwrap() {
        return Err(HttpError::Unauthorized(Status {
            message: "Invalid verification code.".to_string(),
        }));
    }

    let persistent = db.persistent.lock().unwrap();
    let res = persistent.verify_user(full_user.id).await;
    drop(persistent);

    match res {
        Ok(_) => Ok(Json(Status {
            message: "User verified".to_string(),
        })),
        Err(message) => Err(HttpError::InternalServerError(Status { message })),
    }
}
