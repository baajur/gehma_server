use web_contrib::auth::Auth;
use actix_web::{web, HttpResponse};
use web_contrib::utils::{QueryParams, set_response_headers};
use crate::Pool;
use log::{info};
use crate::controllers::blacklist::{create_entry, get_entry, delete_entry};
use core::errors::ServiceError;

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
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/get_all");

    let info = info.into_inner();


    let users = get_entry(&info, pool, &query.access_token, auth).map_err(|_err| ServiceError::InternalServerError)?;

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
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/add");

    create_entry(
            &info.into_inner(),
            &data.into_inner(),
            pool,
            &query.access_token,
            auth,
        ).map_err(|_err| ServiceError::InternalServerError)?;

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
) -> Result<HttpResponse, ServiceError> {
    info!("controllers/blacklist/delete");

    delete_entry(
            &info.into_inner(),
            &data.into_inner(),
            pool,
            &query.access_token,
            auth,
        ).map_err(|_err| ServiceError::InternalServerError)?;

    let mut res = HttpResponse::Ok().content_type("application/json").finish();
    set_response_headers(&mut res);

    Ok(res)
}
