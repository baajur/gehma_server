use crate::auth::FirebaseDatabaseConfiguration;
use actix_web::HttpRequest;
use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::{Blacklist, PhoneNumber, User};
use crate::utils::QueryParams;

use crate::Pool;

use log::{debug, info};

use crate::controllers::blacklist::{create_entry, get_entry, delete_entry};

#[derive(Debug, Deserialize)]
pub struct GetAllData {
    pub numbers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostData {
    pub blocked: String,
    pub country_code: String,
}

pub fn get_all(
    req: HttpRequest,
    info: web::Path<(String)>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/get_all");

    let info = info.into_inner();
    web::block(move || get_entry(&info, pool, &query.firebase_uid, firebase_config)).then(
        |res| match res {
            Ok(users) => {
                let mut res = HttpResponse::Ok()
                    .content_type("application/json")
                    .json(users);
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

pub fn add(
    req: HttpRequest,
    info: web::Path<(String)>,
    data: web::Json<PostData>,
    query: web::Query<QueryParams>,
    pool: web::Data<Pool>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/add");

    web::block(move || {
        create_entry(
            &info.into_inner(),
            &data.into_inner(),
            pool,
            &query.firebase_uid,
            firebase_config,
        )
    })
    .then(|res| match res {
        Ok(_) => {
            let mut res = HttpResponse::Ok().content_type("application/json").finish();
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn delete(
    req: HttpRequest,
    info: web::Path<(String)>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/delete");

    web::block(move || {
        delete_entry(
            &info.into_inner(),
            &data.into_inner(),
            pool,
            &query.firebase_uid,
            firebase_config,
        )
    })
    .then(|res| match res {
        Ok(_) => {
            let mut res = HttpResponse::Ok().content_type("application/json").finish();
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}
