use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use crate::Pool;
use ::core::errors::ServiceError;
use ::core::models::{Analytic, PhoneNumber, UsageStatisticEntry, User};

use log::{info, debug};

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub token: String,
}

pub fn update_token(
    _info: web::Path<(String)>,
    body: web::Json<Payload>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/push_notification/update_token");
    debug!("path {:?}", _info);
    debug!("body {:?}", body);

    web::block(move || update_token_handler(_info.into_inner(), body.into_inner(), pool)).then(
        |res| match res {
            Ok(user) => {
                let mut res = HttpResponse::Ok()
                    .content_type("application/json")
                    .json(user);
                crate::utils::set_response_headers(&mut res);
                Ok(res)
            }
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        },
    )
}

fn update_token_handler(
    uid: String,
    payload: Payload,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let parsed = Uuid::parse_str(&uid)?;

    crate::queries::push_notification::update_token_query(parsed, payload.token, &pool)
}
