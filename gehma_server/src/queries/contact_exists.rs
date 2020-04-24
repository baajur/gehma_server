use actix_web::web;
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::dao::*;
use core::models::dto::*;

//use crate::routes::contact_exists::{PayloadUser};

use crate::Pool;

use crate::persistence::contact_exists::PersistentContactExistsDao;
use log::info;

pub const MAX_ALLOWED_CONTACTS: usize = 10000;
//pub const MIN_TELE_NUM_LENGTH: usize = 3;

#[derive(Clone)]
pub struct PgContactExistsDao {
    pub pool: Pool,
}

impl PersistentContactExistsDao for PgContactExistsDao {
    fn get(
        &self,
        uid: &Uuid,
        _user: &UserDto,
        phone_numbers: &mut Vec<PayloadUserDto>,
        _country_code: &str,
    ) -> Result<Vec<WrappedUserDto>, ServiceError> {
        info!("queries/push_notification/get_query");
        use core::schema::blacklist::dsl::{blacklist, hash_blocked, hash_blocker};
        use core::schema::users::dsl::{changed_at, hash_tele_num, id, users};

        let conn: &PgConnection = &self.pool.get().unwrap();

        if phone_numbers.is_empty() {
            return Ok(Vec::new());
        }

        if phone_numbers.len() == MAX_ALLOWED_CONTACTS {
            return Err(ServiceError::BadRequest("Too many contacts".into()));
        }

        let mut numbers: Vec<WrappedUserDto> = phone_numbers
            .iter_mut()
            .map(|w| WrappedUserDto {
                hash_tele_num: w.hash_tele_num.clone(),
                name: w.name.clone(),
                user: None,
            })
            .collect();

        users
            .filter(id.eq(uid)) // 1. Get user
            .load::<UserDao>(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
            .and_then(|result| {
                if let Some(user) = result.first() {
                    // 2. Get all blacklist
                    blacklist
                        .filter(
                            hash_blocked
                                .eq(&user.hash_tele_num)
                                .or(hash_blocker.eq(&user.hash_tele_num)),
                        )
                        .load::<BlacklistDao>(conn)
                        .map_err(|_db_error| {
                            ServiceError::BadRequest("Cannot find blacklists".into())
                        })
                        .and_then(|lists| {
                            let people_who_blacklisted_user: Vec<_> = lists
                                .into_iter()
                                .map(|w| {
                                    if w.hash_blocker == user.hash_tele_num {
                                        // 3. get the appropriate
                                        w.hash_blocked //jener der blockiert soll sie auch nicht sehen
                                    } else {
                                        w.hash_blocker
                                    }
                                })
                                .collect();

                            users // 4. Get all contacts which were defined in the request array
                                .filter(
                                    hash_tele_num.eq_any(
                                        numbers
                                            .iter()
                                            .map(|w| w.hash_tele_num.clone())
                                            .collect::<Vec<String>>(),
                                    ),
                                )
                                .order(changed_at.desc())
                                .load::<UserDao>(conn)
                                .map_err(|_db_error| {
                                    ServiceError::BadRequest("Invalid User".into())
                                })
                                .and_then(|mut result| {
                                    //i are contacts
                                    for i in result.iter_mut() {
                                        let mut res: Vec<_> = numbers
                                            .iter_mut()
                                            .filter(|w| w.hash_tele_num == i.hash_tele_num) // 5. downgrade user
                                            .collect();

                                        if let Some(mut res_user) = res.first_mut() {
                                            res_user.user = Some(i.clone().into());
                                        }
                                    }

                                    // 6. reset blacklisted

                                    numbers
                                        .iter_mut()
                                        .filter(|w| {
                                            people_who_blacklisted_user.contains(&w.hash_tele_num)
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
                                        .collect::<Vec<WrappedUserDto>>())
                                })
                                .and_then(|numbers| {
                                    use core::schema::contacts::dsl::{contacts, from_id};

                                    let user_contacts: Vec<_> = numbers
                                        .iter()
                                        .map(|n| {
                                            ContactDao::my_from(
                                                n.name.clone(),
                                                &user,
                                                n.user.as_ref().unwrap().tele_num.clone(),
                                            )
                                        })
                                        .collect();

                                    let target = contacts.filter(from_id.eq(user.id));

                                    //We need to delete all numbers, because
                                    //user shall not receive push notifications
                                    //for contacts he deleted
                                    let _ = diesel::delete(target).execute(conn).map_err(
                                        |_db_err| {
                                            eprintln!("{}", _db_err);
                                            ServiceError::BadRequest("Could reset contacts".into())
                                        },
                                    )?;

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
}

/*
pub(crate) fn get_query(
    uid: Uuid,
    user: &User,
    phone_numbers: &mut Vec<PayloadUser>,
    _country_code: &str,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, ServiceError> {
    info!("queries/push_notification/get_query");
    use core::models::Contact;
    use core::schema::blacklist::dsl::{blacklist, hash_blocked, hash_blocker};
    use core::schema::users::dsl::{
        access_token, changed_at, client_version, country_code, created_at, description,
        firebase_token, hash_tele_num, id, led, profile_picture, tele_num, users,
    };

    let conn: &PgConnection = &pool.get().unwrap();

    if phone_numbers.is_empty() {
        return Ok(Vec::new());
    }

    if phone_numbers.len() == MAX_ALLOWED_CONTACTS {
        return Err(ServiceError::BadRequest("Too many contacts".into()));
    }

    let mut numbers: Vec<ResponseUser> = phone_numbers
        .iter_mut()
        .map(|w| ResponseUser {
            hash_tele_num: w.hash_tele_num.clone(),
            name: w.name.clone(),
            user: None,
        })
        .collect();

    users
        .filter(id.eq(uid))
        .left_join(
            blacklist.on(hash_tele_num
                .eq(hash_blocked)
                .and(hash_blocker.eq(&user.hash_tele_num))
                .or(hash_tele_num
                    .eq(hash_blocker)
                    .and(hash_blocked.eq(&user.hash_tele_num)))),
        )
        //.filter(hash_blocked.is_null().or(hash_blocker.is_null()))
        .select((
            id,
            tele_num,
            led,
            created_at,
            country_code,
            description,
            changed_at,
            client_version,
            firebase_token,
            profile_picture,
            access_token,
            hash_tele_num,
        ))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            if let Some(user) = result.first() {
                //only the first row is required

                users
                    .filter(
                        hash_tele_num.eq_any(
                            phone_numbers
                                .iter()
                                .map(|w| &w.hash_tele_num)
                                .collect::<Vec<&String>>(),
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
                                .filter(|w| w.hash_tele_num == *i.hash_tele_num) //TODO #34
                                .collect();

                            if let Some(mut res_user) = res.first_mut() {
                                res_user.user = Some(i.downgrade());
                            }
                        }

                        Ok(numbers
                            .into_iter()
                            .filter(|w| w.user.is_some())
                            .collect::<Vec<ResponseUser>>())
                    })
                    .and_then(|numbers| {
                        use core::schema::contacts::dsl::{contacts, from_id};

                        let user_contacts: Vec<_> = numbers
                            .iter()
                            .map(|n| {
                                Contact::my_from(
                                    n.name.clone(),
                                    &user,
                                    n.user.as_ref().unwrap().tele_num.clone(),
                                )
                            })
                            .collect();

                        let target = contacts.filter(from_id.eq(user.id));

                        //We need to delete all numbers, because
                        //user shall not receive push notifications
                        //for contacts he deleted
                        let _ = diesel::delete(target).execute(conn).map_err(|_db_err| {
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
            } else {
                Ok(vec![])
            }
        })
}
*/
