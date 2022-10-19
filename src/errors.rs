use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError};
use enum_display_derive::Display;
use paperclip::actix::api_v2_errors;

use crate::structs::Status;

#[api_v2_errors(
    code = 400,
    description = "Bad request",
    // code = 401,
    // description = "Unauthorized",
    // code = 403,
    // description = "Forbidden",
    // code = 404,
    // description = "Not found"
)]
#[derive(Debug, Display)]
pub enum ClientError {
    BadRequest(Status),
    // Unauthorized(Status),
    // Forbidden(Status),
    // NotFound(Status),
}

impl ResponseError for ClientError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ClientError::BadRequest(status) => HttpResponse::BadRequest().json(status),
            // ClientError::Unauthorized(status) => HttpResponse::Unauthorized().json(status),
            // ClientError::Forbidden(status) => HttpResponse::Forbidden().json(status),
            // ClientError::NotFound(status) => HttpResponse::NotFound().json(status),
        }
    }
}
