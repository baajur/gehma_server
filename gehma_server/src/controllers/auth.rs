use web_contrib::auth::Auth;
use crate::Pool;
use actix_web::web;
use core::errors::ServiceError;
use core::models::{PhoneNumber, User};
use uuid::Uuid;

use log::{info, error};

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
        let token = Uuid::new_v4().simple().to_string();

        match crate::queries::user::create_query(
            &parsed,
            &body.country_code,
            &body.client_version,
            &token,
            &pool,
        ) {
            Ok(user) => Ok(user),
            Err(ServiceError::AlreadyExists(_)) => {
                crate::queries::user::get_entry_by_tel_query(&parsed, &pool)
            }
            Err(e) => {
                error!("{}", e);
                Err(ServiceError::InternalServerError)
            }
        }
    } else {
        info!("Code was wrong");
        Err(ServiceError::BadRequest(
            "Check node returned false".to_string(),
        ))
    }
}
