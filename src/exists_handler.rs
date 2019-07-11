use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;

use crate::errors::ServiceError;
use crate::models::{Pool, User};

#[derive(Deserialize)]
pub struct Payload {
    pub numbers: Vec<String>,
}

pub fn get(
    base_tel: web::Path<String>,
    mut payload: web::Json<Payload>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || get_entry(&mut payload.numbers, pool)).then(|res| match res {
        Ok(users) => Ok(HttpResponse::Ok().json(&users)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

fn get_entry(
    phone_numbers: &mut Vec<String>,
    pool: web::Data<Pool>,
) -> Result<Vec<User>, crate::errors::ServiceError> {
    let users = get_query(phone_numbers, pool)?;

    Ok(users)
}

fn get_query(
    phone_numbers: &mut Vec<String>,
    pool: web::Data<Pool>,
) -> Result<Vec<User>, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{tele_num, users};

    let conn: &PgConnection = &pool.get().unwrap();

    if phone_numbers.len() == 0 {
        return Ok(Vec::new());
    }

    let numbers: Vec<String> = phone_numbers
        .iter_mut()
        .filter(|w| w.len() > 3)
        .map(|w| w.trim().replace("+", "").replace(" ", ""))
        .collect();

    dbg!(&numbers);

    users
        .filter(tele_num.eq_any(numbers))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}
