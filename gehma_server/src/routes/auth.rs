use crate::Pool;
use actix_web::{error::BlockingError, web, HttpResponse};
use core::errors::ServiceError;
use web_contrib::auth::Auth;

use log::{info};
use futures::Future;

use crate::controllers::auth::{request, check_code};

use web_contrib::utils::set_response_headers;

#[derive(Debug, Deserialize)]
pub struct RequestCode {
    pub tele_num: String,
    pub country_code: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestCheckCode {
    pub tele_num: String,
    pub code: String,
    pub country_code: String,
    pub client_version: String,
}

pub async fn request_code(
    _info: web::Path<()>,
    body: web::Json<RequestCode>,
    pool: web::Data<Pool>,
    auth: web::Data<Auth>,
) -> HttpResponse {
    info!("controllers/auth/request_code");

    let res = request(
            body.into_inner(),
            pool,
            auth,
        ).unwrap();

    let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(res);
            //set_response_headers(&mut res);
            //Ok(res)
    res

    /*
    web::block(move || {
        request(
            body.into_inner(),
            pool,
            auth,
        )
    })
    .then(|res| match res {
        Ok(res) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(res);
            set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
    */
}

pub async fn check(
    _info: web::Path<()>,
    body: web::Json<RequestCheckCode>,
    pool: web::Data<Pool>,
    auth: web::Data<Auth>,
) -> HttpResponse {
    info!("controllers/auth/check");

    let res = check_code(
            body.into_inner(),
            pool,
            auth
        ).unwrap();

    let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(res);
            //set_response_headers(&mut res);
            //Ok(res)
    res


    /*
    web::block(move || {
        check_code(
            body.into_inner(),
            pool,
            auth
        )
    })
    .then(|res| match res {
        Ok(res) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(res);
            set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
    */
}
