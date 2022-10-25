use actix_web::web::Json;
use paperclip::actix::api_v2_operation;

use crate::{
    errors::HttpError,
    structs::{user::FullUser, Status},
    traits::TemporaryStorageProvider,
    types::FullDatabase,
};

#[api_v2_operation]
pub async fn logout(db: FullDatabase, full_user: FullUser) -> Result<Json<Status>, HttpError> {
    let temp = db.temporary.lock().unwrap();
    let success = temp.drop_all(full_user.id.to_string()).await;
    drop(temp);

    if success {
        return Ok(Json(Status {
            message: "Successfully logged out".to_string(),
        }));
    }

    Err(HttpError::InternalServerError(Status {
        message: "Failed to log out".to_string(),
    }))
}
