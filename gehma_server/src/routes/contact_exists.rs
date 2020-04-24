use actix_web::{web, HttpResponse};

use crate::Pool;
use core::errors::ServiceError;
use core::models::dto::PayloadNumbersDto;

use web_contrib::utils::{set_response_headers, QueryParams};

use crate::controllers::contact_exists::get_entry;
use crate::persistence::contact_exists::PersistentContactExistsDao;
use crate::persistence::user::PersistentUserDao;

use log::info;

pub async fn exists(
    info: web::Path<(String, String)>,
    mut payload: web::Json<PayloadNumbersDto>,
    _pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    user_dao: web::Data<&dyn PersistentUserDao>,
    contact_exists_dao: web::Data<&dyn PersistentContactExistsDao>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/contact_exists/exists");

    let info = info.into_inner();
    let users = get_entry(
        &info.0,
        &info.1,
        &mut payload.numbers,
        &query.access_token,
        user_dao,
        contact_exists_dao,
    )
    .map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(users);

    set_response_headers(&mut res);

    Ok(res)
}
