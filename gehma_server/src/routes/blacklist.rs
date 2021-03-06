use crate::controllers::blacklist::{create_entry, delete_entry, get_entry};
use crate::queries::*;
use actix_web::{web, HttpResponse};
use core::errors::ServiceError;
use log::info;
use web_contrib::utils::set_response_headers;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostData {
    pub hash_blocked: String,
    pub country_code: String,
}

pub async fn get_all(
    info: web::Path<String>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/get_all");

    let info = info.into_inner();

    let users = get_entry(&info, user_dao, blacklist_dao)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(users);

    set_response_headers(&mut res);

    Ok(res)
}

pub async fn add(
    info: web::Path<String>,
    data: web::Json<PostData>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/blacklist/add");

    create_entry(
        &info.into_inner(),
        &data.into_inner(),
        user_dao,
        blacklist_dao,
    )?;

    let mut res = HttpResponse::Ok().content_type("application/json").finish();
    set_response_headers(&mut res);

    Ok(res)
}

pub async fn delete(
    info: web::Path<String>,
    data: web::Json<PostData>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/delete");

    delete_entry(
        &info.into_inner(),
        &data.into_inner(),
        user_dao,
        blacklist_dao,
    )?;

    let mut res = HttpResponse::Ok().content_type("application/json").finish();
    set_response_headers(&mut res);

    Ok(res)
}
