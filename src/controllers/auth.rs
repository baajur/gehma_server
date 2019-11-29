use crate::auth::Auth;
use crate::Pool;
use core::errors::ServiceError;
use core::models::{PhoneNumber, User};
use uuid::Uuid;
use actix_web::web;

use log::{info};

use crate::routes::auth::{RequestCheckCode, RequestCode};

/*
#[derive(Debug, Serialize)]
pub struct ResponseCheckCode {
    valid: bool,
    access_token: String,
}
*/

pub(crate) fn request(
    body: RequestCode,
    _pool: web::Data<Pool>,
    auth: web::Data<Auth>,
) -> Result<(), ServiceError> {
    info!("controllers/auth/request_code");

    let parsed = PhoneNumber::my_from(&body.tele_num, &body.country_code)?;

    auth.authenticator.request_code(&parsed)?;

    Ok(())
}

pub(crate) fn check_code(
    body: RequestCheckCode,
    pool: web::Data<Pool>,
    auth: web::Data<Auth>,
) -> Result<User, ServiceError> {
    info!("controllers/auth/check_code");

    let parsed = PhoneNumber::my_from(&body.tele_num, &body.country_code)?;

    let res = auth.authenticator.check_code(&parsed, &body.code)?;

    if res {
        let token = Uuid::new_v4().to_simple().to_string();

        crate::queries::user::create_query(
            &parsed,
            &body.country_code,
            &body.client_version,
            &token,
            &pool,
        )
    } else {
        Err(ServiceError::BadRequest(
            "Check node returned false".to_string(),
        ))
    }
}
