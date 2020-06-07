use actix_web::{web, HttpResponse};

use core::errors::ServiceError;
use core::models::dto::PayloadNumbersDto;

use web_contrib::utils::set_response_headers;

use crate::controllers::contacts::{create as ctrl_create, get_contacts as ctrl_get_contacts};
use crate::queries::*;

use log::info;

pub async fn get_contacts(
    info: web::Path<String>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
    contact_dao: web::Data<Box<dyn PersistentContactsDao>>,
) -> Result<HttpResponse, ServiceError> {
    info!("routes/user/get_contacts");

    let users = ctrl_get_contacts(&info.into_inner(), user_dao, blacklist_dao, contact_dao)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(users);

    set_response_headers(&mut res);

    Ok(res)
}

pub async fn create(
    info: web::Path<(String, String)>,
    mut payload: web::Json<PayloadNumbersDto>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
    contacts_dao: web::Data<Box<dyn PersistentContactsDao>>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/contact_exists/exists");

    let info = info.into_inner();
    let users = ctrl_create(
        &info.0,
        &info.1,
        &mut payload.numbers,
        user_dao,
        blacklist_dao,
        contacts_dao,
    )?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(users);

    set_response_headers(&mut res);

    Ok(res)
}
