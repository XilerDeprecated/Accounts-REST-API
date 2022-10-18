use actix_web::Error;
// Allow a user to register
use paperclip::actix::{api_v2_operation, web::Json};

use crate::structs::{user::UserRegistration, Status};

#[api_v2_operation]
pub async fn register(body: Json<UserRegistration>) -> Result<Json<Status>, Error> {
    // TODO: Create a new user + save it in the db
    // TODO: Check if the user already exists

    Ok(Json(Status {
        code: 201,
        message: "ok".to_string(),
    }))
}
