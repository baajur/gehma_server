use actix_web::web;
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::{Blacklist, User};

use crate::routes::contact_exists::{PayloadUser, ResponseUser};

use crate::Pool;

use log::{error, info};

pub const MAX_ALLOWED_CONTACTS: usize = 10000;
pub const MIN_TELE_NUM_LENGTH: usize = 3;

pub(crate) fn get_query(
    uid: Uuid,
    user: &User,
    phone_numbers: &mut Vec<PayloadUser>,
    country_code: &str,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, ServiceError> {
    info!("queries/push_notification/get_query");
    use core::models::Contact;
    use core::models::PhoneNumber;
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
        .filter(hash_blocked.is_null().or(hash_blocker.is_null()))
        .select((
            id,
            tele_num,
            led,
            created_at,
            country_code,
            description,
            changed_at,
            client_version,
            profile_picture,
            firebase_token,
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
                                .filter(|w| w.hash_tele_num == *i.hash_tele_num.as_ref().unwrap()) //TODO #34
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

                        let res = diesel::insert_into(contacts)
                            .values(user_contacts)
                            //.on_conflict_do_nothing()
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
