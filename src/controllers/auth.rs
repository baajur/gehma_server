use crate::auth::Auth;
use crate::Pool;
use actix_multipart::{Field, MultipartError};
use actix_web::{error::BlockingError, error::PayloadError, web};
use core::errors::ServiceError;
use core::models::{PhoneNumber, User};
use diesel::{prelude::*, PgConnection};
use futures::future::{err, Either};
use futures::stream::Stream;
use futures::Future;
use uuid::Uuid;

use crate::auth::Authenticator;
use log::{error, info};
use std::io::Write;

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
    pool: web::Data<Pool>,
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

    if (res) {
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
