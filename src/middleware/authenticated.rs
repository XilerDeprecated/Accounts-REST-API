use futures::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Mutex,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, HttpResponse,
};

use crate::{
    constants::SESSION_KEY, structs::Status, traits::TemporaryStorageProvider,
    util::data::TemporaryStorage,
};

pub struct Authenticated {
    temporary_storage: Data<Mutex<TemporaryStorage>>,
}

impl Authenticated {
    pub fn new(temporary_storage: Data<Mutex<TemporaryStorage>>) -> Self {
        Self { temporary_storage }
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for Authenticated
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
            temporary_storage: self.temporary_storage.clone(),
        }))
    }
}

pub struct AuthenticatedMiddleware<S> {
    service: Rc<S>,
    temporary_storage: Data<Mutex<TemporaryStorage>>,
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

        let db = self.temporary_storage.clone();
        let svc = self.service.clone();

        Box::pin(async move {
            let mut db = db.lock().unwrap();
            let client_id = db.get(cookie.value().to_string()).await;
            drop(db);

            if client_id.is_none() {
                let (req, _pl) = req.into_parts();
                let res = HttpResponse::Unauthorized()
                    .json(Status {
                        message: "Not authenticated".to_string(),
                    })
                    .map_into_right_body();

                return Ok(ServiceResponse::new(req, res));
            }

            let res = svc.call(req).await?;

            Ok(res.map_into_left_body())
        })
    }
}
