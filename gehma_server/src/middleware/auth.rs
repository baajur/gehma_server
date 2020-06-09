use crate::services::session::SessionService;
use chrono::prelude::*;
use core::errors::ServiceError;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// time expires
    exp: DateTime<Utc>,
    /// time initialised
    iat: DateTime<Utc>,
}

use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpResponse,
};
use futures::{
    future::{ok, Ready},
    Future,
};
use log::{debug, info, warn};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct Authentication;

impl<S, B> Transform<S> for Authentication
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddleware { service })
    }
}
pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service for AuthenticationMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let session_service = req
            .app_data::<SessionService>()
            .expect("no session service configured");

        if req.path().starts_with("/api/signin")
            || req.path().starts_with("/api/auth")
            || req.path().starts_with("/api/static")
        {
            debug!("Skipping authentication");
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        let token = req
            .headers()
            .get("AUTHORIZATION")
            .map(|value| value.to_str().ok())
            .ok_or_else(|| {
                warn!("No AUTHORIZATION header ({})", req.path());
                ServiceError::Unauthorized
            })
            .unwrap()
            .unwrap();

        let authenticate_pass = session_service.validate(token.to_string()).unwrap();

        if authenticate_pass {
            debug!("Authentication ok");
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        } else {
            info!("Authentication failed");
            Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json("Session expired")
                        .into_body(),
                ))
            })
        }
    }
}
