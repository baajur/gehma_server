use crate::Pool;
use actix_web::{web, HttpResponse};
use web_contrib::auth::Auth;

use log::{info};

use crate::controllers::auth::{request, check_code};

use web_contrib::utils::set_response_headers;

use core::errors::ServiceError;

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
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/auth/request_code");

    let _ = request(
            body.into_inner(),
            pool,
            auth,
        ).map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
                    .content_type("application/json")
                    .json(());
    set_response_headers(&mut res);

    Ok(res)
}

pub async fn check(
    _info: web::Path<()>,
    body: web::Json<RequestCheckCode>,
    pool: web::Data<Pool>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/auth/check");

    let res = check_code(
            body.into_inner(),
            pool,
            auth
        ).map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(res);

    set_response_headers(&mut res);
    Ok(res)
}
