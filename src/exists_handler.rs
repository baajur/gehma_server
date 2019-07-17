use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use crate::errors::{InternalError, ServiceError};
use crate::models::{Pool, User};
use crate::utils::phonenumber_to_international;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseUser {
    pub calculated_tele: String,
    pub old: String,
    pub user: Option<User>,
}

#[derive(Debug, Deserialize)]
pub struct Payload {
    pub numbers: Vec<String>,
    pub country_code: String,
}

pub fn get(
    info: web::Path<(String)>,
    mut payload: web::Json<Payload>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    //dbg!(&payload);
    web::block(move || {
        get_entry(
            &info.into_inner(),
            &payload.country_code.clone(),
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
    uid: &String,
    country_code: &String,
    phone_numbers: &mut Vec<String>,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, crate::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let users = get_query(phone_numbers, country_code, pool)?;

    Ok(users)
}

fn get_query(
    phone_numbers: &mut Vec<String>,
    country_code: &String,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, crate::errors::ServiceError> {
    use crate::models::PhoneNumber;
    use crate::schema::users::dsl::{tele_num, users};

    let conn: &PgConnection = &pool.get().unwrap();

    if phone_numbers.len() == 0 {
        return Ok(Vec::new());
    }

    let mut numbers: Vec<ResponseUser> = phone_numbers
        .into_iter()
        .filter(|w| w.len() > 3)
        .filter_map(|w| match PhoneNumber::my_from(w, country_code) {
            Ok(number) => Some(ResponseUser {
                calculated_tele: number.to_string(),
                old: w.to_string(),
                user: None,
            }),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        })
        .collect();

    users
        .filter(
            tele_num.eq_any(
                numbers
                    .iter_mut()
                    .map(|w| w.calculated_tele.clone())
                    .collect::<Vec<String>>(),
            ),
        )
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|mut result| {
            //I is user
            for i in result.iter_mut() {
                let mut res: Vec<_> = numbers
                    .iter_mut()
                    .filter(|w| w.calculated_tele == i.tele_num)
                    .collect();

                if let Some(mut res_user) = res.first_mut() {
                    res_user.user = Some(i.clone());
                }
            }

            Ok(numbers.into_iter().filter(|w| w.user.is_some()).collect())
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}
