use actix_web::{web, HttpResponse};

use log::{info};

use crate::controllers::number_registration::{request, check_code};

use web_contrib::utils::set_response_headers;

use core::errors::ServiceError;

use core::models::dto::*;
use crate::services::number_registration::NumberRegistrationService;
use crate::queries::*;

pub async fn request_code(
    _info: web::Path<()>,
    body: web::Json<RequestCodeDto>,
    //pool: web::Data<Pool>,
    number_registration_service: web::Data<NumberRegistrationService>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/auth/request_code");

    let _ = request(
            body.into_inner(),
            number_registration_service,
        )?;

    let mut res = HttpResponse::Ok()
                    .content_type("application/json")
                    .json(());

    set_response_headers(&mut res);

    Ok(res)
}

pub async fn check(
    _info: web::Path<()>,
    body: web::Json<RequestCheckCodeDto>,
    //pool: web::Data<Pool>,
    number_registration_service: web::Data<NumberRegistrationService>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/auth/check");

    let res = check_code(
            body.into_inner(),
            user_dao,
            number_registration_service,
        )?;

    let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(res);

    set_response_headers(&mut res);
    Ok(res)
}
