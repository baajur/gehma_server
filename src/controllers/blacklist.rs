use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use ::core::errors::ServiceError;
use ::core::models::{Blacklist, PhoneNumber, User};

use crate::Pool;

use log::{debug, info};

#[derive(Debug, Deserialize)]
pub struct GetAllData {
    numbers: Vec<String>,
}

pub fn get_all(
    info: web::Path<(String)>,
    //data: web::Json<GetAllData>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/get_all");
    debug!("path {:?}", info);

    dbg!(&info);
    let info = info.into_inner();
    web::block(move || get_entry(&info, pool)).then(|res| match res {
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

#[derive(Debug, Deserialize)]
pub struct PostData {
    blocked: String,
    country_code: String,
}

pub fn add(
    info: web::Path<(String)>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/add");
    debug!("path {:?}", info);
    debug!("body {:?}", data);

    web::block(move || create_entry(&info.into_inner(), &data.into_inner(), pool)).then(|res| {
        match res {
            Ok(_) => {
                let mut res = HttpResponse::Ok().content_type("application/json").finish();
                crate::utils::set_response_headers(&mut res);
                Ok(res)
            }
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}

pub fn delete(
    info: web::Path<(String)>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/delete");
    debug!("path {:?}", info);
    debug!("body {:?}", data);

    web::block(move || delete_entry(&info.into_inner(), &data.into_inner(), pool)).then(|res| {
        match res {
            Ok(_) => {
                let mut res = HttpResponse::Ok().content_type("application/json").finish();
                crate::utils::set_response_headers(&mut res);
                Ok(res)
            }
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}

fn get_entry(
    blocker: &String,
    pool: web::Data<Pool>,
) -> Result<Vec<Blacklist>, ServiceError> {
    let blocker = Uuid::parse_str(blocker)?;

    let bl = crate::queries::blacklist::get_query(blocker, pool)?;

    dbg!(&bl);

    Ok(bl)
}

fn create_entry(
    blocker: &String,
    data: &PostData,
    pool: web::Data<Pool>,
) -> Result<Blacklist, ServiceError> {
    use ::core::schema::users::dsl::{id, users};

    let blocker2 = Uuid::parse_str(blocker)?;
    let blocked = PhoneNumber::my_from(&data.blocked, &data.country_code)?;

    dbg!(&blocked);
    dbg!(&blocker);

    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .map(|w| w.clone())
        .ok_or(ServiceError::BadRequest(
            "No user found with given uid".into(),
        ))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;
    let b = crate::queries::blacklist::create_query(&tel, &blocked, pool)?;

    dbg!(&b);
    Ok(b)
}

fn delete_entry(
    blocker: &String,
    data: &PostData,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use ::core::schema::users::dsl::{id, users};

    let blocker2 = Uuid::parse_str(blocker)?;
    let blocked = PhoneNumber::my_from(&data.blocked, &data.country_code)?;

    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .map(|w| w.clone())
        .ok_or(ServiceError::BadRequest(
            "No user found with given uid".into(),
        ))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;

    crate::queries::blacklist::delete_query(&tel, &blocked, pool)
}
