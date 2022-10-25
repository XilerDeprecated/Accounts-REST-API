use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError};
use enum_display_derive::Display;
use paperclip::actix::api_v2_errors;

use crate::structs::Status;

#[api_v2_errors(
    code = 400,
    description = "Bad request",
    code = 401,
    description = "Unauthorized",
    // code = 403,
    // description = "Forbidden",
    // code = 404,
    // description = "Not found"
    code = 500,
    description = "Internal server error",
    // code = 501,
    // description = "Not implemented"
)]
#[derive(Debug, Display)]
pub enum HttpError {
    BadRequest(Status),
    Unauthorized(Status),
    // Forbidden(Status),
    // NotFound(Status),
    InternalServerError(Status),
}

impl ResponseError for HttpError {
    fn error_response(&self) -> HttpResponse {
        match self {
            HttpError::BadRequest(status) => HttpResponse::BadRequest().json(status),
            HttpError::Unauthorized(status) => HttpResponse::Unauthorized().json(status),
            // HttpError::Forbidden(status) => HttpResponse::Forbidden().json(status),
            // HttpError::NotFound(status) => HttpResponse::NotFound().json(status),
            HttpError::InternalServerError(status) => {
                HttpResponse::InternalServerError().json(status)
            } // HttpError::NotImplemented(status) => HttpResponse::NotImplemented().json(status),
        }
    }
}
