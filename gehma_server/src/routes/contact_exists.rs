use actix_web::{web, HttpResponse};

use crate::Pool;
use core::errors::ServiceError;
use core::models::dto::PayloadNumbersDto;

use web_contrib::auth::Auth;
use web_contrib::utils::{set_response_headers, QueryParams};

use crate::controllers::contact_exists::get_entry;

use log::info;

pub async fn exists(
    info: web::Path<(String, String)>,
    mut payload: web::Json<PayloadNumbersDto>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/contact_exists/exists");

    let info = info.into_inner();
    let users = get_entry(
        &info.0,
        &info.1,
        &mut payload.numbers,
        pool,
        &query.access_token,
        auth,
    )
    .map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(users);

    set_response_headers(&mut res);

    Ok(res)
}
