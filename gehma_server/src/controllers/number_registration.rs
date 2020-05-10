use actix_web::web;
use core::errors::ServiceError;
use core::models::dto::*;
use core::models::PhoneNumber;
use uuid::Uuid;

use log::{error, info};

use crate::services::number_registration::NumberRegistrationService;
use crate::queries::*;

pub(crate) fn request(
    body: RequestCodeDto,
    number_registration_service: web::Data<NumberRegistrationService>,
) -> Result<(), ServiceError> {
    info!("controllers/auth/request_code");

    let parsed = PhoneNumber::my_from(&body.tele_num, &body.country_code)?;

    number_registration_service.request_code(&parsed)?;

    Ok(())
}

pub(crate) fn check_code(
    body: RequestCheckCodeDto,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    number_registration_service: web::Data<NumberRegistrationService>,
) -> Result<UserDto, ServiceError> {
    info!("controllers/auth/check_code");

    let parsed = PhoneNumber::my_from(&body.tele_num, &body.country_code)?;

    let res = number_registration_service.check_code(&parsed, &body.code)?;

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
