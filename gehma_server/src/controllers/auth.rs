use actix_web::web;
use core::errors::ServiceError;
use core::models::dto::*;
use core::models::PhoneNumber;
use uuid::Uuid;
use web_contrib::auth::Auth;

use log::{error, info};

use crate::persistence::user::PersistentUserDao;

pub(crate) fn request(
    body: RequestCodeDto,
    auth: web::Data<Auth>,
) -> Result<(), ServiceError> {
    info!("controllers/auth/request_code");

    let parsed = PhoneNumber::my_from(&body.tele_num, &body.country_code)?;

    auth.authenticator.request_code(&parsed)?;

    Ok(())
}

pub(crate) fn check_code(
    body: RequestCheckCodeDto,
    auth: web::Data<Auth>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
) -> Result<UserDto, ServiceError> {
    info!("controllers/auth/check_code");

    let parsed = PhoneNumber::my_from(&body.tele_num, &body.country_code)?;

    let res = auth.authenticator.check_code(&parsed, &body.code)?;

    if res {
        let token = Uuid::new_v4().simple().to_string();

        match user_dao
            .get_ref()
            .create(&parsed, &body.country_code, &body.client_version, &token)
        {
            Ok(user) => Ok(user),
            //Err(ServiceError::AlreadyExists(_)) => user_dao.get_ref().get_by_tele_num(&parsed),
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
