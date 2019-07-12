use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;

use crate::errors::ServiceError;
use crate::models::{Pool, User};
use crate::utils::phonenumber_to_international;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseUser { 
    pub calculated_tele: String,
    pub old: String,
    pub user: Option<User> 
}

#[derive(Deserialize)]
pub struct Payload {
    pub numbers: Vec<String>,
}

pub fn get(
    info: web::Path<(String, String)>,
    mut payload: web::Json<Payload>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || {
        get_entry(
            &info.0,
            &info.1,
            &mut payload.numbers,
            pool,
        )
    })
    .then(|res| match res {
        Ok(users) => Ok(HttpResponse::Ok().json(&users)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

fn get_entry(
    tel: &String,
    country_code: &String,
    phone_numbers: &mut Vec<String>,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, crate::errors::ServiceError> {
    let users = get_query(phone_numbers, country_code, pool)?;

    Ok(users)
}

fn get_query(
    phone_numbers: &mut Vec<String>,
    country_code: &String,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{tele_num, users};

    let conn: &PgConnection = &pool.get().unwrap();

    if phone_numbers.len() == 0 {
        return Ok(Vec::new());
    }

    let mut numbers: Vec<ResponseUser> = phone_numbers
        .into_iter()
        .filter(|w| w.len() > 3)
        .map(|w| ResponseUser {
            calculated_tele: phonenumber_to_international(w, &country_code).replace("+", ""),
            old: w.to_string(),
            user: None
        })
        .collect();
        //.filter(tele_num.eq(phonenumber_to_international(&para_num).replace("+", "")))

    dbg!(&numbers);

    users
        .filter(tele_num.eq_any(numbers.iter_mut().map(|w| w.calculated_tele.clone()).collect::<Vec<String>>()))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|mut result| {
            //I is user
            for i in result.iter_mut() {
                let mut res : Vec<_> = numbers.iter_mut().filter(|w| w.calculated_tele == i.tele_num).collect();

                if let Some(mut res_user) = res.first_mut() {
                    res_user.user = Some(i.clone()); 
                }
            }

            Ok(numbers.into_iter().filter(|w| w.user.is_some()).collect())
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}
