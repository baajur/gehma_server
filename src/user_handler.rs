use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;

use crate::errors::ServiceError;
use crate::models::{User, Pool};

pub fn add(base_tel: web::Path<String>, pool: web::Data<Pool>) -> 
impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || create_entry(base_tel.into_inner(), pool)).then(|res| { 
        match res {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError)
            }
        }
    })
}

pub fn get(base_tel: web::Path<String>, pool: web::Data<Pool>) -> 
impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || get_entry(base_tel.into_inner(), pool)).then(|res| { 
        match res {
            Ok(users) => Ok(HttpResponse::Ok().json(&users)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError)
            }
        }
    })
}

fn create_entry(tel: String, pool: web::Data<Pool>) -> Result<(), crate::errors::ServiceError> {
    let _ = dbg!(create_query(tel, pool)?);
    Ok(())
}

fn get_entry(tele_num: String, pool: web::Data<Pool>) -> Result<Vec<User>, crate::errors::ServiceError> {
    let users = get_query(tele_num, pool)?;
    dbg!(&users);

    Ok(users)
}

fn create_query(tel: String, pool: web::Data<Pool>) -> Result<User, crate::errors::ServiceError> {
    use crate::schema::users::dsl::users;

    let new_inv : User = tel.into();
    let conn : &PgConnection = &pool.get().unwrap();

    let ins = diesel::insert_into(users)
        .values(&new_inv)
        .get_result(conn)?;

    Ok(ins)
}

fn get_query(para_num: String, pool: web::Data<Pool>) -> Result<Vec<User>, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{tele_num, users};

    let conn : &PgConnection = &pool.get().unwrap();

    users.filter(tele_num.eq(para_num))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })

}
