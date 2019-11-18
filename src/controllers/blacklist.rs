use crate::auth::FirebaseDatabaseConfiguration;
use actix_web::HttpRequest;
use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::{Blacklist, PhoneNumber, User};
use crate::utils::QueryParams;

use crate::Pool;

use log::{debug, info};

#[derive(Debug, Deserialize)]
pub struct GetAllData {
    numbers: Vec<String>,
}

pub fn get_all(
    req: HttpRequest,
    info: web::Path<(String)>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/get_all");

    let info = info.into_inner();
    web::block(move || get_entry(&info, pool, &query.firebase_uid, firebase_config)).then(
        |res| match res {
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
        },
    )
}

#[derive(Debug, Deserialize)]
pub struct PostData {
    blocked: String,
    country_code: String,
}

pub fn add(
    req: HttpRequest,
    info: web::Path<(String)>,
    data: web::Json<PostData>,
    query: web::Query<QueryParams>,
    pool: web::Data<Pool>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/add");

    web::block(move || {
        create_entry(
            &info.into_inner(),
            &data.into_inner(),
            pool,
            &query.firebase_uid,
            firebase_config,
        )
    })
    .then(|res| match res {
        Ok(_) => {
            let mut res = HttpResponse::Ok().content_type("application/json").finish();
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn delete(
    req: HttpRequest,
    info: web::Path<(String)>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/blacklist/delete");

    web::block(move || {
        delete_entry(
            &info.into_inner(),
            &data.into_inner(),
            pool,
            &query.firebase_uid,
            firebase_config,
        )
    })
    .then(|res| match res {
        Ok(_) => {
            let mut res = HttpResponse::Ok().content_type("application/json").finish();
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

fn get_entry(
    blocker: &str,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    firebase_configuration: web::Data<FirebaseDatabaseConfiguration>,
) -> Result<Vec<Blacklist>, ServiceError> {
    let blocker = Uuid::parse_str(blocker)?;

    let user : Result<User, ServiceError> = authenticate_user_by_uid!(
        blocker,
        &firebase_uid,
        firebase_configuration.into_inner(),
        &pool
    );

    user?;

    let bl = crate::queries::blacklist::get_query(blocker, pool)?;

    Ok(bl)
}

fn create_entry(
    blocker: &str,
    data: &PostData,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> Result<Blacklist, ServiceError> {
    use core::schema::users::dsl::{id, users};

    let blocker2 = Uuid::parse_str(blocker)?;

    let user : Result<User, ServiceError> = authenticate_user_by_uid!(
        blocker2,
        &firebase_uid,
        firebase_config.into_inner(),
        &pool
    );

    user?;

    let blocked = PhoneNumber::my_from(&data.blocked, &data.country_code)?;

    //dbg!(&blocked);
    //dbg!(&blocker);

    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found with given uid".into()))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;
    let b = crate::queries::blacklist::create_query(&tel, &blocked, pool)?;

    //dbg!(&b);
    Ok(b)
}

fn delete_entry(
    blocker: &str,
    data: &PostData,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    firebase_configuration: web::Data<FirebaseDatabaseConfiguration>,
) -> Result<(), ServiceError> {
    use core::schema::users::dsl::{id, users};

    let blocker2 = Uuid::parse_str(blocker)?;

    let user : Result<User, ServiceError>  = authenticate_user_by_uid!(
        blocker2,
        &firebase_uid,
        firebase_configuration.into_inner(),
        &pool
    );

    user?;

    let blocked = PhoneNumber::my_from(&data.blocked, &data.country_code)?;

    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found with given uid".into()))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;

    crate::queries::blacklist::delete_query(&tel, &blocked, pool)
}
