use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;

use crate::errors::ServiceError;
use crate::models::{Pool, User};

#[derive(Deserialize)]
pub struct Payload {
    pub numbers: Vec<String>
}

pub fn get(
    base_tel: web::Path<String>,
    payload : web::Json<Payload>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&payload.numbers);
    web::block(move || get_entry(&payload.numbers, pool)).then(|res| match res {
        Ok(users) => Ok(HttpResponse::Ok().json(&users)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

fn get_entry(
    phone_numbers: &Vec<String>,
    pool: web::Data<Pool>,
) -> Result<Vec<User>, crate::errors::ServiceError> {
    let users = get_query(phone_numbers, pool)?;
    dbg!(&users);

    Ok(users)
}

fn get_query(
    phone_numbers: &Vec<String>,
    pool: web::Data<Pool>,
) -> Result<Vec<User>, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{tele_num, users};

    let conn: &PgConnection = &pool.get().unwrap();

    if phone_numbers.len() == 0 {
        return Ok(Vec::new());
    }

    //let mut filter = users.filter(tele_num.eq(phone_numbers.iter().take(1).collect::<Vec<_>>()[0]));

    /*
    for i in phone_numbers.iter().skip(1) {
        filter = filter.or_filter(tele_num.eq(i));
    }
    */

    users
        .filter(tele_num.eq_any(phone_numbers))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}
