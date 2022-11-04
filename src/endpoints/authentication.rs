use actix_web::web::Json;
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::Deserialize;

use crate::{
    errors::HttpError,
    structs::{user::FullUser, Status},
    traits::PersistentStorageProvider,
    types::FullDatabase,
    util::{actix::Path, math::is_power_of_two},
};

#[derive(Deserialize, Apiv2Schema)]
pub struct AuthenticationMethodValue {
    pub value: String,
}

#[api_v2_operation]
pub async fn remove_authentication_method(
    db: FullDatabase,
    full_user: FullUser,
    method: Path<i16>,
) -> Result<Json<Status>, HttpError> {
    if !is_power_of_two(*method) {
        return Err(HttpError::NotFound());
    }

    let persistent = db.persistent.lock().unwrap();
    let methods = match persistent.get_authentication_methods(full_user.id).await {
        Ok(methods) => methods,
        Err(message) => return Err(HttpError::InternalServerError(Status { message })),
    };

    if methods.len() == 1 {
        return Err(HttpError::BadRequest(Status {
            message: "Cannot remove last authentication method".to_string(),
        }));
    }

    if methods.iter().any(|m| *m == *method) {
        return match persistent
            .remove_authentication_method(full_user.id, *method)
            .await
        {
            Ok(_) => Ok(Json(Status {
                message: "Successfully removed authentication method".to_string(),
            })),
            Err(message) => Err(HttpError::InternalServerError(Status { message })),
        };
    }

    Err(HttpError::NotFound())
}

#[api_v2_operation]
pub async fn update_authentication_method(
    db: FullDatabase,
    full_user: FullUser,
    method: Path<i16>,
    value: Json<AuthenticationMethodValue>,
) -> Result<Json<Status>, HttpError> {
    if !is_power_of_two(*method) {
        return Err(HttpError::BadRequest(Status {
            message: "All authentication methods must be a power of two.".to_string(),
        }));
    }

    let persistent = db.persistent.lock().unwrap();
    let res = persistent
        .update_authentication_method_value(full_user.id, *method, &value.value)
        .await;
    drop(persistent);

    match res {
        Ok(_) => Ok(Json(Status {
            message: "Successfully updated authentication method".to_string(),
        })),
        Err(message) => Err(HttpError::InternalServerError(Status { message })),
    }
}
