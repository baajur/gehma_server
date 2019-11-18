use actix_service::{Service, Transform};
use actix_web::error::PayloadError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage, HttpResponse};
//use actix::fut::{err, ok};
use bytes::BytesMut;
use core::errors::ServiceError;
use futures::future::{FutureResult, ok, err};
use futures::stream::Stream;
use futures::{Future, Poll};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;

pub struct Auth {
    firebase_project_id: String,
    firebase_auth_token: String,
}

impl Auth {
    pub fn default(firebase_project_id: String, firebase_auth_token: String) -> Auth {
        Self {
            firebase_project_id,
            firebase_auth_token,
        }
    }
}

impl<S: 'static, B> Transform<S> for Auth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware {
            service: Rc::new(RefCell::new(service)),
            firebase_project_id: self.firebase_project_id.clone(),
            firebase_auth_token: self.firebase_auth_token.clone(),
        })
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<RefCell<S>>,
    firebase_project_id: String,
    firebase_auth_token: String,
}

impl<S, B> Service for AuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let mut svc = self.service.clone();

        info!(
            "Authentication started. You requested: {:?}",
            req.match_info()
        );

        println!("{:?}", req.path());
        println!("{:?}", req.query_string());

        if req.path() == "/api/signin" {
            //we have no uid yet

            return Box::new(
                req.take_payload()
                    .fold(BytesMut::new(), move |mut body, chunk| {
                        body.extend_from_slice(&chunk);
                        Ok::<_, PayloadError>(body)
                    })
                    .map_err(|e| e.into())
                    .and_then(move |bytes| {
                        let (req2, _) = req.into_parts();
                        Box::new(futures::future::ok(ServiceResponse::new(req2, HttpResponse::Ok().finish())))
                        //Box::new(svc.call(req))

                        //FIXME
                        /*
                        let t: crate::controllers::user::PostUser =
                            serde_json::from_str(&format!("{:?}", bytes))
                                .expect("cannot parse PostUser");
                        let tele_num = t.tele_num;
                        let fire_uid = t.firebase_uid;
                        */

                        //FIXME
                        /*
                        let client = Client::new();
                        let result: FirebaseAuthResponse = client
                            .get(&format!(
                                "https://{}.firebaseio.com/users/{}/.json?auth={}",
                                self.firebase_project_id, fire_uid, self.firebase_auth_token
                            ))
                            .send()
                            .unwrap()
                            .json()
                            .unwrap();

                        let (req2, _) = req.into_parts();
                        */
                        /*
                        if tele_num == result.tele_num {
                            //Box::new(svc.call(req))
                            Ok(Box::new(ok(ServiceResponse::new(req2, HttpResponse::Ok().finish()))))
                        } else {
                            //ok(ServiceResponse::new(req2, HttpResponse::Unauthorized().finish()))
                            Ok(Box::new(ok(ServiceResponse::new(req2, HttpResponse::Unauthorized().finish()))))
                        }
                        */
                    }),
            );
        }

        //FIXME remove unwrap
        /*
        let result : Result<FirebaseAuthResponse, reqwest::Error>  = client
            .get(&format!("https://{}.firebaseio.com/users/{}/.json?auth={}", self.firebase_project_id, uid, self.firebase_auth_token))
            .send()
            .unwrap()
            .json();
        */

        //        println!("{:?}", result.unwrap());

        info!("Authentication ended.");
        Box::new(svc.call(req))
    }
}

/// Response of firebase for auth requests
#[derive(Debug, Deserialize)]
struct FirebaseAuthResponse {
    tele_num: String,
}
