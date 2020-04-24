use crate::controllers::blacklist::{create_entry, delete_entry, get_entry};
use crate::Pool;
use actix_web::{web, HttpResponse};
use core::errors::ServiceError;
use log::info;
use web_contrib::auth::Auth;
use web_contrib::utils::{set_response_headers, QueryParams};

use crate::persistence::blacklist::PersistentBlacklistDao;
use crate::persistence::user::PersistentUserDao;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostData {
    pub blocked: String, //TODO rename `hash_blocked` #45
    pub country_code: String,
}

pub async fn get_all(
    info: web::Path<String>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
    user_dao: web::Data<&dyn PersistentUserDao>,
    blacklist_dao: web::Data<&dyn PersistentBlacklistDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/get_all");

    let info = info.into_inner();

    let users = get_entry(&info, &query.access_token, user_dao, blacklist_dao)
        .map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(users);

    set_response_headers(&mut res);

    Ok(res)
}

pub async fn add(
    info: web::Path<String>,
    data: web::Json<PostData>,
    query: web::Query<QueryParams>,
    pool: web::Data<Pool>,
    auth: web::Data<Auth>,
    user_dao: web::Data<&dyn PersistentUserDao>,
    blacklist_dao: web::Data<&dyn PersistentBlacklistDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/add");

    create_entry(
        &info.into_inner(),
        &data.into_inner(),
        &query.access_token,
        user_dao,
        blacklist_dao,
        pool,
    )
    .map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok().content_type("application/json").finish();
    set_response_headers(&mut res);

    Ok(res)
}

pub async fn delete(
    info: web::Path<String>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
    user_dao: web::Data<&dyn PersistentUserDao>,
    blacklist_dao: web::Data<&dyn PersistentBlacklistDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/delete");

    delete_entry(
        &info.into_inner(),
        &data.into_inner(),
        &query.access_token,
        user_dao,
        blacklist_dao,
        pool,
    )
    .map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok().content_type("application/json").finish();
    set_response_headers(&mut res);

    Ok(res)
}
