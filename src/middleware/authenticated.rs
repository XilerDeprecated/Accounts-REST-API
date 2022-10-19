use futures::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage, HttpRequest, HttpResponse,
};

use crate::{
    constants::SESSION_KEY,
    structs::{user::FullUser, Status},
    traits::{PersistentStorageProvider, TemporaryStorageProvider},
    types::FullDatabase,
};

pub struct AuthenticationService {
    database: FullDatabase,
}

impl AuthenticationService {
    pub fn new(database: FullDatabase) -> AuthenticationService {
        Self { database }
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for AuthenticationService
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticatedMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticatedMiddleware {
            service: Rc::new(service),
            database: self.database.clone(),
        }))
    }
}

pub struct AuthenticatedMiddleware<S> {
    service: Rc<S>,
    database: FullDatabase,
}

impl<S, B> Service<ServiceRequest> for AuthenticatedMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let cookie = match req.cookie(SESSION_KEY) {
            Some(cookie) => cookie,
            None => {
                let (req, _pl) = req.into_parts();
                let res = HttpResponse::BadRequest()
                    .json(Status {
                        message: format!("'{}' cookie not found", SESSION_KEY),
                    })
                    .map_into_right_body();

                return Box::pin(async move { Ok(ServiceResponse::new(req, res)) });
            }
        };

        let db = self.database.clone();
        let svc = self.service.clone();

        Box::pin(async move {
            let mut temporary = db.temporary.lock().unwrap();
            let client_id = temporary.get(cookie.value().to_string()).await;
            drop(temporary);

            if client_id.is_none() {
                let (req, _pl) = req.into_parts();
                let res = HttpResponse::Unauthorized()
                    .json(Status {
                        message: "Not authenticated".to_string(),
                    })
                    .map_into_right_body();

                return Ok(ServiceResponse::new(req, res));
            }

            let client_id = client_id.unwrap();

            let persistent = db.persistent.lock().unwrap();
            let full_user = persistent.get_user_by_id(client_id).await;
            drop(persistent);

            if full_user.is_none() {
                let (req, _pl) = req.into_parts();
                let res = HttpResponse::Gone()
                    .json(Status {
                        message: "User does not exist anymore.".to_string(),
                    })
                    .map_into_right_body();

                return Ok(ServiceResponse::new(req, res));
            }
            let full_user = full_user.unwrap();

            req.extensions_mut().insert(full_user);

            let res = svc.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

impl FromRequest for FullUser {
    type Error = Error;
    type Future = Ready<Result<FullUser, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(req.extensions().get::<FullUser>().unwrap().clone()))
    }
}
