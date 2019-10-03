use crate::Pool;
use ::core::errors::ServiceError;
use ::core::models::{Analytic, PhoneNumber, UsageStatisticEntry, User, Blacklist};
use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use serde_json::json;
use tokio;
use uuid::Uuid;

use log::{info, debug, error};

pub fn add(
    _info: web::Path<()>,
    body: web::Json<PostUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/add");
    debug!("{:?}", body);
    //dbg!(&body);
    web::block(move || create_entry(body.into_inner(), pool)).then(|res| match res {
        Ok(user) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(user);
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn get(
    info: web::Path<(String)>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/get");
    debug!("path {:?}", info);

    web::block(move || get_entry(&info.into_inner(), pool)).then(|res| match res {
        Ok(users) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(users);
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostUser {
    pub tele_num: String,
    pub country_code: String,
    pub client_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub description: String,
    pub led: String,
    pub is_autofahrer: Option<String>,
    pub client_version: String,
}

pub fn update(
    info: web::Path<(String)>,
    data: web::Json<UpdateUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/update");
    debug!("path {:?}", info);
    debug!("data {:?}", data);

    web::block(move || update_user(&info.into_inner(), &data.into_inner(), &pool)).then(|res| {
        match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(&user)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}

fn create_entry(
    body: PostUser,
    pool: web::Data<Pool>,
) -> Result<User, ServiceError> {
    info!("controllers/user/create_entry");
    debug!("body {:?}", body);

    if !crate::ALLOWED_CLIENT_VERSIONS.contains(&body.client_version.as_str()) {
        error!("Version mismatch. Server does not suppoert client version");
        return Err(ServiceError::BadRequest(format!(
            "Version mismatch. The supported versions are {:?}",
            crate::ALLOWED_CLIENT_VERSIONS
        )));
    }

    let tele = &body.tele_num;
    let country_code = &body.country_code;

    let tele2 = PhoneNumber::my_from(&tele, country_code)?;

    dbg!(&tele2.to_string());

    let user = match crate::queries::user::create_query(&tele2, &country_code, &body.client_version, &pool) {
        Ok(u) => Ok(u),
        Err(ServiceError::AlreadyExists(_)) => crate::queries::user::get_entry_by_tel_query(&tele2, &pool),
        Err(err) => Err(err),
    }?;

    if user.client_version != body.client_version {
        update_user(
            &user.id.to_string(),
            &UpdateUser {
                description: user.description.clone(),
                led: format!("{}", user.led),
                is_autofahrer: Some(format!("{}", user.is_autofahrer)),
                client_version: body.client_version.clone(),
            },
            &pool,
        )?;
    }

    dbg!(&user);

    crate::queries::user::analytics_usage_statistics(&pool, &user)?;

    Ok(user)
}

fn get_entry(uid: &String, pool: web::Data<Pool>) -> Result<User, ::core::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let users = crate::queries::user::get_query(parsed, &pool)?;
    dbg!(&users);

    let user = match users.len() {
        0 => Err(ServiceError::BadRequest("No user found".to_string())),
        _ => Ok(users.get(0).unwrap().clone()),
    }?;

    //analytics_usage_statistics(&pool, &user)?; not logging every refresh

    Ok(user)
}

fn update_user(
    uid: &String,
    user: &UpdateUser,
    pool: &web::Data<Pool>,
) -> Result<User, ::core::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let user = crate::queries::user::update_user_query(parsed, user, &pool)?;

    dbg!(&user);

    crate::queries::user::analytics_user(&pool, &user)?;

    Ok(user)
}
