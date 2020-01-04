use crate::Pool;
use actix_web::web;
use core::errors::ServiceError;
use core::models::{
    Analytic, Blacklist, Contact, DowngradedUser, PhoneNumber, UsageStatisticEntry, User,
};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;
use web_contrib::push_notifications::NotifyService;

use crate::routes::user::{ResponseContact, UpdateUser};

use log::{error, info};

pub(crate) fn get_entry_by_tel_query(
    tele: &PhoneNumber,
    pool: &web::Data<Pool>,
) -> Result<User, ::core::errors::ServiceError> {
    info!("queries/user/get_entry_by_tel_query");

    use core::schema::users::dsl::*;

    let conn: &PgConnection = &pool.get().unwrap();

    let tele = tele.to_string();

    //    dbg!(&tele);

    let res = users
        .filter(tele_num.eq(tele))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })?;

    res.first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
}

pub(crate) fn get_contacts(
    user: &User,
    pool: &web::Data<Pool>,
) -> Result<Vec<ResponseContact>, ::core::errors::ServiceError> {
    info!("queries/user/get_contacts");

    use core::schema::blacklist::dsl::{blacklist, hash_blocked, hash_blocker};
    use core::schema::contacts::dsl::{contacts, from_id, target_hash_tele_num};
    use core::schema::users::dsl::*;

    let conn: &PgConnection = &pool.get().unwrap();

    contacts
        .filter(from_id.eq(user.id))
        .inner_join(users.on(hash_tele_num.eq(target_hash_tele_num)))
        .left_join(
            blacklist.on(target_hash_tele_num
                .eq(hash_blocked)
                .and(hash_blocker.eq(&user.hash_tele_num))
                .or(hash_tele_num
                    .eq(hash_blocker)
                    .and(hash_blocked.eq(&user.hash_tele_num)))),
        )
        //.filter(hash_blocked.is_null().or(hash_blocker.is_null()))
        .select((
            tele_num,
            led,
            country_code,
            description,
            changed_at,
            profile_picture,
            hash_tele_num,
            hash_blocked.nullable(),
        ))
        .distinct()
        .load::<(
            String,
            bool,
            String,
            String,
            chrono::NaiveDateTime,
            String,
            String,
            Option<String>,
        )>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|values| {
            dbg!(&values);
            Ok(values
                .into_iter()
                .map(
                    |(
                        _tele_num,
                        _led,
                        _country_code,
                        _description,
                        _changed_at,
                        _profile_picture,
                        _hash_tele_num,
                        _blocked,
                    )| {
                        ResponseContact::new(
                            _tele_num,
                            _led,
                            _country_code,
                            _description,
                            _changed_at,
                            _profile_picture,
                            _hash_tele_num,
                            _blocked,
                        )
                    },
                )
                .collect())
        })
}

pub(crate) fn analytics_user(
    pool: &web::Data<Pool>,
    user: &User,
) -> Result<Analytic, ::core::errors::ServiceError> {
    info!("queries/user/analytics_user");
    use core::schema::analytics::dsl::analytics;

    let ana = Analytic::my_from(user);
    let conn: &PgConnection = &pool.get().unwrap();

    let w = diesel::insert_into(analytics)
        .values(&ana)
        .get_result(conn)
        .map_err(|_db_error| {
            eprintln!("{}", _db_error);
            ServiceError::BadRequest("Could not log change".into())
        })?;

    Ok(w)
}

pub(crate) fn create_query(
    tel: &PhoneNumber,
    country_code: &str,
    version: &str,
    access_token: &str,
    pool: &web::Data<Pool>,
) -> Result<User, ServiceError> {
    info!("queries/user/create_query");
    use core::schema::users::dsl::users;

    let new_inv: User = User::my_from(&tel.to_string(), country_code, version, access_token);
    let conn: &PgConnection = &pool.get().unwrap();

    let ins = diesel::insert_into(users)
        .values(&new_inv)
        .get_result(conn)?;

    //dbg!(&ins);

    Ok(ins)
}

pub(crate) fn analytics_usage_statistics(
    pool: &web::Data<Pool>,
    user: &User,
) -> Result<UsageStatisticEntry, ::core::errors::ServiceError> {
    info!("queries/user/analytics_usage_statistics");
    use core::schema::usage_statistics::dsl::usage_statistics;

    let ana = UsageStatisticEntry::my_from(user);
    let conn: &PgConnection = &pool.get().unwrap();

    let w = diesel::insert_into(usage_statistics)
        .values(&ana)
        .get_result(conn)
        .map_err(|_db_error| {
            eprintln!("{}", _db_error);
            ServiceError::BadRequest("Could not log change".into())
        })?;

    Ok(w)
}

pub(crate) fn update_user_query(
    myid: Uuid,
    user: &UpdateUser,
    pool: &web::Data<Pool>,
    notify_service: &web::Data<NotifyService>,
) -> Result<User, ::core::errors::ServiceError> {
    info!("queries/user/update_user_query");
    use core::schema::users::dsl::{changed_at, client_version, description, id, led, users};

    let conn: &PgConnection = &pool.get().unwrap();

    let target = users.filter(id.eq(myid));

    /*
    let my_led = match &*user.led {
        "true" => true,
        "false" => false,
        _ => false,
    };
    */
    let my_led = user.led;

    diesel::update(target)
        .set((
            description.eq(user.description.to_string()),
            led.eq(my_led),
            changed_at.eq(chrono::Local::now().naive_local()),
            client_version.eq(user.client_version.clone()),
        ))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    users
        .filter(id.eq(myid))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|res_users| {
            Ok(res_users
                .first()
                .cloned()
                .ok_or_else(|| ServiceError::BadRequest("No user found".into()))?)
        })
        .and_then(|user| {
            if my_led {
                //Sending push notification
                sending_push_notifications(&user, pool, notify_service).map_err(|err| {
                    error!("{}", err);
                    ServiceError::BadRequest("Cannot send push notifications".to_string())
                })?;
            }

            Ok(user)
        })
}

fn sending_push_notifications(
    user: &User, //this is the sender
    pool: &web::Data<Pool>,
    notify_service: &web::Data<NotifyService>,
) -> Result<(), ServiceError> {
    info!("queries/user/sending_push_notifications");
    use core::schema::blacklist::dsl::{blacklist, hash_blocked, hash_blocker};
    use core::schema::contacts::dsl::{
        contacts, created_at, from_id, name, target_hash_tele_num,
        target_tele_num,
    };
    use core::schema::users::dsl::{firebase_token, users, hash_tele_num};

    let conn: &PgConnection = &pool.get().unwrap();

    type FirebaseToken = String;

    let my_contacts: Vec<(Contact, FirebaseToken)> = contacts
        .filter(from_id.eq(&user.id))
        .left_join(
            blacklist.on(target_hash_tele_num
                .eq(hash_blocked)
                .and(hash_blocker.eq(&user.hash_tele_num))
                .or(target_hash_tele_num
                    .eq(hash_blocker)
                    .and(hash_blocked.eq(&user.hash_tele_num)))),
        )
        .filter(hash_blocked.is_null().or(hash_blocker.is_null()))
        //.join(contacts.on()) //reverse
        .select((
            from_id,
            target_tele_num,
            created_at,
            name,
            target_hash_tele_num,
        ))
        .load::<Contact>(conn) //I got all my contacts here
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))?
        .into_iter()
        .take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS)
        .filter_map(|c| {
            // Now, I need for every contact, his/her firebase_token to send the notification to them
            let token = users
                .filter(hash_tele_num.eq(&c.target_hash_tele_num))
                .select(firebase_token)
                .load::<Option<String>>(conn)
                .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()));

            if let Err(err) = token {
                error!("{:?}", err);
                return None;
            }

            let token = token.unwrap().clone();
            let f = token.first();

            if let Some(Some(token)) = f {
                Some((c, token.clone()))
            } else {
                None
            }
        })
        .collect();

    dbg!(&my_contacts);

    for (contact, token) in my_contacts.iter() {
        //assert_eq!(user.id, contact.from_id);
        info!("{} ist motiviert zu {}", contact.name, token);
    }

    //FIXME check
    notify_service
        .clone()
        .into_inner()
        .service
        .push(my_contacts)?;

    Ok(())
}

/// Get the user by uid
pub(crate) fn get_query(
    myid: Uuid,
    my_access_token: &str,
    pool: &web::Data<Pool>,
) -> Result<User, ServiceError> {
    info!("queries/user/get_query");
    use core::schema::users::dsl::{access_token, id, users};

    let conn: &PgConnection = &pool.get().unwrap();

    users
        .filter(id.eq(myid).and(access_token.eq(my_access_token)))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|w| {
            w.first()
                .cloned()
                .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
        })
}

/// Get the user by uid
pub(crate) fn get_user_by_tele_num(
    phone_number: &PhoneNumber,
    my_access_token: &str,
    pool: &web::Data<Pool>,
) -> Result<User, ServiceError> {
    info!("queries/user/get_query");
    use core::schema::users::dsl::{access_token, tele_num, users};

    let conn: &PgConnection = &pool.get().unwrap();

    users
        .filter(
            tele_num
                .eq(phone_number.to_string())
                .and(access_token.eq(my_access_token)),
        )
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|w| {
            w.first()
                .cloned()
                .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
        })
}

pub(crate) fn update_profile_picture(
    uid: Uuid,
    ending: String,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    info!("queries/user/update_profile_picture");
    use core::schema::users::dsl::{changed_at, id, profile_picture, users};

    let conn: &PgConnection = &pool.get().unwrap();

    let target = users.filter(id.eq(uid));

    diesel::update(target)
        .set((
            changed_at.eq(chrono::Local::now().naive_local()),
            profile_picture.eq(format!("static/profile_pictures/{}.{}", uid, ending)),
        ))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    info!(
        "Updating profile {}",
        format!("static/profile_pictures/{}.{}", uid, ending)
    );

    Ok(())
}

pub(crate) fn update_token_query(
    uid: Uuid,
    token: String,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    info!("queries/push_notification/update_token_query");
    use core::schema::users::dsl::*;
    let conn: &PgConnection = &pool.get().unwrap();

    let target = users.filter(id.eq(uid));

    diesel::update(target)
        .set(
            firebase_token.eq(Some(token))
            //disabled 29.11.2019
            //changed_at.eq(chrono::Local::now().naive_local()),
        )
        .execute(conn)
        .map_err(|_db_error| {
            error!("db_error {}", _db_error);
            ServiceError::BadRequest("Updating state failed".into())
        })?;

    Ok(())
}
