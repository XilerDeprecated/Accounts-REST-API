use actix_web::web::Json;
use paperclip::actix::api_v2_operation;

use crate::{
    errors::HttpError,
    structs::user::{FullUser, User},
    types::FullDatabase,
};

#[api_v2_operation]
pub async fn get_account(_db: FullDatabase, full_user: FullUser) -> Result<Json<User>, HttpError> {
    Ok(Json(full_user.to_user()))
}
