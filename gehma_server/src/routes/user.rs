use crate::ratelimits::RateLimitWrapper;
use crate::Pool;
use actix_web::{web, HttpResponse};
use core::errors::ServiceError;
use core::models::dto::{PostUserDto, UpdateUserDto};

use log::info;
use web_contrib::utils::{set_response_headers, QueryParams};

use crate::controllers::user::{
    get_entry, update_token_handler, update_user_with_auth, user_signin,
};
use crate::persistence::user::PersistentUserDao;
use chrono::Local;
use web_contrib::auth::Auth;
use web_contrib::push_notifications::NotifyService;

pub async fn signin(
    _info: web::Path<()>,
    body: web::Json<PostUserDto>,
    query: web::Query<QueryParams>,
    user_dao: web::Data<&dyn PersistentUserDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/signin");

    let current_time = Local::now();

    let user = user_signin(
        body.into_inner(),
        &query.access_token,
        user_dao,
        current_time,
    )
    .map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(user);

    set_response_headers(&mut res);

    Ok(res)
}

pub async fn get(
    info: web::Path<String>,
    query: web::Query<QueryParams>,
    user_dao: web::Data<&dyn PersistentUserDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/get");

    let users = get_entry(&info.into_inner(), &query.access_token, user_dao)
        .map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(users);

    set_response_headers(&mut res);

    Ok(res)
}

pub async fn get_contacts(
    info: web::Path<String>,
    query: web::Query<QueryParams>,
    user_dao: web::Data<&dyn PersistentUserDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/get_contacts");

    let users =
        crate::controllers::user::get_contacts(&info.into_inner(), user_dao, &query.access_token)
            .map_err(|_err| ServiceError::InternalServerError)?;

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
    query: web::Query<QueryParams>,
    user_dao: web::Data<&dyn PersistentUserDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/update");

    let current_time = Local::now();

    let user = update_user_with_auth(
        &info.into_inner(),
        &data.into_inner(),
        &query.access_token,
        user_dao,
        current_time,
    )
    .map_err(|_err| ServiceError::InternalServerError)?;

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
    query: web::Query<QueryParams>,
    user_dao: web::Data<&dyn PersistentUserDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/push_notification/update_token");

    let _ = update_token_handler(
        _info.into_inner(),
        body.into_inner(),
        &query.access_token,
        user_dao,
    )
    .map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok().content_type("application/json").json(());

    set_response_headers(&mut res);

    Ok(res)
}
