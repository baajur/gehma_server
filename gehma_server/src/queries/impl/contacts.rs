use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use core::errors::{InternalServerError, ServiceError};
use core::models::dao::*;
use core::models::dto::*;

use crate::Pool;

use crate::queries::*;
use log::{error, info};

#[derive(Clone)]
pub struct PgContactsDao {
    pub pool: Pool,
}

impl PersistentContactsDao for PgContactsDao {
    ///users which should be added, already filtered after blacklist contacts: Vec<ContactDto>,
    fn create<'a>(
        &self,
        uid: &Uuid,
        _user: &UserDao,
        payload: &'a Vec<&'a mut PayloadUserDto>,
    ) -> Result<(), ServiceError> {
        info!("queries/contacts/create");
        use core::schema::contacts::dsl::{contacts, target_hash_tele_num};

        let conn: &PgConnection = &self.pool.get().unwrap();

        if payload.is_empty() {
            return Ok(());
        }

        let inserts: Vec<_> = payload
            .into_iter()
            .map(|w| ContactInsertDao {
                from_id: uid.clone(),
                target_hash_tele_num: w.hash_tele_num.clone(),
                name: w.name.clone(),
                created_at: chrono::Utc::now().naive_local(),
            })
            .collect();

        for phone_contact in inserts {
            let _ = diesel::insert_into(contacts)
                .values(phone_contact)
                .execute(conn); //IGNORE error, because there might be no user in `users`
        }

        Ok(())

        /*
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
                                                .collect::<Vec<HashedTeleNum>>(),
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
        */
    }

    fn get_contacts(
        &self,
        user: &UserDao,
    ) -> Result<Vec<ContactDto>, ::core::errors::ServiceError> {
        info!("queries/user/get_contacts");

        use core::schema::blacklist::dsl::{blacklist, hash_blocked, hash_blocker};
        use core::schema::contacts::dsl::{contacts, from_id, name, target_hash_tele_num};
        use core::schema::users::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        contacts
            .filter(from_id.eq(user.id))
            .inner_join(users.on(hash_tele_num.eq(target_hash_tele_num)))
            .left_join(
                blacklist.on(target_hash_tele_num
                    .eq(hash_blocked)
                    .and(hash_blocker.eq(&user.hash_tele_num))),
            )
            .select((
                id,
                name,
                tele_num,
                led,
                country_code,
                description,
                changed_at,
                profile_picture,
                hash_tele_num,
                hash_blocked.nullable(),
                xp,
                created_at,
                client_version,
                firebase_token.nullable(),
                access_token,
            ))
            .distinct()
            .load::<(
                Uuid,   //id
                String, //name
                String, //tele_num
                bool,   //led
                String, //cc
                String, //description
                chrono::NaiveDateTime,
                String,        //profile pic
                HashedTeleNum, //hash_blocked
                Option<HashedTeleNum>,
                i32,                   //XP
                chrono::NaiveDateTime, //created_at
                String,                //client
                Option<String>,        //firebase
                String,                //access_token
            )>(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
            .and_then(|values| {
                Ok(values
                    .into_iter()
                    .map(
                        |(
                            _id,
                            _name,
                            _tele_num,
                            _led,
                            _country_code,
                            _description,
                            _changed_at,
                            _profile_picture,
                            _hash_tele_num,
                            _blocked,
                            _xp,
                            _created_at,
                            _client_version,
                            _firebase_token,
                            _access_token,
                        )| {
                            let user_d = UserDao {
                                id: _id,
                                tele_num: _tele_num,
                                led: _led,
                                created_at: _created_at,
                                country_code: _country_code,
                                description: _description,
                                changed_at: _changed_at,
                                client_version: _client_version,
                                firebase_token: _firebase_token,
                                profile_picture: _profile_picture,
                                access_token: _access_token,
                                hash_tele_num: _hash_tele_num,
                                xp: _xp,
                            };

                            ContactDto::new(_name, _blocked.is_some(), user_d.into())
                        },
                    )
                    .collect())
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
