use crate::Pool;
use actix_web::{web, HttpResponse};
use core::models::dto::{PostUserDto, UpdateUserDto};
use core::errors::ServiceError;
use crate::ratelimits::RateLimitWrapper;

use web_contrib::utils::{QueryParams, set_response_headers};
use log::{info};

use web_contrib::auth::Auth;
use web_contrib::push_notifications::NotifyService;
use crate::controllers::user::{user_signin, get_entry, update_user_with_auth, update_token_handler};
use chrono::{Local};

pub async fn signin(
    _info: web::Path<()>,
    body: web::Json<PostUserDto>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
    notify_service: web::Data<NotifyService>,
    ratelimit_service: web::Data<RateLimitWrapper>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/signin");

    let current_time = Local::now();

    let user = user_signin(
            body.into_inner(),
            pool,
            &query.access_token,
            auth,
            notify_service,
            ratelimit_service,
            current_time,
    ).map_err(|_err| {
        ServiceError::InternalServerError
    })?;

    let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(user);

    set_response_headers(&mut res);

    Ok(res)
}

pub async fn get(
    info: web::Path<String>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/get");

    let users = get_entry(&info.into_inner(), pool, &query.access_token, auth).map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
                    .content_type("application/json")
                    .json(users);

    set_response_headers(&mut res);

    Ok(res)
}

pub async fn get_contacts(
    info: web::Path<String>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/get_contacts");

    let users = crate::controllers::user::get_contacts(&info.into_inner(), pool, &query.access_token, auth).map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
                    .content_type("application/json")
                    .json(users);

    set_response_headers(&mut res);

    Ok(res)
}

/*
pub fn upload_profile_picture(
    info: web::Path<String>,
    multipart: Multipart,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("routes/upload_profile_picture");

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
                &query.access_token,
                auth.clone(),
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
*/

pub async fn update(
    info: web::Path<String>,
    data: web::Json<UpdateUserDto>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
    notify_service: web::Data<NotifyService>,
    ratelimit_service: web::Data<RateLimitWrapper>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/update");

    let current_time = Local::now();

    let user = update_user_with_auth(
            &info.into_inner(),
            &data.into_inner(),
            &pool,
            &query.access_token,
            auth,
            &notify_service,
            &ratelimit_service,
            current_time,
        ).map_err(|_err| ServiceError::InternalServerError)?;

    Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(&user))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTokenPayload {
    pub token: String,
}

pub async fn update_token(
    _info: web::Path<String>,
    body: web::Json<UpdateTokenPayload>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/push_notification/update_token");

    let _ = update_token_handler(
            _info.into_inner(),
            body.into_inner(),
            pool,
            &query.access_token,
            auth,
        ).map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(());

    set_response_headers(&mut res);

    Ok(res)
}
