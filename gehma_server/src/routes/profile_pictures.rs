use crate::queries::*;
use crate::controllers::profile_pictures::*;
use actix_web::{web, HttpResponse};
use core::errors::ServiceError;
use log::info;
use web_contrib::utils::set_response_headers;

pub async fn get_all(
    info: web::Path<String>,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    p_dao: web::Data<Box<dyn PersistentProfilePictureDao>>,
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/get_all");

    let info = info.into_inner();

    let users = get_all_profile_pictures(&info, user_dao, p_dao)?;

    let mut res = HttpResponse::Ok()
        .content_type("application/json")
        .json(users);

    set_response_headers(&mut res);

    Ok(res)
}
