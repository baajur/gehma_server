use actix_web::web;
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use ::core::errors::ServiceError;
use ::core::models::{Blacklist, User};

use crate::controllers::contact_exists::{PayloadUser, ResponseUser, MAX_ALLOWED_CONTACTS, MIN_TELE_NUM_LENGTH};

use crate::Pool;

use log::{info, error};

pub(crate) fn get_query(
    uid: Uuid,
    phone_numbers: &mut Vec<PayloadUser>,
    country_code: &String,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, ServiceError> {
    info!("queries/push_notification/get_query");
    use ::core::models::Contact;
    use ::core::models::PhoneNumber;
    use ::core::schema::blacklist::dsl::{blacklist, blocked, blocker};
    use ::core::schema::users::dsl::{changed_at, id, tele_num, users};

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
                error!("{}", err);
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
                                        res_user.user = Some(i.downgrade());
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
                                        }
                                        None => {}
                                    });

                                Ok(numbers
                                    .into_iter()
                                    .filter(|w| w.user.is_some())
                                    .collect::<Vec<ResponseUser>>())
                            })
                            .and_then(|numbers| {
                                use ::core::schema::contacts::dsl::{contacts, from_id};

                                let user_contacts: Vec<_> = numbers
                                    .iter()
                                    .map(|n| {
                                        Contact::my_from(
                                            n.name.clone(),
                                            &user,
                                            n.calculated_tele.clone(),
                                        )
                                    })
                                    .collect();

                                let target = contacts.filter(from_id.eq(user.id));

                                //We need to delete all numbers, because
                                //user shall not receive push notifications
                                //for contacts he deleted
                                let _ =
                                    diesel::delete(target).execute(conn).map_err(|_db_err| {
                                        eprintln!("{}", _db_err);
                                        ServiceError::BadRequest("Could reset contacts".into())
                                    })?;

                                let _ = diesel::insert_into(contacts)
                                    .values(user_contacts)
                                    .on_conflict_do_nothing()
                                    .execute(conn)
                                    .map_err(|_db_err| {
                                        eprintln!("{}", _db_err);
                                        ServiceError::BadRequest("Could set contacts".into())
                                    })?;

                                Ok(numbers)
                            })
                    })
            } else {
                Err(ServiceError::BadRequest("No user found".into()))
            }
        })
}
