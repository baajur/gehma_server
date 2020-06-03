use actix_web::web;
use core::errors::{InternalServerError, InvalidUserInput, ServiceError};
use core::models::dto::*;
use core::models::PhoneNumber;
use uuid::Uuid;

use log::{debug, error, info};

use crate::queries::*;
use crate::services::number_registration::NumberRegistrationService;

const ACCESS_TOKEN_LENGTH : usize = 32;

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

    // Create a new user when the code was correct
    if res {
        // Check if a user already exists
        match user_dao.get_ref().get_by_tele_num(&parsed) {
            Ok(user) => return Ok(user.into()),
            Err(ServiceError::ResourceDoesNotExist) => debug!("User does not exist. Inserting"),
            Err(e) => return Err(e),
        }

        // If not then create one
        let token = core::utils::generate_random_string(ACCESS_TOKEN_LENGTH);

        match user_dao
            .get_ref()
            .create(&parsed, &body.country_code, &body.client_version, &token)
        {
            Ok(user) => Ok(user.into()),
            Err(e) => {
                error!("{}", e);
                Err(ServiceError::InternalServerError(
                    InternalServerError::DatabaseError(e.to_string()),
                ))
            }
        }
    } else {
        info!("Code was wrong");
        Err(ServiceError::InvalidUserInput(
            InvalidUserInput::InvalidCode,
        ))
    }
}
