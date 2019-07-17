use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;

use crate::errors::ServiceError;
use crate::models::{Pool, User};
use crate::utils::phonenumber_to_international;

use crate::models::Blacklist;

#[derive(Debug, Deserialize)]
pub struct PostData {
    blocked: String
}

pub fn add(
    info: web::Path<(String, String)>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    web::block(move || create_entry(&info.0, &info.1, &data.into_inner(), pool)).then(|res| match res {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn delete(
    info: web::Path<(String, String)>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    web::block(move || delete_entry(&info.0, &info.1, &data.into_inner(), pool)).then(|res| match res {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

fn create_entry(
    blocker: &String,
    country_code: &String,
    data: &PostData,
    pool: web::Data<Pool>,
) -> Result<Blacklist, crate::errors::ServiceError> {
    let blocked = &data.blocked;
    let b = create_query(blocker, country_code, blocked, pool)?;
    dbg!(&b);
    Ok(b)
}

fn create_query(
    blocker: &String,
    country_code: &String,
    blocked: &String,
    pool: web::Data<Pool>,
) -> Result<Blacklist, crate::errors::ServiceError> {
    use crate::schema::blacklist::dsl::blacklist;

    let new_inv: Blacklist = Blacklist::my_from(blocker, blocked);
    let conn: &PgConnection = &pool.get().unwrap();

    let ins = diesel::insert_into(blacklist)
        .values(&new_inv)
        .get_result(conn)?;

    dbg!(&ins);

    Ok(ins)
}

fn delete_entry(blocker: &String, country_code: &String, data: &PostData, pool: web::Data<Pool>) -> Result<(), crate::errors::ServiceError> {
    let blocked = &data.blocked;
    delete_query(blocker, country_code, blocked, pool)
   // dbg!(&b);
}

fn delete_query(sblocker: &String, country_code: &String, sblocked: &String, pool: web::Data<Pool>) -> Result<(), crate::errors::ServiceError> {
    use crate::schema::blacklist::dsl::{blacklist, blocker, blocked};

    let conn: &PgConnection = &pool.get().unwrap();

    let target = blacklist.filter(blocker.eq(sblocker)).filter(blocked.eq(sblocked));

    diesel::delete(target)
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot delete blacklist".into()))?;

    Ok(())
}
