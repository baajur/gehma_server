use crate::Pool;
use actix_web::{error::BlockingError, web, HttpResponse};
use core::errors::ServiceError;
use futures::stream::Stream;

use futures::Future;
use log::{error, info};

use crate::controllers::events::{get_all_by_city};


/// Get all events by city
pub fn get_all(
    city: web::Path<String>,
    pool: web::Data<Pool>,
    //query: web::Query<QueryParams>,
    //auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("routes/events/get_all");

    web::block(move || get_all_by_city(&city.into_inner(), pool)).then(|res| {
        match res {
            Ok(users) => {
                let mut res = HttpResponse::Ok()
                    .content_type("application/json")
                    .json(users);
                web_contrib::utils::set_response_headers(&mut res);
                Ok(res)
            }
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}
