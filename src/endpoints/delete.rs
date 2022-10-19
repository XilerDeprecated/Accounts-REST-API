use std::sync::Mutex;

use actix_web::web::Data;
use paperclip::actix::{api_v2_operation, web::Json, Apiv2Schema};
use serde::Serialize;

use crate::{
    errors::ClientError,
    structs::{session::Session, user::User, Status},
    util::data::PersistentStorage,
};

/// Merge the user with the session details
#[derive(Serialize, Apiv2Schema)]
pub struct UserRegistrationResponse {
    pub user: User,
    pub session: Session,
}

/// Delete your account
#[api_v2_operation]
pub async fn delete_account(
    _db: Data<Mutex<PersistentStorage>>,
) -> Result<Json<Status>, ClientError> {
    Err(ClientError::BadRequest(Status {
        message: "Not implemented".to_string(),
    }))
    // let mut db = db.lock().unwrap();

    // TODO: Get the user ID from the session
    // match db.delete_user(body.id.clone()).await {
    //     Ok(_) => Ok(Json(Status {
    //         code: 201,
    //         message: "success".to_string(),
    //     })),
    //     Err(e) => Err(ClientError::BadRequest(Status {
    //         code: 400,
    //         message: e.to_string(),
    //     })),
    // }
}
