use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;

use crate::errors::ServiceError;
use crate::models::{User, Pool};

pub fn get_contacts(base_tel: web::Path<String>, pool: web::Data<Pool>) -> 
impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || get(base_tel.into_inner(), pool)).then(|res| { 
        match res {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError)
            }
        }
    })
}

fn get(tele_num: String, pool: web::Data<Pool>) -> Result<Vec<User>, crate::errors::ServiceError> {
    Ok(dbg!(query(tele_num, pool)?))
}

fn query(para_num: String, pool: web::Data<Pool>) -> Result<Vec<User>, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{tele_num, users};

    let conn : &PgConnection = &pool.get().unwrap();

    users.filter(tele_num.eq(para_num))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid Invitation".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })

}
