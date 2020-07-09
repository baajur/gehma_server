use actix_web::{web, HttpRequest, HttpResponse};
use core::errors::ServiceError;

use crate::controllers::broadcast::*;
use crate::queries::*;

use web_contrib::utils::set_response_headers;

use log::info;

#[derive(Deserialize)]
pub struct Info {
    mark_seen: bool,
}

pub async fn get_all(
    _req: HttpRequest,
    info: web::Path<String>,
    mark_seen: web::Query<Info>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    contact_dao: web::Data<Box<dyn PersistentContactsDao>>,
) -> Result<HttpResponse, ServiceError> {
    info!("fn get_all()");

    let mark_seen: bool = mark_seen.into_inner().mark_seen;

    let elements = get_entries(&info.into_inner(), user_dao, contact_dao, mark_seen)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(elements);

    set_response_headers(&mut res);

    Ok(res)

    //response!(elements)
}
