use actix_web::{error::BlockingError, web, HttpResponse};
use log::{info};
use futures::Future;

use crate::Pool;
use core::errors::ServiceError;
use crate::utils::QueryParams;
use crate::auth::FirebaseDatabaseConfiguration;

use crate::controllers::push_notification::update_token_handler;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub token: String,
}

pub fn update_token(
    _info: web::Path<(String)>,
    body: web::Json<Payload>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/push_notification/update_token");

    web::block(move || update_token_handler(_info.into_inner(), body.into_inner(), pool, &query.firebase_uid, firebase_config)).then(
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


