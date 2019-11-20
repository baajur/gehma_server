use crate::auth::FirebaseDatabaseConfiguration;
use crate::Pool;
use actix_web::{error::BlockingError, web, HttpResponse};
use actix_multipart::{Multipart};
use core::errors::ServiceError;
use futures::stream::Stream;

use log::{error, info};
use crate::utils::QueryParams;
use futures::Future;

use crate::controllers::user::{create_entry, get_entry, update_user_with_auth, save_file};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostUser {
    pub tele_num: String,
    pub country_code: String,
    pub client_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub description: String,
    pub led: String,
    pub client_version: String,
}

pub fn signin(
    _info: web::Path<()>,
    body: web::Json<PostUser>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/add");

    web::block(move || create_entry(body.into_inner(), pool, &query.firebase_uid, firebase_config)).then(
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

pub fn get(
    info: web::Path<(String)>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/get");

    web::block(move || get_entry(&info.into_inner(), pool, &query.firebase_uid, firebase_config)).then(
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

pub fn upload_profile_picture(
    info: web::Path<String>,
    multipart: Multipart,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/upload_profile_picture");

    let uid = info.into_inner();
    multipart
        .map_err(|err| {
            error!("Multipart error: {}", err);
            ServiceError::InternalServerError
        })
        .map(move |field| {
            save_file(
                uid.clone(),
                field,
                pool.clone(),
                &query.firebase_uid,
                firebase_config.clone(),
            )
            .into_stream()
        })
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|err| {
            error!("Multipart error: {}", err);
            err
        })
}



pub fn update(
    info: web::Path<(String)>,
    data: web::Json<UpdateUser>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/update");

    web::block(move || update_user_with_auth(&info.into_inner(), &data.into_inner(), &pool, &query.firebase_uid, firebase_config)).then(|res| {
        match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(&user)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}
