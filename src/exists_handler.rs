use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use crate::errors::{ServiceError};
use crate::models::{Blacklist, Pool, User};

const MAX_ALLOWED_CONTACTS : usize = 10000;
const MIN_TELE_NUM_LENGTH : usize = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseUser {
    pub calculated_tele: String,
    pub old: String,
    pub name: String,
    pub user: Option<User>,
}

#[derive(Debug, Deserialize)]
pub struct Payload {
    pub numbers: Vec<PayloadUser>,
}

#[derive(Debug, Deserialize)]
pub struct PayloadUser {
    pub name: String,
    pub tele_num: String
}


pub fn get(
    info: web::Path<(String, String)>,
    mut payload: web::Json<Payload>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    //dbg!(&payload);
    web::block(move || {
        let info = info.into_inner();
        get_entry(&info.0, &info.1, &mut payload.numbers, pool)
    })
    .then(|res| match res {
        Ok(users) => {
            let mut res = HttpResponse::Ok().content_type("application/json").json(users);
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        },
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

fn get_entry(
    uid: &String,
    country_code: &String,
    phone_numbers: &mut Vec<PayloadUser>,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, crate::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let users = get_query(parsed, phone_numbers, country_code, pool)?;

    Ok(users)
}

fn get_query(
    uid: Uuid,
    phone_numbers: &mut Vec<PayloadUser>,
    country_code: &String,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, crate::errors::ServiceError> {
    use crate::models::PhoneNumber;
    use crate::schema::blacklist::dsl::{blacklist, blocked, blocker};
    use crate::schema::users::dsl::{id, tele_num, users, changed_at};

    let conn: &PgConnection = &pool.get().unwrap();

    if phone_numbers.len() == 0 {
        return Ok(Vec::new());
    }

    if phone_numbers.len() == MAX_ALLOWED_CONTACTS {
        return Err(ServiceError::BadRequest("Too many contacts".into()));
    }

    let mut numbers: Vec<ResponseUser> = phone_numbers
        .into_iter()
        .filter(|w| w.tele_num.len() > MIN_TELE_NUM_LENGTH)
        .filter_map(|w| match PhoneNumber::my_from(&w.tele_num, country_code) {
            Ok(number) => Some(ResponseUser {
                calculated_tele: number.to_string(),
                old: w.tele_num.clone(),
                user: None,
                name: w.name.clone(),
            }),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        })
        .collect();

    users
        .filter(id.eq(uid))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            if let Some(user) = result.first() {
                blacklist
                    .filter(blocked.eq(&user.tele_num).or(blocker.eq(&user.tele_num)))
                    .load::<Blacklist>(conn)
                    .map_err(|_db_error| ServiceError::BadRequest("Cannot find blacklists".into()))
                    .and_then(|lists| {
                        let people_who_blacklisted_user: Vec<_> = lists
                            .into_iter()
                            .map(|w| match w.blocker == user.tele_num {
                                true => w.blocked.clone(), //jener der blockiert soll sie auch nicht sehen
                                false => w.blocker.clone(),
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
                            .order(changed_at.desc())
                            .load::<User>(conn)
                            .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
                            .and_then(|mut result| {
                                //i are contacts
                                for i in result.iter_mut() {
                                    let mut res: Vec<_> = numbers
                                        .iter_mut()
                                        .filter(|w| w.calculated_tele == i.tele_num)
                                        .collect();

                                    if let Some(mut res_user) = res.first_mut() {
                                        res_user.user = Some(i.clone());
                                    }
                                }

                                numbers
                                    .iter_mut()
                                    .filter(|w| {
                                        people_who_blacklisted_user.contains(&w.calculated_tele)
                                    })
                                    .for_each(|ref mut w| match &mut w.user {
                                        Some(ref mut u) => {
                                            u.led = false; //Ignoring happens here
                                            u.description = String::new();

                                            //TODO to cross self blocked users cross here the
                                            //people
                                        },
                                        None => {}
                                    });

                                Ok(numbers.into_iter().filter(|w| w.user.is_some()).collect())
                                //Err(ServiceError::BadRequest("Invalid Invitation".into()))
                            })
                    })
            } else {
                Err(ServiceError::BadRequest("No user found".into()))
            }
        })
}
