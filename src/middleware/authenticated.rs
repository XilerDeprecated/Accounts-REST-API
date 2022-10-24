// TODO: Refactor this file
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
    util::{
        hashing::xx_hash,
        parse::{parse_browser_cookie, parse_user_agent},
    },
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
        let cookie = cookie.value().to_string();

        let db = self.database.clone();
        let svc = self.service.clone();

        Box::pin(async move {
            let mut temporary = db.temporary.lock().unwrap();
            let client_id = temporary.get(cookie.clone()).await;
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

            if cookie.starts_with("s1") {
                let user_agent = match req.headers().get("User-Agent") {
                    Some(agent) => agent,
                    None => {
                        let (req, _pl) = req.into_parts();
                        let res = HttpResponse::BadRequest()
                            .json(Status {
                                message: "No user agent present".to_string(),
                            })
                            .map_into_right_body();

                        return Ok(ServiceResponse::new(req, res));
                    }
                };

                let ip = req.peer_addr().unwrap().ip().to_string();
                let parsed_user_agent = parse_user_agent(user_agent.to_str().unwrap().to_string());
                let expected_cookie = match parse_browser_cookie(&cookie) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        let (req, _pl) = req.into_parts();
                        let res = HttpResponse::BadRequest()
                            .json(Status {
                                message: e.to_string(),
                            })
                            .map_into_right_body();

                        return Ok(ServiceResponse::new(req, res));
                    }
                };

                // s1.
                // 2426094911.
                // 1768803836-3566326825-3912814204||3967086928-2746952293-707505781.
                // 3788223220_59786466.
                // Jmtl9LJ3bAYkoymfnCdHCjYjE00hJhdJ
                let penalty_threshold = 0.6;
                let mut cookie_owner_probability: f64 = 1.0;
                let cookie_platform_pentalty_values = (expected_cookie.platforms.len() * 3) as f64;
                // Improve accuracy the more variables
                let penalty: f64 = 1.0
                    / (1.0
                        + cookie_platform_pentalty_values
                        + expected_cookie.extensions.len() as f64);

                if expected_cookie.ip != xx_hash(&ip) {
                    cookie_owner_probability -= penalty;
                }

                for (expected, received) in expected_cookie
                    .platforms
                    .into_iter()
                    .zip(parsed_user_agent.platforms.into_iter())
                {
                    if expected.name != xx_hash(&received.name) {
                        cookie_owner_probability -= penalty;
                    }

                    if expected.version != xx_hash(&received.version) {
                        cookie_owner_probability -= penalty;
                    }

                    if expected.details != xx_hash(&received.details) {
                        cookie_owner_probability -= penalty;
                    }
                }

                for (expected, received) in expected_cookie
                    .extensions
                    .into_iter()
                    .zip(parsed_user_agent.extensions.into_iter())
                {
                    if expected != xx_hash(&received) {
                        cookie_owner_probability -= penalty;
                    }
                }

                let same_client = cookie_owner_probability > penalty_threshold;

                if !same_client {
                    let (req, _pl) = req.into_parts();
                    let res = HttpResponse::Gone()
                        .json(Status {
                            message: "Anti-Cookie theft has removed this session.".to_string(),
                        })
                        .map_into_right_body();

                    let mut temporary = db.temporary.lock().unwrap();
                    temporary.delete(cookie).await;
                    drop(temporary);

                    return Ok(ServiceResponse::new(req, res));
                }
            }

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
