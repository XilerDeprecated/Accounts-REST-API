use std::future::{ready, Ready};

use actix_router::PathDeserializer;
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use derive_more::{AsRef, Deref, DerefMut, Display, From};
use paperclip::actix::Apiv2Schema;
use serde::{de::DeserializeOwned, Deserialize};

use crate::{errors::HttpError, structs::Status};

#[derive(
    Apiv2Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Deref, DerefMut, AsRef, Display, From,
)]
pub struct Path<T>(T);

impl<T> FromRequest for Path<T>
where
    T: DeserializeOwned,
{
    type Error = HttpError;
    type Future = Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            Deserialize::deserialize(PathDeserializer::new(req.match_info()))
                .map(Path)
                .map_err(|err| {
                    HttpError::BadRequest(Status {
                        message: err.to_string(),
                    })
                }),
        )
    }
}
