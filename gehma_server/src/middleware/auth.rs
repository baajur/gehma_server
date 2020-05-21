use crate::queries::PersistentUserDao;
use chrono::prelude::*;
use core::errors::ServiceError;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// time expires
    exp: DateTime<Utc>,
    /// time initialised
    iat: DateTime<Utc>,
}

use actix_service::{Service, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::Method,
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

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let mut authenticate_pass: bool = false;

        if req.path().starts_with("api/static") || req.path().starts_with("api/") {
            authenticate_pass = true;
        }

        if !authenticate_pass {
            if let Some(user_dao) = req.app_data::<Box<dyn PersistentUserDao>>() {
                let path : Vec<&str> = req.path().split("/").skip(3).take(1).collect();
                let access_token = req.query_string().split("=").collect::<Vec<_>>()[1].to_string();

                //debug!("id {:?} access_token {:?}", path, access_token);
                let id = uuid::Uuid::parse_str(path[0]).expect("invalid uuid");

                let res = user_dao
                    .get_ref()
                    .get_by_id(&id, access_token)
                    .map_err(|w| {
                        log::error!("{:?}", w);
                        ServiceError::Unauthorized
                    });

                //debug!("auth_res {:?}", res);

                if res.is_ok() {
                    //info!("Authentication ok");
                    authenticate_pass = true;
                } 
                /*else {
                    warn!("auth failed");
                }*/
            }
        }

        if authenticate_pass {
            debug!("Authentication ok");
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        } else {
            warn!("Authentication failed");
            Box::pin(async move {
                Ok(req.into_response(HttpResponse::Unauthorized().finish().into_body()))
            })
        }
    }
}
